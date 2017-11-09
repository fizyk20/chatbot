use super::dictionary::Dictionary;
use chrono::Duration;
use config::CONFIG;
use core::{BotCoreAPI, Command, Message, MessageContent};
use plugins::{Plugin, ResumeEventHandling};
use rand::{self, Rng};
use serde_json::{self, Value};

pub struct RandomChat {
    plugin_id: String,
    dict: Dictionary,
    dict_path: String,
    enabled: bool,
    probability: u8,
    timer_initialised: bool,
}

impl RandomChat {
    fn init_timer(&mut self, core: &mut BotCoreAPI) {
        core.schedule_timer(self.plugin_id.clone(), Duration::minutes(10));
        self.timer_initialised = true;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RandomChatConfig {
    enabled: bool,
    probability: u8,
    dictionary_path: Option<String>,
}

impl Plugin for RandomChat {
    fn create(id: String, config: Option<Value>) -> RandomChat {
        let config: RandomChatConfig = serde_json::from_value(config.unwrap()).unwrap();
        let dict_path = config.dictionary_path.unwrap_or(
            "dictionary.dat".to_owned(),
        );
        let dict = Dictionary::load(&dict_path).unwrap();
        RandomChat {
            plugin_id: id,
            dict,
            dict_path,
            enabled: config.enabled,
            probability: config.probability,
            timer_initialised: false,
        }
    }

    fn handle_message(&mut self, core: &mut BotCoreAPI, msg: Message) -> ResumeEventHandling {
        if !self.enabled {
            return ResumeEventHandling::Resume;
        }
        self.init_timer(core);
        if core.get_nick(&msg.channel.source) != msg.author {
            if let MessageContent::Text(txt) = msg.content {
                self.dict.learn_from_line(txt);
            }
        }
        if rand::thread_rng().gen_range(0, 100) < self.probability {
            let response = self.dict.generate_sentence();
            core.send(Message {
                author: "".to_owned(),
                channel: msg.channel,
                content: MessageContent::Text(response),
            }).unwrap();
            ResumeEventHandling::Resume
        } else {
            ResumeEventHandling::Resume
        }
    }

    fn handle_command(&mut self, core: &mut BotCoreAPI, command: Command) -> ResumeEventHandling {
        if command.params[0] == "gadaj" {
            let response = self.dict.generate_sentence();
            core.send(Message {
                author: "".to_owned(),
                channel: command.channel,
                content: MessageContent::Text(response),
            }).unwrap();
            ResumeEventHandling::Stop
        } else if command.params[0] == "random" {
            if command.params.len() < 2 {
                core.send(Message {
                    author: "".to_owned(),
                    channel: command.channel,
                    content: MessageContent::Text(format!("Not enough parameters")),
                }).unwrap();
                return ResumeEventHandling::Stop;
            }
            if command.params[1] == "enable" {
                self.enabled = true;
                let mut config = CONFIG.lock().unwrap();
                config
                    .plugins
                    .get_mut(&self.plugin_id)
                    .unwrap()
                    .config
                    .as_mut()
                    .map(|ref mut config| { config["enabled"] = Value::Bool(true); });
                core.send(Message {
                    author: "".to_owned(),
                    channel: command.channel,
                    content: MessageContent::Text(format!("RandomChat enabled.")),
                }).unwrap();
                ResumeEventHandling::Stop
            } else if command.params[1] == "disable" {
                self.enabled = false;
                let mut config = CONFIG.lock().unwrap();
                config
                    .plugins
                    .get_mut(&self.plugin_id)
                    .unwrap()
                    .config
                    .as_mut()
                    .map(|ref mut config| { config["enabled"] = Value::Bool(false); });
                core.send(Message {
                    author: "".to_owned(),
                    channel: command.channel,
                    content: MessageContent::Text(format!("RandomChat disabled.")),
                }).unwrap();
                ResumeEventHandling::Stop
            } else {
                core.send(Message {
                    author: "".to_owned(),
                    channel: command.channel,
                    content: MessageContent::Text(
                        format!("Unknown parameter value: {}", command.params[1]).to_string(),
                    ),
                }).unwrap();
                ResumeEventHandling::Stop
            }
        } else {
            ResumeEventHandling::Resume
        }
    }

    fn handle_timer(&mut self, core: &mut BotCoreAPI, id: String) -> ResumeEventHandling {
        if id == self.plugin_id {
            self.dict.save(&self.dict_path);
            self.init_timer(core);
            ResumeEventHandling::Stop
        } else {
            ResumeEventHandling::Resume
        }
    }
}
