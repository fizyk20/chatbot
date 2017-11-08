#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SourceId(pub String);

#[derive(Clone, Debug)]
pub enum Channel {
    None,
    Channel(String),
    User(String),
    Group(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct SourceChannel {
    pub source: SourceId,
    pub channel: Channel,
}

#[derive(Clone, Debug)]
pub enum MessageContent {
    Text(String),
    Image,
    Me(String),
}

#[derive(Clone, Debug)]
pub struct Message {
    author: String,
    channel: Channel,
    content: MessageContent,
}

#[derive(Clone, Debug)]
pub enum Event {
    Connected,
    Disconnected,
    ReceivedMessage {
        channel: SourceChannel,
        msg: Message,
    },
    UserOnline(String),
    UserOffline(String),
    Other(String),
}

#[derive(Clone, Debug)]
pub struct SourceEvent {
    pub source: SourceId,
    pub event: Event,
}

#[derive(Clone, Debug)]
pub enum SourceCommand {
    SendMessage {
        to: SourceChannel,
        msg: MessageContent,
    },
}
