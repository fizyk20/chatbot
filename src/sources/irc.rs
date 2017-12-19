use core::*;
use irc::client::prelude::*;
use serde_json::{self, Value};
use sources::*;
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};

/// A helper enum for IrcSource
enum SourceState {
    Disconnected,
    Connected(IrcServer, JoinHandle<SourceResult<()>>),
}

/// An IRC event source
pub struct IrcSource {
    /// bot's nick on the server
    nick: String,
    /// the source ID
    id: SourceId,
    /// IRC client configuration data
    config: Config,
    /// Event sender object
    sender: Sender<SourceEvent>,
    /// Current state of the source
    state: SourceState,
}

impl EventSourceBuilder for IrcSource {
    type Source = Self;

    /// Creates an IrcSource with the given configuration
    fn build_source(
        source_id: SourceId,
        sender: Sender<SourceEvent>,
        config: Option<Value>,
    ) -> IrcSource {
        let config = config.expect(&format!("No config given for IRC source {:?}!", source_id));
        let config: Config = serde_json::from_value(config).ok().expect(&format!(
            "Invalid configuration supplied to IRC source {:?}",
            source_id
        ));

        IrcSource {
            id: source_id.clone(),
            nick: config.nickname().to_owned(),
            config,
            sender,
            state: SourceState::Disconnected,
        }
    }
}

fn message_to_events(msg: ::irc::client::prelude::Message) -> Vec<Event> {
    use irc::client::prelude::Command::*;
    use irc::client::prelude::Response::*;
    let sender = msg.prefix
        .clone()
        .unwrap_or_else(|| "".to_string())
        .chars()
        .take_while(|c| *c != '!')
        .collect();
    match msg.command {
        PING(_, _) => vec![],
        PONG(_, _) => vec![],
        PRIVMSG(from, txt) => vec![
            Event::ReceivedMessage(::core::Message {
                author: sender,
                channel: if from.starts_with("#") {
                    Channel::Channel(from)
                } else {
                    Channel::User(from)
                },
                content: MessageContent::Text(txt),
            }),
        ],
        NICK(new_nick) => vec![Event::NickChange(sender, new_nick)],
        JOIN(_, _, _) => vec![Event::UserOnline(sender)],
        PART(_, comment) | QUIT(comment) => vec![Event::UserOffline(sender, comment)],
        Response(code, _, ref msg) if code == RPL_NAMREPLY => {
            if let &Some(ref msg) = msg {
                msg.split_whitespace()
                    .map(|x| Event::UserOnline(x.to_owned()))
                    .collect()
            } else {
                vec![]
            }
        }
        _ => vec![Event::Other(format!("{:?}", msg))],
    }
}

impl EventSource for IrcSource {
    fn get_nick(&self) -> &str {
        &self.nick
    }

    fn get_type(&self) -> SourceType {
        SourceType::Irc
    }

    fn connect(&mut self) -> SourceResult<()> {
        let server = IrcServer::from_config(self.config.clone())?;

        // create clones of some values for the event thread
        let thread_server = server.clone();
        let thread_sender = self.sender.clone();
        let source_id = self.id.clone();

        // create the event handling thread
        let handle = thread::spawn(move || -> SourceResult<()> {
            thread_server.identify()?;
            let _ = thread_server.for_each_incoming(|message| {
                let events = message_to_events(message);
                for event in events {
                    let _ = thread_sender.send(SourceEvent {
                        source: source_id.clone(),
                        event,
                    });
                }
            });
            Ok(())
        });

        // save the server object and thread handle
        self.state = SourceState::Connected(server, handle);
        Ok(())
    }

    fn join(&mut self, _channel: &str) -> SourceResult<()> {
        Ok(())
    }

    /// Sends a message to a user or an IRC channel
    fn send(&mut self, dst: Channel, msg: MessageContent) -> SourceResult<()> {
        let state = match self.state {
            SourceState::Connected(ref server, _) => server,
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
        let message = ::irc::client::prelude::Command::PRIVMSG(target, msg);
        state.send(message)?;
        Ok(())
    }

    fn reconnect(&mut self) -> SourceResult<()> {
        Ok(())
    }
}
