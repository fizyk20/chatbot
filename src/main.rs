extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate universal_chat;

mod modules;
mod config;

use config::CONFIG;
use modules::{MsgPipe, RandomChat};
use std::collections::HashMap;
use universal_chat::{Core, ModuleBuilder};

fn main() {
    let mut builders = HashMap::<String, ModuleBuilder>::new();
    builders.insert("MsgPipe".to_owned(), MsgPipe::create);
    builders.insert("RandomChat".to_owned(), RandomChat::create);
    // Create a core object
    let mut core = {
        let config = CONFIG.lock().unwrap();
        Core::new(&builders, &*config)
    };
    // Connect all event sources
    core.connect_all();
    // Run the event processing loop
    core.run();
}
