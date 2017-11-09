use core::EventType;
use plugins::PluginType;
use serde_json::{self, Value};
use sources::SourceType;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

/// Structure representing the bot configuration along with
/// the path to the file where it is saved
#[derive(Clone)]
pub struct Config {
    path: PathBuf,
    inner: ConfigInner,
}

/// A definition of an event source
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceDef {
    pub source_type: SourceType,
    pub config: Option<Value>,
}

/// A definition of a plugin
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginDef {
    pub plugin_type: PluginType,
    pub config: Option<Value>,
    pub priority: u8,
    pub subscriptions: HashMap<String, HashSet<EventType>>,
}

/// Inner structure with configuration data, read by Serde from a file
/// in JSON format
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigInner {
    pub command_char: String,
    pub sources: HashMap<String, SourceDef>,
    pub plugins: HashMap<String, PluginDef>,
}

impl Config {
    /// Loads configuration from a file and returns the resulting Config object
    pub fn new<P: AsRef<Path>>(path: P) -> Config {
        let path_buf = path.as_ref().to_path_buf();
        let mut file = fs::File::open(path).ok().expect(&format!(
            "Couldn't open file {:?}",
            path_buf
        ));
        let mut config = String::new();
        file.read_to_string(&mut config).expect(
            "Couldn't read from file",
        );
        Config {
            path: path_buf,
            inner: serde_json::from_str(&config).unwrap(),
        }
    }

    /// Saves the configuration to the file it was read from (overwrites the previous one)
    pub fn save(&self) {
        let mut file = fs::File::create(&self.path).ok().expect(&format!(
            "Couldn't create file {:?}",
            self.path
        ));
        let json = serde_json::to_string(&self.inner).unwrap();
        let _ = file.write(json.as_bytes());
    }
}

impl Deref for Config {
    type Target = ConfigInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Config {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.inner
    }
}

/// A global object to access the configuration
lazy_static! {
    pub static ref CONFIG : ::std::sync::Mutex<Config> = ::std::sync::Mutex::new(Config::new("config.json"));
}
