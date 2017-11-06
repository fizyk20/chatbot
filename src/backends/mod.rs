use core::{BackendChannel, BackendEvent, MessageContent};
use std::convert::Into;

pub trait BotBackend {
    type LoginData;
    type ChannelName;
    type Error;

    fn connect(&mut self, login_data: Self::LoginData) -> Result<(), Self::Error>;
    fn join<T: Into<Self::ChannelName>>(&mut self, channel: T) -> Result<(), Self::Error>;
    fn send(&mut self, dst: BackendChannel, msg: MessageContent) -> Result<(), Self::Error>;
    fn reconnect(&mut self) -> Result<(), Self::Error>;
    fn get_event(&mut self) -> Result<Option<BackendEvent>, Self::Error>;
}
