use backends::*;
use core::*;
use irc::client::prelude::*;
use serde_json::{self, Value};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};

enum BackendState {
    Disconnected,
    Connected(IrcServer, JoinHandle<BackendResult<()>>),
}

pub struct IrcBackend {
    id: BackendId,
    config: Config,
    sender: Sender<Event>,
    state: BackendState,
}

impl BotBackendBuilder for IrcBackend {
    type Backend = Self;

    fn build_backend(
        backend_id: BackendId,
        sender: Sender<Event>,
        config: Option<Value>,
    ) -> IrcBackend {
        let config = config.expect(&format!(
            "No config given for IRC backend {:?}!",
            backend_id
        ));

        IrcBackend {
            id: backend_id.clone(),
            config: serde_json::from_value(config).ok().expect(&format!(
                "Invalid configuration supplied to IRC backend {:?}",
                backend_id
            )),
            sender,
            state: BackendState::Disconnected,
        }
    }
}

impl From<::irc::client::prelude::Message> for BackendEvent {
    fn from(msg: ::irc::client::prelude::Message) -> BackendEvent {
        BackendEvent::Other(format!("{:?}", msg.command))
    }
}

impl BotBackend for IrcBackend {
    fn get_type(&self) -> BackendType {
        BackendType::Irc
    }

    fn connect(&mut self) -> BackendResult<()> {
        let server = IrcServer::from_config(self.config.clone())?;
        let thread_server = server.clone();
        let thread_sender = self.sender.clone();
        let backend_id = self.id.clone();

        let handle = thread::spawn(move || -> BackendResult<()> {
            thread_server.identify()?;
            let _ = thread_server.for_each_incoming(|message| {
                let event = message.into();
                let _ = thread_sender.send(Event {
                    backend: backend_id.clone(),
                    event,
                });
            });
            Ok(())
        });

        self.state = BackendState::Connected(server, handle);
        Ok(())
    }

    fn join(&mut self, channel: &str) -> BackendResult<()> {
        Ok(())
    }

    fn send(&mut self, dst: BackendChannel, msg: MessageContent) -> BackendResult<()> {
        let state = match self.state {
            BackendState::Connected(ref server, _) => server,
            _ => return Err(BackendError::Disconnected(self.id.clone())),
        };
        let target = match dst {
            BackendChannel::Channel(c) => c,
            BackendChannel::User(u) => u,
            _ => return Err(BackendError::InvalidChannel(self.id.clone(), dst)),
        };
        let msg = match msg {
            MessageContent::Text(t) => t,
            MessageContent::Me(t) => t,
            _ => return Err(BackendError::InvalidMessage(self.id.clone(), msg)),
        };
        let message = Command::PRIVMSG(target, msg);
        state.send(message)?;
        Ok(())
    }

    fn reconnect(&mut self) -> BackendResult<()> {
        Ok(())
    }
}
