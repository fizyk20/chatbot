use backends::*;
use core::*;
use irc::client::prelude::*;
use serde_json::{self, Value};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};

pub enum IrcEvent {
    Connected,
    Disconnected,
}

enum BackendState {
    Disconnected,
    Connected(IrcServer, JoinHandle<()>),
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

impl BotBackend for IrcBackend {
    fn get_type(&self) -> BackendType {
        BackendType::Irc
    }

    fn connect(&mut self) -> BackendResult<()> {
        let server = IrcServer::from_config(self.config.clone()).unwrap();
        let thread_server = server.clone();
        let thread_sender = self.sender.clone();
        let backend_id = self.id.clone();

        let handle = thread::spawn(move || {
            thread_server.identify().unwrap();
            let _ = thread_server.for_each_incoming(|message| {
                let event = match message.command {
                    cmd => BackendEvent::Other(format!("{:?}", cmd)),
                };
                let _ = thread_sender.send(Event {
                    backend: backend_id.clone(),
                    event,
                });
            });
        });

        self.state = BackendState::Connected(server, handle);
        Ok(())
    }

    fn join(&mut self, channel: &str) -> BackendResult<()> {
        Ok(())
    }

    fn send(&mut self, dst: BackendChannel, msg: MessageContent) -> BackendResult<()> {
        Ok(())
    }

    fn reconnect(&mut self) -> BackendResult<()> {
        Ok(())
    }
}
