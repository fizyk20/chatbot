mod msg_pipe;
mod patterns;
mod randomchat;

pub use self::msg_pipe::MsgPipe;
pub use self::patterns::Patterns;
pub use self::randomchat::RandomChat;
use config::CONFIG;
use universal_chat::{Channel, Message, MessageContent};

#[derive(Clone, Debug)]
pub struct Command {
    pub sender: String,
    pub channel: Channel,
    pub params: Vec<String>,
}

impl Command {
    fn from_msg<'a>(msg: &'a Message) -> Option<Command> {
        if let MessageContent::Text(txt) = msg.content.clone() {
            let cmd_char = CONFIG
                .lock()
                .ok()
                .expect("Couldn't lock CONFIG")
                .custom
                .command_char
                .clone();
            if !txt.starts_with(&cmd_char) {
                return None;
            }
            let text = &txt[cmd_char.len()..];
            let words = text.split(" ");
            Some(Command {
                sender: msg.author.clone(),
                channel: msg.channel.clone(),
                params: words.into_iter().map(str::to_owned).collect(),
            })
        } else {
            None
        }
    }
}
