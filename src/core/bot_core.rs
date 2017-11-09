use chrono::Duration;
use config::CONFIG;
use core::{Event, EventType, Message, SourceEvent, SourceId};
use logger::*;
use plugins::*;
use sources::*;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, channel};
use timer::{Guard, MessageTimer};

struct PluginDef {
    object: Box<Plugin>,
    priority: u8,
    subscriptions: HashMap<SourceId, HashSet<EventType>>,
}

pub struct BotCoreAPI {
    sources: HashMap<SourceId, Box<EventSource>>,
    logger: Logger,
    timer: MessageTimer<SourceEvent>,
    timer_guards: HashMap<String, Guard>,
}

/// The core of the bot
pub struct BotCore {
    event_rx: Receiver<SourceEvent>,
    plugins: Vec<PluginDef>,
    api: BotCoreAPI,
}

impl BotCore {
    /// Creates the core
    /// Sets up the event passing channel, reads the config and
    /// creates and configures appropriate event sources and plugins
    pub fn new() -> Self {
        let (sender, receiver) = channel();

        let sources_def = &CONFIG.lock().unwrap().sources;
        let mut sources = HashMap::new();
        for (id, def) in sources_def {
            let source_id = SourceId(id.clone());
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
        for (id, def) in plugins_def {
            let plugin: Box<Plugin> = match def.plugin_type {
                PluginType::RandomChat => Box::new(
                    RandomChat::create(id.clone(), def.config.clone()),
                ),
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

        let timer = MessageTimer::new(sender.clone());
        let log_folder = &CONFIG.lock().unwrap().log_folder;

        BotCore {
            event_rx: receiver,
            plugins,
            api: BotCoreAPI {
                sources,
                logger: Logger::new(log_folder),
                timer,
                timer_guards: HashMap::new(),
            },
        }
    }

    /// Calls connect() on all sources
    pub fn connect_all(&mut self) {
        for (_, source) in self.api.sources.iter_mut() {
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
                    Event::Timer(id) => self.handle_timer(id),
                    Event::Other(other) => println!("Other event: {}", other),
                }
            } else {
                println!("Channel error! {}", event.unwrap_err());
            }
        }
    }

    fn get_subscribers(plugins: &mut Vec<PluginDef>, event: EventType) -> Vec<&mut Box<Plugin>> {
        let mut subscribing_plugins: Vec<_> = plugins
            .iter_mut()
            .filter(|def| {
                def.subscriptions
                    .get(&SourceId("core".to_owned()))
                    .map(|events| events.contains(&event))
                    .unwrap_or(false)
            })
            .map(|def| (def.priority, &mut def.object))
            .collect();
        subscribing_plugins.sort_by_key(|x| x.0);
        subscribing_plugins.into_iter().map(|x| x.1).collect()
    }

    fn handle_direct_input(&mut self, src: SourceId, input: String) {
        let _ = self.api.logger.log_with_mode(
            &src.0,
            format!("Got direct input from {:?}: {}", src, input),
            LogMode::Console,
        );
    }

    fn handle_message(&mut self, src: SourceId, msg: Message) {
        let _ = self.api.logger.log_with_mode(
            &src.0,
            format!("Got a message from {:?}: {:?}", src, msg),
            LogMode::Console,
        );
    }

    fn handle_user_online(&mut self, src: SourceId, user: String) {
        let _ = self.api.logger.log_with_mode(
            &src.0,
            format!("User {} came online in {:?}", user, src),
            LogMode::Console,
        );
    }

    fn handle_user_offline(&mut self, src: SourceId, user: String) {
        let _ = self.api.logger.log_with_mode(
            &src.0,
            format!("User {} went offline in {:?}", user, src),
            LogMode::Console,
        );
    }

    fn handle_timer(&mut self, id: String) {
        let subscribing_plugins = Self::get_subscribers(&mut self.plugins, EventType::Timer);
        for plugin in subscribing_plugins {
            if plugin.handle_timer(&mut self.api, id.clone()) == ResumeEventHandling::Stop {
                break;
            }
        }
    }
}

impl BotCoreAPI {
    pub fn get_nick(&self, source_id: &SourceId) -> &str {
        self.sources
            .get(&source_id)
            .map(|source| source.get_nick())
            .unwrap()
    }

    pub fn schedule_timer(&mut self, id: String, after: Duration) {
        let guard = self.timer.schedule_with_delay(
            after,
            SourceEvent {
                source: SourceId("core".to_owned()),
                event: Event::Timer(id.clone()),
            },
        );
        let _ = self.timer_guards.insert(id, guard);
    }

    pub fn send(&mut self, msg: Message) -> SourceResult<()> {
        let source = self.sources.get_mut(&msg.channel.source).unwrap();
        let _ = self.logger.log(
            &msg.channel.source.0,
            msg.content.display_with_nick(source.get_nick()),
        );
        source.send(msg.channel.channel, msg.content)
    }
}
