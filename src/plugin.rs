use core::{Channel, Message};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResumeEventHandling {
    Stop,
    Resume,
}

pub enum PluginEvent {
    None(ResumeEventHandling),
    Log(String, ResumeEventHandling),
    Send(Message, ResumeEventHandling),
}

pub trait Plugin {
    fn plugin_priority(&self, msg: Message) -> i16;
    fn handle_command(&mut self, user: &str, channel: Channel, params: Vec<String>) -> PluginEvent;
    fn handle_message(&mut self, data: Message) -> PluginEvent;
}
