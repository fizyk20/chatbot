use config::CONFIG;
use core::{Event, EventType, Message, SourceEvent, SourceId};
use plugins::*;
use sources::*;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, channel};

struct PluginDef {
    object: Box<Plugin>,
    priority: u8,
    subscriptions: HashMap<SourceId, HashSet<EventType>>,
}

/// The core of the bot
pub struct BotCore {
    event_rx: Receiver<SourceEvent>,
    sources: HashMap<SourceId, Box<EventSource>>,
    plugins: Vec<PluginDef>,
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

        let plugins_def = &CONFIG.lock().unwrap().plugins;
        let mut plugins = vec![];
        for def in plugins_def {
            let plugin: Box<Plugin> = match def.plugin_type {
                //PluginType::RandomChat => RandomChat::new(def.config.clone()),
                //PluginType::MessagePasser => MessagePasser::new(def.config.clone()),
                _ => unimplemented!(),
            };
            plugins.push(PluginDef {
                priority: def.priority,
                subscriptions: def.subscriptions
                    .iter()
                    .map(|(id, set)| (SourceId(id.clone()), set.clone()))
                    .collect(),
                object: plugin,
            });
        }

        BotCore {
            event_rx: receiver,
            sources,
            plugins,
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
                match event.event {
                    Event::Connected => (),
                    Event::Disconnected => (),
                    Event::DirectInput(input) => self.handle_direct_input(event.source, input),
                    Event::ReceivedMessage(msg) => self.handle_message(event.source, msg),
                    Event::UserOnline(user) => self.handle_user_online(event.source, user),
                    Event::UserOffline(user) => self.handle_user_offline(event.source, user),
                    Event::Other(other) => println!("Other event: {}", other),
                }
            } else {
                println!("Channel error! {}", event.unwrap_err());
            }
        }
    }

    fn handle_direct_input(&mut self, src: SourceId, input: String) {
        println!("Got direct input from {:?}: {}", src, input);
    }

    fn handle_message(&mut self, src: SourceId, msg: Message) {
        println!("Got a message from {:?}: {:?}", src, msg);
    }

    fn handle_user_online(&mut self, src: SourceId, user: String) {
        println!("User {} came online in {:?}", user, src);
    }

    fn handle_user_offline(&mut self, src: SourceId, user: String) {
        println!("User {} went offline in {:?}", user, src);
    }
}
