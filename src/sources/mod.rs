use core::{Channel, MessageContent, SourceEvent, SourceId};
use irc::error::Error as IrcError;
use serde_json::Value;
use std::sync::mpsc::Sender;

pub mod discord;
pub mod irc;
pub mod stdin;
pub mod slack;

pub use self::discord::DiscordSource;
pub use self::irc::IrcSource;
pub use self::slack::SlackSource;
pub use self::stdin::StdinSource;

/// Types of the supported event sources
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SourceType {
    Stdin,
    Irc,
    Slack,
    Discord,
}

/// An error type for the application
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
        Other(txt: String) {}
    }
}

/// A more concise result type
pub type SourceResult<T> = Result<T, SourceError>;

/// Trait representing a source of events
pub trait EventSource {
    /// Gets the bot's nickname on this source
    fn get_nick(&self) -> &str;
    /// Gets the type of the source
    fn get_type(&self) -> SourceType;
    /// Connects to the source
    fn connect(&mut self) -> SourceResult<()>;
    /// Joins a channel in the source
    fn join(&mut self, channel: &str) -> SourceResult<()>;
    /// Sends a message to the source
    fn send(&mut self, dst: Channel, msg: MessageContent) -> SourceResult<()>;
    /// Reconnects to the source
    fn reconnect(&mut self) -> SourceResult<()>;
}

/// Trait representing a type capable of creating an event source object
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
        let _ = |a: &mut EventSource| {
            a.reconnect().unwrap();
        };
    }
}
