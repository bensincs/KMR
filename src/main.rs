mod heartbeat;
mod kafka;
mod settings;
use lazy_static::lazy_static;
use settings::Settings;
use simple_logger::SimpleLogger;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new().unwrap();
}

#[tokio::main]
async fn main() {
    // Configure logging
    SimpleLogger::new().env().init().unwrap();

    let mut all_handles = vec![];
    let neighbors: Arc<Mutex<HashMap<String, u128>>> = Arc::new(Mutex::new(HashMap::new()));

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
        let mut kafka_handles = kafka::start_tasks();
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
