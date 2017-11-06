use core::{BackendChannel, BackendEvent, MessageContent};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum BackendType {
    Irc,
    Slack,
    Discord,
}

quick_error! {
    #[derive(Debug)]
    pub enum BackendError {
        LoginDataInvalid {}
    }
}

pub type BackendResult<T> = Result<T, BackendError>;

pub trait BotBackend {
    fn get_type(&self) -> BackendType;
    fn connect(&mut self, login_data: Value) -> BackendResult<()>;
    fn join(&mut self, channel: &str) -> BackendResult<()>;
    fn send(&mut self, dst: BackendChannel, msg: MessageContent) -> BackendResult<()>;
    fn reconnect(&mut self) -> BackendResult<()>;
    fn get_event(&mut self) -> BackendResult<Option<BackendEvent>>;
}
