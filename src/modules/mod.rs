use core::{BotCoreAPI, SourceEvent};
use serde_json::Value;

mod randomchat;
mod msg_pipe;

pub use self::msg_pipe::MsgPipe;
pub use self::randomchat::RandomChat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResumeEventHandling {
    Stop,
    Resume,
}

pub trait Module {
    fn create(id: String, config: Option<Value>) -> Self
    where
        Self: Sized;
    fn handle_event(&mut self, core: &mut BotCoreAPI, event: SourceEvent) -> ResumeEventHandling;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ModuleType {
    RandomChat,
    MsgPipe,
}
