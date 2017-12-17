use config::CONFIG;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SourceId(pub String);

/// Different kinds of communication channels
#[derive(Clone, Debug)]
pub enum Channel {
    None,
    Channel(String),
    User(String),
    Group(Vec<String>),
}

/// Content of a message
#[derive(Clone, Debug)]
pub enum MessageContent {
    /// Simple text message
    Text(String),
    /// An image - TODO
    Image,
    /// A /me type message
    Me(String),
}

impl MessageContent {
    pub fn display_with_nick(&self, nick: &str) -> String {
        match *self {
            MessageContent::Text(ref txt) => format!("<{}> {}", nick, txt),
            MessageContent::Me(ref txt) => format!("* {} {}", nick, txt),
            MessageContent::Image => format!("<{}> [Image]", nick),
        }
    }
}

/// Message content bundled with the author and the source channel
#[derive(Clone, Debug)]
pub struct Message {
    pub author: String,
    pub channel: Channel,
    pub content: MessageContent,
}

#[derive(Clone, Debug)]
pub struct Command {
    pub sender: String,
    pub channel: Channel,
    pub params: Vec<String>,
}

impl Message {
    pub fn parse_command(&self) -> Option<Command> {
        if let MessageContent::Text(txt) = self.content.clone() {
            let cmd_char = CONFIG.lock().unwrap().command_char.clone();
            if !txt.starts_with(&cmd_char) {
                return None;
            }
            let text = &txt[cmd_char.len()..];
            let words = text.split(" ");
            Some(Command {
                sender: self.author.clone(),
                channel: self.channel.clone(),
                params: words.into_iter().map(str::to_owned).collect(),
            })
        } else {
            None
        }
    }
}

/// Type representing events that can be sent by the sources
#[derive(Clone, Debug)]
pub enum Event {
    Connected,
    Disconnected,
    DirectInput(String),
    ReceivedMessage(Message),
    UserOnline(String),
    UserOffline(String, Option<String>),
    NickChange(String, String),
    Timer(String),
    Other(String),
}

/// Enum representing types of events
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    Connection,
    TextMessage,
    MeMessage,
    ImageMessage,
    UserStatus,
    Timer,
    Other,
}

impl Event {
    pub fn get_type(&self) -> EventType {
        match *self {
            Event::Connected | Event::Disconnected => EventType::Connection,
            Event::DirectInput(_) => EventType::TextMessage,
            Event::ReceivedMessage(ref msg) => {
                match msg.content {
                    MessageContent::Text(_) => EventType::TextMessage,
                    MessageContent::Me(_) => EventType::MeMessage,
                    MessageContent::Image => EventType::ImageMessage,
                }
            }
            Event::UserOnline(_) |
            Event::UserOffline(_, _) |
            Event::NickChange(_, _) => EventType::UserStatus,
            Event::Timer(_) => EventType::Timer,
            Event::Other(_) => EventType::Other,
        }
    }
}

/// The event bundled with the source ID
#[derive(Clone, Debug)]
pub struct SourceEvent {
    pub source: SourceId,
    pub event: Event,
}
