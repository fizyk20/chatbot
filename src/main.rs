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
mod plugins;

use core::BotCore;

fn main() {
    // Create a core object
    let mut core = BotCore::new();
    // Connect all event sources
    core.connect_all();
    // Run the event processing loop
    core.run();
}
