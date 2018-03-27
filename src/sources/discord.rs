use core::*;
use discord::{Discord, State};
use discord::Error as DiscordError;
use discord::model::{ChannelId, CurrentUser, UserId};
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
        let mut state = State::new(ready);
        let id = self.id.clone();
        let sender = self.sender.clone();
        let receiver = thread::spawn(move || loop {
            if let Ok(event) = connection.recv_event() {
                state.update(&event);
                Self::handle_event(event, &state, &id, &sender);
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
        let discord = self.discord()?;
        let channel_id = match dst {
            Channel::Channel(c) => self.get_channel_id(&c)?,
            Channel::User(u) => self.get_user_channel_id(&u)?,
            _ => return Err(SourceError::InvalidChannel(self.id.clone(), dst)),
        };
        let msg = match msg {
            MessageContent::Text(t) => t,
            MessageContent::Me(t) => t,
            _ => return Err(SourceError::InvalidMessage(self.id.clone(), msg)),
        };
        discord.send_message(channel_id, &msg, "", false)?;
        Ok(())
    }

    fn reconnect(&mut self) -> SourceResult<()> {
        Ok(())
    }
}

impl DiscordSource {
    fn handle_event(
        event: ::discord::model::Event,
        state: &State,
        id: &SourceId,
        sender: &Sender<SourceEvent>,
    ) {
        use discord::ChannelRef::*;
        use discord::model::Event::*;
        match event {
            MessageCreate(msg) => {
                // don't react to own messages
                if msg.author.id == state.user().id {
                    return;
                }
                let (author, channel) = match state
                    .find_channel(msg.channel_id)
                    .expect("Message from an unknown channel")
                {
                    Private(prv) => (
                        prv.recipient.name.to_owned(),
                        Channel::User(prv.recipient.name.clone()),
                    ),
                    Public(srv, publ) => {
                        let member = srv.members
                            .iter()
                            .find(|m| m.user.id == msg.author.id)
                            .map(|m| m.display_name().to_owned())
                            .unwrap_or(msg.author.name.to_owned());
                        (member, Channel::Channel(publ.name.to_owned()))
                    }
                    _ => {
                        return;
                    }
                };
                let msg = Message {
                    author,
                    channel,
                    content: MessageContent::Text(msg.content),
                };
                let _ = sender.send(SourceEvent {
                    source: id.clone(),
                    event: Event::ReceivedMessage(msg),
                });
            }
            _ => {
                let _ = sender.send(SourceEvent {
                    source: id.clone(),
                    event: Event::Other(format!("{:?}", event)),
                });
            }
        }
    }

    fn discord(&self) -> SourceResult<&Discord> {
        match self.state {
            SourceState::Connected(ref discord, _, _) => Ok(discord),
            _ => return Err(SourceError::Disconnected(self.id.clone())),
        }
    }

    fn get_user_id(&self, user: &str) -> SourceResult<UserId> {
        let discord = self.discord()?;
        for server in discord.get_servers()? {
            let members = discord.get_server_members(server.id)?;
            for member in members {
                if member.name == user {
                    return Ok(member.id);
                }
            }
        }
        Err(SourceError::InvalidChannel(
            self.id.clone(),
            Channel::User(user.to_owned()),
        ))
    }

    fn get_channel_id(&self, channel: &str) -> SourceResult<ChannelId> {
        let discord = self.discord()?;
        for server in discord.get_servers()? {
            let channels = discord.get_server_channels(server.id)?;
            for c in channels {
                if c.name == channel {
                    return Ok(c.id);
                }
            }
        }
        Err(SourceError::InvalidChannel(
            self.id.clone(),
            Channel::Channel(channel.to_owned()),
        ))
    }

    fn get_user_channel_id(&self, user: &str) -> SourceResult<ChannelId> {
        let user_id = self.get_user_id(user)?;
        Ok(self.discord()?.create_private_channel(user_id)?.id)
    }
}

impl From<DiscordError> for SourceError {
    fn from(err: DiscordError) -> Self {
        SourceError::Other(format!("{:?}", err))
    }
}
