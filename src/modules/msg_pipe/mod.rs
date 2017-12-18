use core::{BotCoreAPI, Channel, Event, Message, MessageContent, SourceEvent, SourceId};
use modules::{Module, ResumeEventHandling};
use serde_json::{self, Value};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MsgPipeConfig {
    source1: String,
    channel1: String,
    source2: String,
    channel2: String,
}

pub struct MsgPipe {
    endpoint1: (SourceId, Channel),
    endpoint2: (SourceId, Channel),
}

impl Module for MsgPipe {
    fn create(_: String, config: Option<Value>) -> MsgPipe {
        let config: MsgPipeConfig = serde_json::from_value(config.unwrap()).unwrap();
        MsgPipe {
            endpoint1: (SourceId(config.source1), Channel::Channel(config.channel1)),
            endpoint2: (SourceId(config.source2), Channel::Channel(config.channel2)),
        }
    }

    fn handle_event(&mut self, core: &mut BotCoreAPI, event: SourceEvent) -> ResumeEventHandling {
        let (target_source, target_channel) = if event.source == self.endpoint1.0 {
            self.endpoint2.clone()
        } else if event.source == self.endpoint2.0 {
            self.endpoint1.clone()
        } else {
            return ResumeEventHandling::Resume;
        };
        if let Event::ReceivedMessage(msg) = event.event {
            if let MessageContent::Text(txt) = msg.content {
                let new_content = format!("[{}]: {}", msg.author, txt);
                let message = Message {
                    author: "".to_owned(),
                    channel: target_channel,
                    content: MessageContent::Text(new_content),
                };
                core.send(&target_source, message).unwrap();
            }
        }
        ResumeEventHandling::Resume
    }
}
