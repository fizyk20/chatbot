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

impl EventSource for SlackSource {
    fn get_nick(&self) -> &str {
        self.state
            .start_response()
            .and_then(|resp| resp.slf.as_ref())
            .and_then(|user| user.name.as_ref().map(|s| s as &str))
            .unwrap_or("")
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
            client.run(&mut handler).map_err(|err| {
                SourceError::ConnectionError(id, err.to_string())
            })
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
        let target = match dst {
            Channel::Channel(c) => c,
            Channel::User(u) => u,
            _ => return Err(SourceError::InvalidChannel(self.id.clone(), dst)),
        };
        let msg = match msg {
            MessageContent::Text(t) => t,
            MessageContent::Me(t) => t,
            _ => return Err(SourceError::InvalidMessage(self.id.clone(), msg)),
        };
        //sender.send(message)?;
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

    fn on_event(&mut self, _: &RtmClient, event: ::slack::Event) {
        let event = match event {
            _ => Event::Other(format!("{:?}", event)),
        };
        let _ = self.sender.send(SourceEvent {
            source: self.id.clone(),
            event: event,
        });
    }
}
