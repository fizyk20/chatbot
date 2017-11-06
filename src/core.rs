#[derive(Clone, Copy, Debug)]
pub struct BackendId(usize);

#[derive(Clone, Debug)]
pub enum BackendChannel {
    Channel(String),
    User(String),
    Group(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct Channel {
    backend: BackendId,
    dst: BackendChannel,
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
}

#[derive(Clone, Debug)]
pub struct Event {
    backend: BackendId,
    event: BackendEvent,
}
