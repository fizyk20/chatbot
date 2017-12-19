use core::*;
use serde_json::{self, Value};
use slack::{EventHandler, RtmClient};
use slack_api::rtm::StartResponse;
use sources::*;
use std::thread::{self, JoinHandle};

/// A helper enum for SlackSource
enum SourceState {
    Disconnected,
    Connected(::slack::Sender, StartResponse, JoinHandle<SourceResult<()>>),
}

impl SourceState {
    fn start_response(&self) -> Option<&StartResponse> {
        if let SourceState::Connected(_, ref resp, _) = *self {
            Some(resp)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SlackConfig {
    token: String,
}

/// A Slack event source
pub struct SlackSource {
    /// the source ID
    id: SourceId,
    /// Slack client configuration data
    config: SlackConfig,
    /// Current state of the source
    state: SourceState,
    /// Event sender
    sender: Sender<SourceEvent>,
}

impl EventSourceBuilder for SlackSource {
    type Source = Self;

    /// Creates an SlackSource with the given configuration
    fn build_source(
        source_id: SourceId,
        sender: Sender<SourceEvent>,
        config: Option<Value>,
    ) -> SlackSource {
        let config = config.expect(&format!(
            "No config given for Slack source {:?}!",
            source_id
        ));
        let config: SlackConfig = serde_json::from_value(config).ok().expect(&format!(
            "Invalid configuration supplied to Slack source {:?}",
            source_id
        ));

        SlackSource {
            id: source_id.clone(),
            config,
            state: SourceState::Disconnected,
            sender,
        }
    }
}

impl SlackSource {
    fn get_id(&self) -> &str {
        self.state
            .start_response()
            .and_then(|resp| resp.slf.as_ref())
            .and_then(|user| user.id.as_ref().map(|s| s as &str))
            .unwrap_or("[no id]")
    }
}

impl EventSource for SlackSource {
    fn get_nick(&self) -> &str {
        self.state
            .start_response()
            .and_then(|resp| resp.slf.as_ref())
            .and_then(|user| user.name.as_ref().map(|s| s as &str))
            .unwrap_or("[no nick]")
    }

    fn get_type(&self) -> SourceType {
        SourceType::Slack
    }

    fn connect(&mut self) -> SourceResult<()> {
        let client = RtmClient::login(&self.config.token).unwrap();
        let sender = client.sender().clone();
        let src_sender = self.sender.clone();
        let response = client.start_response().clone();
        let id = self.id.clone();

        // create the event handling thread
        let handle = thread::spawn(move || -> SourceResult<()> {
            let mut handler = SlackHandler {
                id: id.clone(),
                sender: src_sender,
            };
            client
                .run(&mut handler)
                .map_err(|err| SourceError::ConnectionError(id, err.to_string()))
        });

        // save the server object and thread handle
        self.state = SourceState::Connected(sender, response, handle);
        Ok(())
    }

    fn join(&mut self, _channel: &str) -> SourceResult<()> {
        Ok(())
    }

    /// Sends a message to a user or a Slack channel
    fn send(&mut self, dst: Channel, msg: MessageContent) -> SourceResult<()> {
        let sender = match self.state {
            SourceState::Connected(ref sender, _, _) => sender,
            _ => return Err(SourceError::Disconnected(self.id.clone())),
        };
        let state = &self.state;
        let channel_id = match dst {
            Channel::Channel(c) => state
                .start_response()
                .and_then(|resp| get_id_by_channel(resp, &c))
                .map(|id| id.to_owned()),
            Channel::User(u) => state
                .start_response()
                .and_then(|resp| get_id_by_nick(resp, &u))
                .map(|id| id.to_owned()),
            _ => return Err(SourceError::InvalidChannel(self.id.clone(), dst)),
        };
        let msg = match msg {
            MessageContent::Text(t) => t,
            MessageContent::Me(t) => t,
            _ => return Err(SourceError::InvalidMessage(self.id.clone(), msg)),
        };
        channel_id.and_then(|cid| {
            let _ = sender.send_message(&cid, &msg);
            Some(())
        });
        Ok(())
    }

    fn reconnect(&mut self) -> SourceResult<()> {
        Ok(())
    }
}

struct SlackHandler {
    id: SourceId,
    sender: Sender<SourceEvent>,
}

impl EventHandler for SlackHandler {
    fn on_connect(&mut self, _: &RtmClient) {
        let _ = self.sender.send(SourceEvent {
            source: self.id.clone(),
            event: Event::Connected,
        });
    }

    fn on_close(&mut self, _: &RtmClient) {
        let _ = self.sender.send(SourceEvent {
            source: self.id.clone(),
            event: Event::Disconnected,
        });
    }

    fn on_event(&mut self, client: &RtmClient, event: ::slack::Event) {
        use slack::Event::*;
        use slack::Message::*;
        let events = match event {
            // ignore noise
            ReconnectUrl { .. } => vec![],
            UserTyping { .. } => vec![],
            // process other events
            PresenceChange {
                ref user,
                ref presence,
            } => {
                let resp = client.start_response();
                let nick = get_nick_by_id(resp, user);
                nick.map(|nick| match presence as &str {
                    "active" => vec![Event::UserOnline(nick.to_owned())],
                    "away" => vec![Event::UserOffline(nick.to_owned(), None)],
                    _ => vec![],
                }).unwrap_or_else(Vec::new)
            }
            Message(msg) => match *msg {
                Standard(msg) => {
                    if let (Some(sender), Some(channel), Some(text)) =
                        (msg.user, msg.channel, msg.text)
                    {
                        let resp = client.start_response();
                        let msg = ::core::Message {
                            author: get_nick_by_id(resp, &sender)
                                .unwrap_or("[no author]")
                                .to_owned(),
                            channel: Channel::Channel(
                                get_channel_by_id(resp, &channel)
                                    .unwrap_or("[invalid channel]")
                                    .to_owned(),
                            ),
                            content: MessageContent::Text(text),
                        };
                        vec![Event::ReceivedMessage(msg)]
                    } else {
                        vec![]
                    }
                }
                _ => vec![Event::Other(format!("{:?}", msg))],
            },
            _ => vec![Event::Other(format!("{:?}", event))],
        };
        for event in events {
            let _ = self.sender.send(SourceEvent {
                source: self.id.clone(),
                event: event,
            });
        }
    }
}

fn get_id_by_nick<'a, 'b>(start_resp: &'a StartResponse, nick: &'b str) -> Option<&'a str> {
    start_resp
        .users
        .as_ref()
        .and_then(|users| {
            users
                .into_iter()
                .find(|user| user.name.as_ref().map(|name| name == nick).unwrap_or(false))
        })
        .and_then(|user| user.id.as_ref())
        .map(|id| id as &str)
}

fn get_nick_by_id<'a, 'b>(start_resp: &'a StartResponse, id: &'b str) -> Option<&'a str> {
    start_resp
        .users
        .as_ref()
        .and_then(|users| {
            users
                .into_iter()
                .find(|user| user.id.as_ref().map(|uid| uid == id).unwrap_or(false))
        })
        .and_then(|user| user.name.as_ref())
        .map(|name| name as &str)
}

fn get_id_by_channel<'a, 'b>(
    start_resp: &'a StartResponse,
    channel_name: &'b str,
) -> Option<&'a str> {
    start_resp
        .channels
        .as_ref()
        .and_then(|channels| {
            channels.into_iter().find(|channel| {
                channel
                    .name
                    .as_ref()
                    .map(|name| name == channel_name)
                    .unwrap_or(false)
            })
        })
        .and_then(|channel| channel.id.as_ref())
        .map(|id| id as &str)
}

fn get_channel_by_id<'a, 'b>(start_resp: &'a StartResponse, id: &'b str) -> Option<&'a str> {
    start_resp
        .channels
        .as_ref()
        .and_then(|channels| {
            channels
                .into_iter()
                .find(|channel| channel.id.as_ref().map(|cid| cid == id).unwrap_or(false))
        })
        .and_then(|channel| channel.name.as_ref())
        .map(|name| name as &str)
}
