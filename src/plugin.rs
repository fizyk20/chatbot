use BotEvent;
use MessageData;

pub trait Plugin {
    fn plugin_priority(&self, user: &str, channel: &str, msg: &str) -> i16;
    fn handle_command(&mut self, user: &str, channel: &str, params: Vec<String>) -> BotEvent;
    fn handle_message(&mut self, data: MessageData) -> BotEvent;
}
