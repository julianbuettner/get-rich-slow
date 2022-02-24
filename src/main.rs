mod account;
mod api;
mod asset;
mod blockchain;
mod config;
mod cryptoprice;
mod domainconfig;
mod error;
mod ethereum;
mod growth;
mod kraken;
mod nordigen;
mod scalable;

#[macro_use]
extern crate rocket;
use rocket::{Build, Rocket};

#[launch]
fn start() -> Rocket<Build> {
    println!("(Very slow brrrrrr noise)");

    let config = config::read_config();
    let domainconfig = domainconfig::DomainConfig::from_config(config);
    println!("Config read and parsed");

    api::get_rocket_build(domainconfig)
}
