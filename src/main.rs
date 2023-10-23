mod heartbeat;
mod kafka;

use std::{collections::HashMap, sync::{Arc, Mutex}};

use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() {
    SimpleLogger::new().env().init().unwrap();

    let neighbors: Arc<Mutex<HashMap<String, u128>>> = Arc::new(Mutex::new(HashMap::new()));

    let heartbeat_handles = vec![
        tokio::spawn(heartbeat::cast()),
        tokio::spawn(heartbeat::listen()),
        tokio::spawn(heartbeat::response_listen(neighbors.clone()))
    ];

    let kafka_handles = kafka::start_tasks();

    for handle in kafka_handles {
        handle.await.unwrap();
    }

    for handle in heartbeat_handles {
        handle.await.unwrap();
    }
}
