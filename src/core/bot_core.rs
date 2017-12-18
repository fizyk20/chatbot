use chrono::Duration;
use config::CONFIG;
use core::{Event, EventType, Message, MessageContent, SourceEvent, SourceId};
use logger::*;
use modules::*;
use sources::*;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver};
use timer::{Guard, MessageTimer};

struct ModuleDef {
    object: Box<Module>,
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
    modules: Vec<ModuleDef>,
    api: BotCoreAPI,
}

impl BotCore {
    /// Creates the core
    /// Sets up the event passing channel, reads the config and
    /// creates and configures appropriate event sources and modules
    pub fn new() -> Self {
        let (sender, receiver) = channel();

        let mut sources = HashMap::new();
        {
            let sources_def = &CONFIG.lock().unwrap().sources;
            for (id, def) in sources_def {
                let source_id = SourceId(id.clone());
                let source: Box<EventSource> = match def.source_type {
                    SourceType::Irc => Box::new(IrcSource::build_source(
                        source_id.clone(),
                        sender.clone(),
                        def.config.clone(),
                    )),
                    SourceType::Stdin => Box::new(StdinSource::build_source(
                        source_id.clone(),
                        sender.clone(),
                        None,
                    )),
                    SourceType::Slack => Box::new(SlackSource::build_source(
                        source_id.clone(),
                        sender.clone(),
                        def.config.clone(),
                    )),
                    _ => unreachable!(),
                };
                sources.insert(source_id, source);
            }
        }

        let mut modules = vec![];
        {
            let modules_def = &CONFIG.lock().unwrap().modules;
            for (id, def) in modules_def {
                let module: Box<Module> = match def.module_type {
                    ModuleType::RandomChat => {
                        Box::new(RandomChat::create(id.clone(), def.config.clone()))
                    }
                    //ModuleType::MessagePasser => MessagePasser::new(def.config.clone()),
                    _ => unimplemented!(),
                };
                modules.push(ModuleDef {
                    priority: def.priority,
                    subscriptions: def.subscriptions
                        .iter()
                        .map(|(id, set)| (SourceId(id.clone()), set.iter().cloned().collect()))
                        .collect(),
                    object: module,
                });
            }
        }

        let timer = MessageTimer::new(sender.clone());
        let log_folder = CONFIG.lock().unwrap().log_folder.clone();

        BotCore {
            event_rx: receiver,
            modules,
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
                self.log_event(&event);
                self.handle_event(event);
            } else {
                println!("Channel error! {}", event.unwrap_err());
            }
        }
    }

    fn log_event(&mut self, event: &SourceEvent) {
        let text = match event.event {
            Event::ReceivedMessage(ref msg) => match msg.content {
                MessageContent::Text(ref txt) => format!("<{}> {}", msg.author, txt),
                MessageContent::Me(ref txt) => {
                    format!("* {} {}", self.api.get_nick(&event.source), txt)
                }
                MessageContent::Image => format!("[Image]"),
            },
            Event::Other(ref txt) => txt.clone(),
            _ => format!("{:?}", event.event),
        };
        self.api.logger.log(&event.source.0, text).unwrap();
    }

    fn get_subscribers<'a, 'b>(
        source_id: &'a SourceId,
        modules: &'b mut Vec<ModuleDef>,
        event: EventType,
    ) -> Vec<&'b mut Box<Module>> {
        let mut subscribing_modules: Vec<_> = modules
            .iter_mut()
            .filter(|def| {
                def.subscriptions
                    .get(&source_id)
                    .map(|events| events.contains(&event))
                    .unwrap_or(false)
            })
            .map(|def| (def.priority, &mut def.object))
            .collect();
        subscribing_modules.sort_by_key(|x| x.0);
        subscribing_modules.into_iter().map(|x| x.1).collect()
    }

    fn handle_event(&mut self, event: SourceEvent) {
        let subscribers =
            Self::get_subscribers(&event.source, &mut self.modules, event.event.get_type());
        for module in subscribers {
            if module.handle_event(&mut self.api, event.clone()) == ResumeEventHandling::Stop {
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

    pub fn send(&mut self, source_id: &SourceId, msg: Message) -> SourceResult<()> {
        let source = self.sources.get_mut(source_id).unwrap();
        let _ = self.logger.log(
            &source_id.0,
            msg.content.display_with_nick(source.get_nick()),
        );
        source.send(msg.channel, msg.content)
    }
}
