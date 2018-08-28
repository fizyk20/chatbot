#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotConfig {
    pub command_char: String,
}

config!(BotConfig, "config.toml");
