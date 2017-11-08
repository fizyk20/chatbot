use config::CONFIG;
use core::{SourceEvent, SourceId};
use sources::*;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, channel};

pub struct BotCore {
    event_rx: Receiver<SourceEvent>,
    sources: HashMap<SourceId, Box<EventSource>>,
}

impl BotCore {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        let sources_def = &CONFIG.lock().unwrap().sources;
        let mut sources: HashMap<SourceId, Box<EventSource>> = HashMap::new();
        for def in sources_def {
            let source_id = SourceId(def.source_id.clone());
            let source = match def.source_type {
                SourceType::Irc => {
                    IrcSource::build_source(
                        source_id.clone(),
                        sender.clone(),
                        Some(def.config.clone()),
                    )
                }
                _ => unreachable!(),
            };
            sources.insert(source_id, Box::new(source));
        }
        BotCore {
            event_rx: receiver,
            sources,
        }
    }

    pub fn connect_all(&mut self) {
        for (_, source) in self.sources.iter_mut() {
            source.connect().unwrap();
        }
    }

    pub fn run(&mut self) {
        loop {
            let event = self.event_rx.recv();
            println!("Event: {:?}", event);
        }
    }
}
