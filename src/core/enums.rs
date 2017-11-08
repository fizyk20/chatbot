#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BackendId(pub String);

#[derive(Clone, Debug)]
pub enum BackendChannel {
    Channel(String),
    User(String),
    Group(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct Channel {
    pub backend: BackendId,
    pub dst: BackendChannel,
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
pub enum BackendEvent {
    Connected,
    Disconnected,
    ReceivedMessage {
        channel: BackendChannel,
        msg: Message,
    },
    UserOnline(String),
    UserOffline(String),
    Other(String),
}

#[derive(Clone, Debug)]
pub struct Event {
    pub backend: BackendId,
    pub event: BackendEvent,
}

#[derive(Clone, Debug)]
pub enum BackendCommand {
    SendMessage {
        to: BackendChannel,
        msg: MessageContent,
    },
}
