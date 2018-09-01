use modules::Command;
use rand::{thread_rng, Rng};
use toml::Value;
use universal_chat::{
    CoreAPI, Event, Message, MessageContent, Module, ResumeEventHandling, SourceEvent,
};

#[derive(Clone, Debug, Deserialize)]
struct EightballConfig {
    responses: Vec<String>,
}

pub struct Eightball {
    #[allow(unused)]
    module_id: String,
    enabled: bool,
    config: EightballConfig,
}

impl Eightball {
    pub fn create(id: String, config: Option<Value>) -> Box<Module> {
        let config: EightballConfig = config
            .expect("No config passed to Eightball")
            .try_into()
            .ok()
            .expect("Failed parsing a Value into EightballConfig");
        Box::new(Eightball {
            module_id: id,
            enabled: true,
            config,
        })
    }
}

impl Module for Eightball {
    fn handle_event(&mut self, core: &mut CoreAPI, event: SourceEvent) -> ResumeEventHandling {
        let SourceEvent { source, event } = event;
        if !self.enabled {
            return ResumeEventHandling::Resume;
        }
        match event {
            Event::ReceivedMessage(msg) => if let Some(cmd) = Command::from_msg(&msg) {
                if cmd.params[0] == "eightball" && cmd.params.len() > 1 {
                    //TODO: validate question?
                    if let Some(response) = thread_rng().choose(&self.config.responses) {
                        let response = response.replace("%s", &msg.author);
                        core.send(
                            &source,
                            Message {
                                author: "".to_owned(),
                                channel: msg.channel.clone(),
                                content: MessageContent::Text(response),
                            },
                        );
                    }
                }
                ResumeEventHandling::Resume
            } else {
                // ignore non-commands
                ResumeEventHandling::Resume
            },
            _ => ResumeEventHandling::Resume,
        }
    }
}
