use backends::*;
use config::CONFIG;
use core::{BackendId, Event};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, channel};

pub struct BotCore {
    event_rx: Receiver<Event>,
    backends: HashMap<BackendId, Box<BotBackend>>,
}

impl BotCore {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        let backends_def = &CONFIG.lock().unwrap().backends;
        let mut backends: HashMap<BackendId, Box<BotBackend>> = HashMap::new();
        for def in backends_def {
            let backend_id = BackendId(def.backend_id.clone());
            let backend = match def.backend_type {
                BackendType::Irc => {
                    IrcBackend::build_backend(
                        backend_id.clone(),
                        sender.clone(),
                        Some(def.config.clone()),
                    )
                }
                _ => unreachable!(),
            };
            backends.insert(backend_id, Box::new(backend));
        }
        BotCore {
            event_rx: receiver,
            backends,
        }
    }

    pub fn connect_all(&mut self) {
        for (_, backend) in self.backends.iter_mut() {
            backend.connect().unwrap();
        }
    }

    pub fn run(&mut self) {
        loop {
            let event = self.event_rx.recv();
            println!("Event: {:?}", event);
        }
    }
}
