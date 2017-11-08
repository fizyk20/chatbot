use backends::BackendType;
use serde_json::{self, Value};
use std::fs;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Config {
    path: PathBuf,
    inner: ConfigInner,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendDef {
    pub backend_id: String,
    pub backend_type: BackendType,
    pub config: Value,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigInner {
    pub command_char: String,
    pub backends: Vec<BackendDef>,
}

impl Config {
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

lazy_static! {
    pub static ref CONFIG : ::std::sync::Mutex<Config> = ::std::sync::Mutex::new(Config::new("config.json"));
}
