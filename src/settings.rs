use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::{env, net::Ipv4Addr};

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Kafka {
    pub enabled: bool,
    pub multicast_port: u16,
    pub response_port: u16,
    pub brokers: String,
    pub group_id: String,
    pub origin_id: String,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub topic: String,
    pub multicast_addr: Ipv4Addr,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Heartbeat {
    pub enabled: bool,
    pub multicast_addr: String,
    pub multicast_port: u16,
    pub response_port: u16,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub kafka: Kafka,
    pub heartbeat: Heartbeat,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("kmr").separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
