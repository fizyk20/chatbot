extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[macro_use]
extern crate universal_chat;

mod config;
mod modules;

use config::CONFIG;
use modules::{Eightball, MsgPipe, Patterns, RandomChat};
use std::collections::HashMap;
use universal_chat::{Core, ModuleBuilder};

fn main() {
    let mut builders = HashMap::<String, ModuleBuilder>::new();
    builders.insert("MsgPipe".to_owned(), MsgPipe::create);
    builders.insert("RandomChat".to_owned(), RandomChat::create);
    builders.insert("Patterns".to_owned(), Patterns::create);
    builders.insert("Eightball".to_owned(), Eightball::create);
    // Create a core object
    let mut core = {
        let config = CONFIG.lock().ok().expect("Couldn't lock CONFIG");
        Core::new(&builders, &*config)
    };
    // Connect all event sources
    core.connect_all();
    // Run the event processing loop
    core.run();
}
