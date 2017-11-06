extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod backends;
mod config;
mod core;
mod logger;
mod plugin;

fn main() {
    println!("Hello, world!");
}
