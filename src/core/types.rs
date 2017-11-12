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

/// Type representing events that can be sent by the sources
#[derive(Clone, Debug)]
pub enum Event {
    Connected,
    Disconnected,
    DirectInput(String),
    ReceivedMessage(Message),
    ReceivedCommand(Command),
    UserOnline(String),
    UserOffline(String),
    Timer(String),
    Other(String),
}

/// Enum representing types of events
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    Connection,
    Command,
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
            Event::DirectInput(_) |
            Event::ReceivedCommand(_) => EventType::Command,
            Event::ReceivedMessage(ref msg) => {
                match msg.content {
                    MessageContent::Text(_) => EventType::TextMessage,
                    MessageContent::Me(_) => EventType::MeMessage,
                    MessageContent::Image => EventType::ImageMessage,
                }
            }
            Event::UserOnline(_) |
            Event::UserOffline(_) => EventType::UserStatus,
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

#[derive(Clone, Debug)]
pub struct Command {
    pub sender: String,
    pub channel: Channel,
    pub params: Vec<String>,
}
