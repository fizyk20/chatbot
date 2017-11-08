use config::CONFIG;
use core::{Event, SourceEvent, SourceId};
use sources::*;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, channel};

/// The core of the bot
pub struct BotCore {
    event_rx: Receiver<SourceEvent>,
    sources: HashMap<SourceId, Box<EventSource>>,
}

impl BotCore {
    /// Creates the core
    /// Sets up the event passing channel, reads the config and
    /// creates and configures appropriate event sources and plugins
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        let sources_def = &CONFIG.lock().unwrap().sources;
        let mut sources = HashMap::new();
        for def in sources_def {
            let source_id = SourceId(def.source_id.clone());
            let source: Box<EventSource> = match def.source_type {
                SourceType::Irc => {
                    Box::new(IrcSource::build_source(
                        source_id.clone(),
                        sender.clone(),
                        def.config.clone(),
                    ))
                }
                SourceType::Stdin => {
                    Box::new(StdinSource::build_source(
                        source_id.clone(),
                        sender.clone(),
                        None,
                    ))
                }
                _ => unreachable!(),
            };
            sources.insert(source_id, source);
        }
        BotCore {
            event_rx: receiver,
            sources,
        }
    }

    /// Calls connect() on all sources
    pub fn connect_all(&mut self) {
        for (_, source) in self.sources.iter_mut() {
            source.connect().unwrap();
        }
    }

    /// Runs the event loop, processing them
    pub fn run(&mut self) {
        loop {
            let event = self.event_rx.recv();
            if let Ok(event) = event {
                let source_type =
                    if let Some(st) = self.sources.get(&event.source).map(
                        |source| source.get_type(),
                    )
                    {
                        st
                    } else {
                        println!("Got an event from an unknown source: {:?}", event.source);
                        continue;
                    };
                match source_type {
                    SourceType::Stdin => self.handle_stdin(event),
                    SourceType::Irc => self.handle_irc(event),
                    _ => unreachable!(),
                }
            } else {
                println!("Channel error! {}", event.unwrap_err());
            }
        }
    }

    fn handle_stdin(&mut self, event: SourceEvent) {
        match event.event {
            Event::DirectInput(s) => println!("Got input: {}", s),
            _ => println!("Got a weird event from stdin: {:?}", event),
        }
    }

    fn handle_irc(&mut self, event: SourceEvent) {
        println!("IRC event: {:?}", event.event);
    }
}
