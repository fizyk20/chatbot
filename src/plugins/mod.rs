use core::{BotCoreAPI, Command, Message};
use serde_json::Value;

mod randomchat;

pub use self::randomchat::RandomChat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResumeEventHandling {
    Stop,
    Resume,
}

pub trait Plugin {
    fn create(id: String, config: Option<Value>) -> Self
    where
        Self: Sized;
    fn handle_command(&mut self, core: &mut BotCoreAPI, command: Command) -> ResumeEventHandling;
    fn handle_message(&mut self, core: &mut BotCoreAPI, data: Message) -> ResumeEventHandling;
    fn handle_timer(&mut self, core: &mut BotCoreAPI, id: String) -> ResumeEventHandling;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PluginType {
    RandomChat,
    MessagePasser,
}
