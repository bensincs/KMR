use crate::SETTINGS;
use lazy_static::lazy_static;
use log::{info, warn};
use rdkafka::{
    config::{ClientConfig, RDKafkaLogLevel},
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::{Header, Headers, Message, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
};
use serde::{Deserialize, Serialize};
use socket2::{Domain, Protocol, Socket, Type};
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    mem::MaybeUninit,
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration, hash::{Hash, Hasher}, sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;

lazy_static! {
    static ref MULTICAST_PORT: u16 = SETTINGS.kafka.multicast_port;
    static ref RESPONSE_PORT: u16 = SETTINGS.kafka.response_port;
    static ref LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
    static ref BROKERS: String = SETTINGS.kafka.brokers.clone();
    static ref GROUP_ID: String = SETTINGS.kafka.group_id.clone();
    static ref ORIGIN_ID: String = SETTINGS.kafka.origin_id.clone();
    static ref ORIGIN_HEADER_NAME: String = "kmr_origin".to_string();
    static ref RULES: HashMap<String, Ipv4Addr> = SETTINGS
        .kafka
        .rules
        .iter()
        .map(|r| (r.topic.clone(), r.multicast_addr.clone()))
        .collect::<HashMap<String, Ipv4Addr>>();
}

#[derive(Serialize, Deserialize, Debug, Hash)]
struct MessageWrapper {
    pub payload: String,
    pub key: Option<Vec<u8>>,
    pub topic: String,
    pub origin: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageAck {
    pub hash: u64,
}

pub fn start_tasks() -> Vec<JoinHandle<()>> {
    let mut multicast_addresses: Vec<Ipv4Addr> = vec![];
    let read_reciepts: Arc<Mutex<HashMap<u64, Option<Vec<Ipv4Addr>>>>> = Arc::new(Mutex::new(HashMap::new()));

    for (_, addr) in RULES.iter() {
        multicast_addresses.push(addr.clone());
    }

    let tasks = vec![
        tokio::spawn(consume_and_cast(RULES.clone(), read_reciepts.clone())),
        tokio::spawn(receive_produce_and_respond(multicast_addresses)),
        tokio::spawn(response_listen(read_reciepts.clone())),
        tokio::spawn(read_reciept_print(read_reciepts.clone())),
    ];

    return tasks;
}

async fn consume_and_cast(mappings: HashMap<String, Ipv4Addr>, read_reciepts: Arc<Mutex<HashMap<u64, Option<Vec<Ipv4Addr>>>>>) {
    let topics = mappings.keys().map(|s| s.as_str()).collect::<Vec<&str>>();

    warn!("{:?}", BROKERS.clone());

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", GROUP_ID.clone())
        .set("bootstrap.servers", BROKERS.clone())
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
                let headers = m.headers();

                let default_headers = OwnedHeaders::new();
                let headers = match headers {
                    None => {
                        warn!("No headers found");
                        default_headers.as_borrowed()
                    }
                    Some(headers) => headers,
                };

                let mut forward = true;
                for header in headers.iter() {
                    if header.key == ORIGIN_HEADER_NAME.as_str() {
                        forward = false;
                    }
                }

                if !forward {
                    info!("Message already forwarded");
                    continue;
                }

                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        warn!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };

                let multicast_addr = mappings.get(m.topic()).unwrap();
                let sock_addr = SocketAddrV4::new(*multicast_addr, *MULTICAST_PORT);

                let key = match m.key_view::<[u8]>() {
                    None => None,
                    Some(Ok(k)) => Some(k.to_vec()),
                    Some(Err(e)) => {
                        warn!("Error while deserializing message key: {:?}", e);
                        None
                    }
                };

                let message = MessageWrapper {
                    payload: payload.to_string(),
                    key: key,
                    topic: m.topic().to_string(),
                    origin: ORIGIN_ID.to_string(),
                };

                let mut default_hasher = DefaultHasher::new();
                message.hash(&mut default_hasher);

                add_read_reciept(default_hasher.finish(), None, read_reciepts.clone());

                info!("Sending to: {:?}", sock_addr);

                let message = bincode::serialize(&message).unwrap();

                socket.send_to(&message, &sock_addr.into()).unwrap();

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}

async fn receive_produce_and_respond(multicast_addresses: Vec<Ipv4Addr>) {
    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", BROKERS.clone())
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

    let addr = SocketAddrV4::new(*LOCAL_ADDR, *MULTICAST_PORT);
    socket.bind(&addr.into()).unwrap();
    let response_socket: Socket =
    Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

    let mut buf = [MaybeUninit::uninit(); 1024];

    loop {
        let (_, addr) = socket.recv_from(&mut buf).unwrap();

        let buf = unsafe { std::mem::transmute::<_, [u8; 1024]>(buf) };

        let message: MessageWrapper = bincode::deserialize(&buf).unwrap();

        info!("Received message: {:?}", message);

        let mut default_hasher = DefaultHasher::new();
        message.hash(&mut default_hasher);

        let hash = default_hasher.finish();

        if message.origin != *ORIGIN_ID {
            let payload = message.payload.as_bytes();
            let key = match message.key {
                None => Vec::new(),
                Some(k) => k,
            };
            let topic = message.topic.clone();

            let header = Header {
                key: ORIGIN_HEADER_NAME.as_str(),
                value: Some(&message.origin),
            };

            let headers = OwnedHeaders::new().insert(header);

            let record = FutureRecord::to(topic.as_str())
                .payload(payload)
                .key(key.as_slice())
                .headers(headers);

            let _ = producer.send(record, Duration::from_secs(0)).await;
        }

        let response_addr =
            SocketAddrV4::new(addr.as_socket_ipv4().unwrap().ip().clone(), *RESPONSE_PORT);


        let message_ack = MessageAck { hash: hash };

        let message_ack = bincode::serialize(&message_ack).unwrap();

        response_socket.send_to(&message_ack, &response_addr.into()).unwrap();

        info!("Message produced");
    }
}


pub async fn response_listen(read_reciepts: Arc<Mutex<HashMap<u64, Option<Vec<Ipv4Addr>>>>>) {
    let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    let addr = SocketAddrV4::new(*LOCAL_ADDR, *RESPONSE_PORT);
    socket.bind(&addr.into()).unwrap();

    let mut buf = [MaybeUninit::uninit(); 1024];

    loop {
        let (_, addr) = socket.recv_from(&mut buf).unwrap();

        let buf = unsafe { std::mem::transmute::<_, [u8; 1024]>(buf) };

        let ack: MessageAck = bincode::deserialize(&buf).unwrap();

        add_read_reciept(ack.hash, Some(addr.as_socket_ipv4().unwrap().ip().to_owned()), read_reciepts.clone())
    }
}


// Utils

fn add_read_reciept(hash: u64, addr: Option<Ipv4Addr>, read_reciepts: Arc<Mutex<HashMap<u64, Option<Vec<Ipv4Addr>>>>>) {
    let mut read_reciepts = read_reciepts.lock().unwrap();
    let mut reciepts = match read_reciepts.get(&hash) {
        None => vec![],
        Some(r) => match r {
            None => vec![],
            Some(r) => r.clone(),
        },
    };

    match addr {
        None => {}
        Some(addr) => {
            reciepts.push(addr);
        }
    }

    read_reciepts.insert(hash, Some(reciepts));
}


async fn read_reciept_print(read_reciepts: Arc<Mutex<HashMap<u64, Option<Vec<Ipv4Addr>>>>>) {
    loop {
        println!("{:?}", read_reciepts.lock().unwrap());
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}