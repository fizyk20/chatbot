extern crate chrono;
extern crate irc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod sources;
mod config;
mod core;
mod logger;
mod plugin;

use core::BotCore;

fn main() {
    let mut core = BotCore::new();
    core.connect_all();
    core.run();
}
