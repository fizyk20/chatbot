use core::{Channel, MessageContent, SourceEvent, SourceId};
use irc::error::Error as IrcError;
use serde_json::Value;
use std::sync::mpsc::Sender;

pub mod irc;
pub mod stdin;

pub use self::irc::IrcSource;
pub use self::stdin::StdinSource;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SourceType {
    Stdin,
    Irc,
    Slack,
    Discord,
}

quick_error! {
    #[derive(Debug)]
    pub enum SourceError {
        Eof(id: SourceId) {}
        Disconnected(id: SourceId) {}
        ConnectionError(id: SourceId, txt: String) {}
        InvalidChannel(id: SourceId, ch: Channel) {}
        InvalidMessage(id: SourceId, msg: MessageContent) {}
        IrcError(err: IrcError) {
            from()
        }
    }
}

pub type SourceResult<T> = Result<T, SourceError>;

pub trait EventSource {
    fn get_type(&self) -> SourceType;
    fn connect(&mut self) -> SourceResult<()>;
    fn join(&mut self, channel: &str) -> SourceResult<()>;
    fn send(&mut self, dst: Channel, msg: MessageContent) -> SourceResult<()>;
    fn reconnect(&mut self) -> SourceResult<()>;
}

pub trait EventSourceBuilder {
    type Source: EventSource;
    fn build_source(
        source_id: SourceId,
        sender: Sender<SourceEvent>,
        config: Option<Value>,
    ) -> Self::Source;
}

#[cfg(test)]
mod test {
    use sources::*;

    #[test]
    fn test_object_safety() {
        // if this compiles, EventSource can be used as a trait object
        let f = |a: &mut EventSource| { a.reconnect().unwrap(); };
    }
}
