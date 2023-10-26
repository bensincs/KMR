mod heartbeat;
mod kafka;
mod settings;
use lazy_static::lazy_static;
use log::info;
use settings::Settings;
use simple_logger::SimpleLogger;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex}, net::Ipv4Addr,
};

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new().unwrap();
    static ref RULES: HashMap<String, Ipv4Addr> = SETTINGS
    .kafka
    .rules
    .iter()
    .map(|r| (r.topic.clone(), r.multicast_addr.clone()))
    .collect::<HashMap<String, Ipv4Addr>>();
}

#[tokio::main]
async fn main() {
    // Configure logging
    SimpleLogger::new().env().init().unwrap();

    let mut all_handles = vec![];
    let neighbors: Arc<Mutex<HashMap<String, u128>>> = Arc::new(Mutex::new(HashMap::new()));
    let topic_hashes: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));


    if SETTINGS.heartbeat.enabled {
        let mut heartbeat_handles = vec![
            tokio::spawn(heartbeat::cast()),
            tokio::spawn(heartbeat::listen()),
            tokio::spawn(heartbeat::response_listen(neighbors.clone())),
            tokio::spawn(neighbor_printer(neighbors.clone())),
        ];
        all_handles.append(&mut heartbeat_handles);
    }

    if SETTINGS.kafka.enabled {
        let mut multicast_addresses: Vec<Ipv4Addr> = vec![];
        for (_, addr) in RULES.iter() {
            multicast_addresses.push(addr.clone());
        }

        let mut kafka_handles = vec![
            tokio::spawn(kafka::consume_and_cast(RULES.clone(), topic_hashes.clone())),
            tokio::spawn(kafka::receive_produce_and_respond(multicast_addresses)),
            tokio::spawn(topic_hashes_printer(topic_hashes.clone())),
        ];

        all_handles.append(&mut kafka_handles);
    }

    for handle in all_handles {
        handle.await.unwrap();
    }
}

async fn neighbor_printer(neighbors: Arc<Mutex<HashMap<String, u128>>>) {
    loop {
        println!("{:?}", neighbors.lock().unwrap());
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn topic_hashes_printer(topic_hashes: Arc<Mutex<HashMap<String, String>>>) {
    loop {
        println!("{:?}", topic_hashes.lock().unwrap());
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
