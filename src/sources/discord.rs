use core::*;
use discord::Discord;
use discord::model::CurrentUser;
use serde_json::{self, Value};
use sources::*;
use std::thread::{self, JoinHandle};

/// A helper enum for DiscordSource
enum SourceState {
    Disconnected,
    Connected(Discord, CurrentUser, JoinHandle<()>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DiscordConfig {
    token: String,
}

/// A Discord event source
pub struct DiscordSource {
    /// the source ID
    id: SourceId,
    /// Discord client configuration data
    config: DiscordConfig,
    /// Current state of the source
    state: SourceState,
    /// Event sender
    sender: Sender<SourceEvent>,
}

impl EventSourceBuilder for DiscordSource {
    type Source = Self;

    /// Creates an DiscordSource with the given configuration
    fn build_source(
        source_id: SourceId,
        sender: Sender<SourceEvent>,
        config: Option<Value>,
    ) -> DiscordSource {
        let config = config.expect(&format!(
            "No config given for Discord source {:?}!",
            source_id
        ));
        let config: DiscordConfig = serde_json::from_value(config).ok().expect(&format!(
            "Invalid configuration supplied to Discord source {:?}",
            source_id
        ));

        DiscordSource {
            id: source_id.clone(),
            config,
            state: SourceState::Disconnected,
            sender,
        }
    }
}

impl EventSource for DiscordSource {
    fn get_nick(&self) -> &str {
        if let SourceState::Connected(_, ref user, _) = self.state {
            &user.username
        } else {
            ""
        }
    }

    fn get_type(&self) -> SourceType {
        SourceType::Discord
    }

    fn connect(&mut self) -> SourceResult<()> {
        let discord =
            Discord::from_bot_token(&self.config.token).expect("Couldn't create Discord struct");
        let (mut connection, ready) = discord.connect().expect("Couldn't connect to Discord");
        let _ = self.sender.send(SourceEvent {
            source: self.id.clone(),
            event: Event::Other(format!("{:?}", ready)),
        });
        let sender = self.sender.clone();
        let receiver = thread::spawn(move || loop {
            if let Ok(event) = connection.recv_event() {
                Self::handle_event(event, &sender);
            } else {
                break;
            }
        });
        let user = discord
            .get_current_user()
            .expect("Couldn't get current user");
        self.state = SourceState::Connected(discord, user, receiver);
        Ok(())
    }

    fn join(&mut self, channel: &str) -> SourceResult<()> {
        Ok(())
    }

    fn send(&mut self, dst: Channel, msg: MessageContent) -> SourceResult<()> {
        Ok(())
    }

    fn reconnect(&mut self) -> SourceResult<()> {
        Ok(())
    }
}

impl DiscordSource {
    fn handle_event(event: ::discord::model::Event, sender: &Sender<SourceEvent>) {}
}
