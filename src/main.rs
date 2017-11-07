extern crate chrono;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod backends;
mod config;
mod core;
mod logger;
mod plugin;

use config::CONFIG;

fn main() {
    println!("Hello, world!");
    println!("command_char: {}", CONFIG.lock().unwrap().command_char);
}
