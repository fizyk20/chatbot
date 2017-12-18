extern crate chrono;
extern crate irc;
extern crate slack;
extern crate slack_api;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;
extern crate rand;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate timer;

mod sources;
mod config;
mod core;
mod logger;
mod modules;

use core::BotCore;

fn main() {
    // Create a core object
    let mut core = BotCore::new();
    // Connect all event sources
    core.connect_all();
    // Run the event processing loop
    core.run();
}
