use modules::Command;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use toml::Value;
use universal_chat::{
    CoreAPI, Event, Message, MessageContent, Module, ResumeEventHandling, SourceEvent,
};

fn deserialize_regex<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Regex, D::Error> {
    String::deserialize(deserializer)
        .map(|s| Regex::new(&s).ok().expect(&format!("Invalid regex: {}", s)))
}

#[derive(Clone, Debug, Deserialize)]
struct Pattern {
    #[serde(deserialize_with = "deserialize_regex")]
    pattern: Regex,
    response: String,
}

#[derive(Clone, Debug, Deserialize)]
struct PatternsConfig {
    patterns: Vec<Pattern>,
}

pub struct Patterns {
    #[allow(unused)]
    module_id: String,
    enabled: bool,
    config: PatternsConfig,
}

impl Patterns {
    pub fn create(id: String, config: Option<Value>) -> Box<Module> {
        let config: PatternsConfig = config
            .expect("No config passed to Patterns")
            .try_into()
            .ok()
            .expect("Failed parsing a Value into PatternsConfig");
        Box::new(Patterns {
            module_id: id,
            enabled: true,
            config,
        })
    }
}

impl Module for Patterns {
    fn handle_event(&mut self, core: &mut CoreAPI, event: SourceEvent) -> ResumeEventHandling {
        let SourceEvent { source, event } = event;
        if !self.enabled {
            return ResumeEventHandling::Resume;
        }
        match event {
            Event::ReceivedMessage(msg) => if Command::from_msg(&msg).is_some() {
                // ignore commands
                ResumeEventHandling::Resume
            } else {
                if let MessageContent::Text(txt) = msg.content {
                    for pattern in &self.config.patterns {
                        if pattern.pattern.is_match(&txt) {
                            core.send(
                                &source,
                                Message {
                                    author: "".to_owned(),
                                    channel: msg.channel.clone(),
                                    content: MessageContent::Text(pattern.response.clone()),
                                },
                            );
                        }
                    }
                }
                ResumeEventHandling::Resume
            },
            _ => ResumeEventHandling::Resume,
        }
    }
}
