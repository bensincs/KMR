mod heartbeat;

use std::{future::Future, collections::HashMap, sync::{Arc, Mutex}};

use simple_logger::SimpleLogger;
use tokio::task::JoinHandle;

// use socket2::{Domain, Protocol, Socket, Type};
// use std::{
//     mem::MaybeUninit,
//     net::{Ipv4Addr, SocketAddrV4},
//     thread::spawn,
// };

// use log::{info, warn};

// use rdkafka::{config::{ClientConfig, RDKafkaLogLevel}, message::ToBytes};
// use rdkafka::consumer::stream_consumer::StreamConsumer;
// use rdkafka::consumer::{CommitMode, Consumer};
// use rdkafka::message::Message;

// const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 69);
// const LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
// const MULTICAST_PORT: u16 = 6969;

// const BROKERS: &str = "localhost:29092";
// const GROUP_ID: &str = "ben";
// const TOPICS: &[&str] = &["ben"];


// async fn consume_and_cast(brokers: &str, group_id: &str, topics: &[&str]) {

//     let consumer: StreamConsumer = ClientConfig::new()
//         .set("group.id", group_id)
//         .set("bootstrap.servers", brokers)
//         .set("enable.partition.eof", "false")
//         .set("session.timeout.ms", "6000")
//         .set("enable.auto.commit", "true")
//         //.set("statistics.interval.ms", "30000")
//         //.set("auto.offset.reset", "smallest")
//         .set_log_level(RDKafkaLogLevel::Debug)
//         .create()
//         .expect("Consumer creation failed");

//     consumer
//         .subscribe(&topics.to_vec())
//         .expect("Can't subscribe to specified topics");

//     let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
//     let addr = SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT);

//     loop {
//         match consumer.recv().await {
//             Err(e) => {
//                 warn!("Kafka error: {}", e);
//             }
//             Ok(m) => {
//                 let payload = match m.payload_view::<str>() {
//                     None => "",
//                     Some(Ok(s)) => s,
//                     Some(Err(e)) => {
//                         warn!("Error while deserializing message payload: {:?}", e);
//                         ""
//                     }
//                 };

//                 info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
//                       m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());

//                 let timestamp_string = &m.timestamp().to_millis().unwrap().to_string();

//                 let timestamp_buffer = timestamp_string.as_bytes();

//                  socket.send_to(&timestamp_buffer, &addr.into()).unwrap();

//                 consumer.commit_message(&m, CommitMode::Async).unwrap();
//             }
//         };
//     }
// }

// async fn receive_and_produce () {
//     info!("Starting multicast listener");
//     let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
//         socket.set_reuse_address(true).unwrap();
//         socket.set_multicast_loop_v4(true).unwrap();
//         socket.set_multicast_ttl_v4(1).unwrap();

//         let addr = SocketAddrV4::new(LOCAL_ADDR, MULTICAST_PORT);
//         socket.bind(&addr.into()).unwrap();

//         socket
//             .join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)
//             .unwrap();

//         let mut buf = [MaybeUninit::uninit(); 1024];

//         loop {
//             let (_, _) = socket.recv_from(&mut buf).unwrap();

//             let buf = unsafe { std::mem::transmute::<_, [u8; 1024]>(buf) };

//             let payload = std::str::from_utf8(&buf).unwrap();

//             println!("{}", payload);
//         }
// }

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let mut neighbors: Arc<Mutex<HashMap<String, u128>>> = Arc::new(Mutex::new(HashMap::new()));

    let handles = vec![
        tokio::spawn(heartbeat::multicaster::cast()),
        tokio::spawn(heartbeat::multicaster::listen()),
        tokio::spawn(heartbeat::multicaster::response_listen(neighbors.clone())),
        tokio::spawn(neighbor_printer(neighbors.clone())),

    ];

    for handle in handles {
        handle.await.unwrap();
    }
}

async fn neighbor_printer(neighbors: Arc<Mutex<HashMap<String, u128>>>) {
    loop {
        println!("{:?}", neighbors.lock().unwrap());
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
