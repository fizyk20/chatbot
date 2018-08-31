use toml::Value;
use universal_chat::{
    Channel, CoreAPI, Event, Message, MessageContent, Module, ResumeEventHandling, SourceEvent,
    SourceId,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Endpoint {
    source: String,
    channel: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MsgPipe {
    endpoints: Vec<Endpoint>,
}

impl MsgPipe {
    pub fn create(_: String, config: Option<Value>) -> Box<Module> {
        let m: Self = config
            .expect("No config passed to MsgPipe")
            .try_into()
            .ok()
            .expect("Failed parsing a Value into MsgPipe");
        Box::new(m)
    }
}

impl Module for MsgPipe {
    fn handle_event(&mut self, core: &mut CoreAPI, event: SourceEvent) -> ResumeEventHandling {
        if let Event::ReceivedMessage(msg) = event.event {
            if let MessageContent::Text(txt) = msg.content {
                let new_content = format!("[{}]: {}", msg.author, txt);
                let source = &event.source.0;
                let channel = &msg.channel;
                if !self.endpoints.iter().any(|endpoint| {
                    source == &endpoint.source
                        && *channel == Channel::Channel(endpoint.channel.clone())
                }) {
                    return ResumeEventHandling::Resume;
                }
                for endpoint in &self.endpoints {
                    let source = SourceId(endpoint.source.clone());
                    let channel = Channel::Channel(endpoint.channel.clone());
                    if event.source == source && msg.channel == channel {
                        continue;
                    }
                    let message = Message {
                        author: "".to_owned(),
                        channel,
                        content: MessageContent::Text(new_content.clone()),
                    };
                    core.send(&source, message);
                }
            }
        }
        ResumeEventHandling::Resume
    }
}
