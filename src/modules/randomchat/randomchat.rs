use super::dictionary::Dictionary;
use chrono::Duration;
use config::CONFIG;
use modules::Command;
use rand::{self, Rng};
use toml::Value;
use universal_chat::{
    CoreAPI, Event, Message, MessageContent, Module, ResumeEventHandling, SourceEvent, SourceId,
};

pub struct RandomChat {
    module_id: String,
    dict: Dictionary,
    dict_path: String,
    enabled: bool,
    probability: u8,
    timer_initialised: bool,
}

impl RandomChat {
    fn init_timer(&mut self, core: &mut CoreAPI) {
        core.schedule_timer(self.module_id.clone(), Duration::minutes(10));
        self.timer_initialised = true;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RandomChatConfig {
    enabled: bool,
    probability: u8,
    dictionary_path: Option<String>,
}

impl RandomChat {
    pub fn create(id: String, config: Option<Value>) -> Box<Module> {
        let config: RandomChatConfig = config
            .expect("No config passed to RandomChat")
            .try_into()
            .ok()
            .expect("Failed parsing a Value into RandomChatConfig");
        let dict_path = config
            .dictionary_path
            .unwrap_or("dictionary.dat".to_owned());
        let dict = Dictionary::load(&dict_path)
            .ok()
            .expect("Dictionary::load failed");
        Box::new(RandomChat {
            module_id: id,
            dict,
            dict_path,
            enabled: config.enabled,
            probability: config.probability,
            timer_initialised: false,
        })
    }
}

impl Module for RandomChat {
    fn handle_event(&mut self, core: &mut CoreAPI, event: SourceEvent) -> ResumeEventHandling {
        let SourceEvent { source, event } = event;
        match event {
            Event::ReceivedMessage(msg) => if let Some(cmd) = Command::from_msg(&msg) {
                self.handle_command(core, source, cmd)
            } else {
                self.handle_message(core, source, msg)
            },
            Event::Timer(id) => self.handle_timer(core, id),
            _ => ResumeEventHandling::Resume,
        }
    }
}

impl RandomChat {
    fn handle_message(
        &mut self,
        core: &mut CoreAPI,
        src: SourceId,
        msg: Message,
    ) -> ResumeEventHandling {
        if !self.enabled {
            return ResumeEventHandling::Resume;
        }
        self.init_timer(core);
        if core.get_nick(&src) != msg.author {
            if let MessageContent::Text(txt) = msg.content {
                self.dict.learn_from_line(txt);
            }
        }
        if rand::thread_rng().gen_range(0, 100) < self.probability {
            let response = self.dict.generate_sentence();
            core.send(
                &src,
                Message {
                    author: "".to_owned(),
                    channel: msg.channel,
                    content: MessageContent::Text(response),
                },
            ).ok()
                .expect("core.send() failed");
            ResumeEventHandling::Resume
        } else {
            ResumeEventHandling::Resume
        }
    }

    fn handle_command(
        &mut self,
        core: &mut CoreAPI,
        src: SourceId,
        command: Command,
    ) -> ResumeEventHandling {
        if command.params[0] == "gadaj" {
            let response = self.dict.generate_sentence();
            core.send(
                &src,
                Message {
                    author: "".to_owned(),
                    channel: command.channel,
                    content: MessageContent::Text(response),
                },
            ).ok()
                .expect("core.send() failed");
            ResumeEventHandling::Stop
        } else if command.params[0] == "random" {
            if command.params.len() < 2 {
                core.send(
                    &src,
                    Message {
                        author: "".to_owned(),
                        channel: command.channel,
                        content: MessageContent::Text(format!("Not enough parameters")),
                    },
                ).ok()
                    .expect("core.send() failed");
                return ResumeEventHandling::Stop;
            }
            if command.params[1] == "enable" {
                self.enabled = true;
                let mut config = CONFIG.lock().ok().expect("Couldn't lock CONFIG");
                config
                    .modules
                    .get_mut(&self.module_id)
                    .expect(&format!("Couldn't find module {:?}", self.module_id))
                    .config
                    .as_mut()
                    .map(|ref mut config| {
                        config["enabled"] = Value::Boolean(true);
                    });
                core.send(
                    &src,
                    Message {
                        author: "".to_owned(),
                        channel: command.channel,
                        content: MessageContent::Text(format!("RandomChat enabled.")),
                    },
                ).ok()
                    .expect("core.send() failed");
                ResumeEventHandling::Stop
            } else if command.params[1] == "disable" {
                self.enabled = false;
                let mut config = CONFIG.lock().ok().expect("Couldn't lock CONFIG");
                config
                    .modules
                    .get_mut(&self.module_id)
                    .expect(&format!("Couldn't find module {:?}", self.module_id))
                    .config
                    .as_mut()
                    .map(|ref mut config| {
                        config["enabled"] = Value::Boolean(false);
                    });
                core.send(
                    &src,
                    Message {
                        author: "".to_owned(),
                        channel: command.channel,
                        content: MessageContent::Text(format!("RandomChat disabled.")),
                    },
                ).ok()
                    .expect("core.send() failed");
                ResumeEventHandling::Stop
            } else {
                core.send(
                    &src,
                    Message {
                        author: "".to_owned(),
                        channel: command.channel,
                        content: MessageContent::Text(
                            format!("Unknown parameter value: {}", command.params[1]).to_string(),
                        ),
                    },
                ).ok()
                    .expect("core.send() failed");
                ResumeEventHandling::Stop
            }
        } else {
            ResumeEventHandling::Resume
        }
    }

    fn handle_timer(&mut self, core: &mut CoreAPI, id: String) -> ResumeEventHandling {
        if id == self.module_id {
            let _ = self.dict.save(&self.dict_path);
            self.init_timer(core);
            ResumeEventHandling::Stop
        } else {
            ResumeEventHandling::Resume
        }
    }
}
