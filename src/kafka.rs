use std::{
    collections::HashMap,
    mem::MaybeUninit,
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use socket2::{Domain, Protocol, Socket, Type};

use tokio::task::JoinHandle;

use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::Message;
use rdkafka::{
    config::{ClientConfig, RDKafkaLogLevel},
    message::ToBytes,
};
use rdkafka::{
    consumer::stream_consumer::StreamConsumer,
    producer::{FutureProducer, FutureRecord},
};

use log::{info, warn};

const MULTICAST_PORT: u16 = 6971;
const LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const BROKERS: &str = "localhost:29092";
const GROUP_ID: &str = "ben";
const ORIGIN_ID: &str = "ben";

#[derive(Serialize, Deserialize, Debug)]
struct MessageWrapper {
    pub payload: String,
    pub key: Vec<u8>,
    pub topic: String,
    pub origin: String,
}

pub fn start_tasks() -> Vec<JoinHandle<()>> {

    let mut mappings: HashMap<String, Ipv4Addr> = HashMap::new();
    let mut reverse_mapping: HashMap<Ipv4Addr, String> = HashMap::new();

    mappings.insert("test".to_string(), Ipv4Addr::new(224, 0, 0, 71));
    mappings.insert("test2".to_string(), Ipv4Addr::new(224, 0, 0, 72));
    mappings.iter().for_each(|(k, v)| {
        reverse_mapping.insert(*v, k.to_string());
    });

    let tasks = vec![
        tokio::spawn(consume_and_cast(mappings)),
        tokio::spawn(receive_and_produce(reverse_mapping)),
    ];

    return tasks;
}

async fn consume_and_cast(mappings: HashMap<String, Ipv4Addr>) {
    let topics = mappings.keys().map(|s| s.as_str()).collect::<Vec<&str>>();

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", GROUP_ID)
        .set("bootstrap.servers", BROKERS)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&topics)
        .expect("Can't subscribe to specified topics");

    let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

    loop {
        match consumer.recv().await {
            Err(e) => {
                warn!("Kafka error: {}", e);
            }
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        warn!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };

                let multicast_addr = mappings.get(m.topic()).unwrap();
                let sock_addr = SocketAddrV4::new(*multicast_addr, MULTICAST_PORT);

                let message = MessageWrapper {
                    payload: payload.to_string(),
                    key: m.key().unwrap().to_vec(),
                    topic: m.topic().to_string(),
                    origin: ORIGIN_ID.to_string(),
                };

                info!("Sending to: {:?}", sock_addr);

                let message = bincode::serialize(&message).unwrap();

                socket.send_to(&message, &sock_addr.into()).unwrap();

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}

async fn receive_and_produce(mappings: HashMap<Ipv4Addr, String>) {
    let multicast_addresses = mappings.keys().collect::<Vec<&Ipv4Addr>>();

    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", BROKERS)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.set_multicast_loop_v4(true).unwrap();
    socket.set_multicast_ttl_v4(1).unwrap();

    multicast_addresses.iter().for_each(|addr| {
        socket
            .join_multicast_v4(addr, &Ipv4Addr::UNSPECIFIED)
            .unwrap();
    });

    let addr = SocketAddrV4::new(LOCAL_ADDR, MULTICAST_PORT);
    socket.bind(&addr.into()).unwrap();

    let mut buf = [MaybeUninit::uninit(); 1024];

    loop {
        let (_, _) = socket.recv_from(&mut buf).unwrap();

        let buf = unsafe { std::mem::transmute::<_, [u8; 1024]>(buf) };

        let message: MessageWrapper = bincode::deserialize(&buf).unwrap();

        info!("Received message: {:?}", message);

        if message.origin != ORIGIN_ID {
            let payload = message.payload.as_bytes();
            let key = message.key;
            let topic = message.topic;

            let delivery_status = producer
                .send(
                    FutureRecord::to(topic.as_str())
                        .payload(payload)
                        .key(key.as_slice())
                        .headers(rdkafka::message::OwnedHeaders::new()),
                    Duration::from_secs(0),
                )
                .await;
        }
    }
}
