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

/// Channel bundled with a source ID
#[derive(Clone, Debug)]
pub struct SourceChannel {
    pub source: SourceId,
    pub channel: Channel,
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

/// Message content bundled with the author and the source channel
#[derive(Clone, Debug)]
pub struct Message {
    author: String,
    channel: SourceChannel,
    content: MessageContent,
}

/// Type representing events that can be sent by the sources
#[derive(Clone, Debug)]
pub enum Event {
    Connected,
    Disconnected,
    DirectInput(String),
    ReceivedMessage(Message),
    UserOnline(String),
    UserOffline(String),
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
}

/// The event bundled with the source ID
#[derive(Clone, Debug)]
pub struct SourceEvent {
    pub source: SourceId,
    pub event: Event,
}
