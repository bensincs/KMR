use serde::{Deserialize, Serialize};
use socket2::{Domain, Protocol, Socket, Type};
use std::{net::{Ipv4Addr, SocketAddrV4}, mem::MaybeUninit, collections::HashMap, sync::{Arc, Mutex}};
use std::time::{SystemTime, UNIX_EPOCH};

use log::info;

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 69);
const LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const MULTICAST_PORT: u16 = 6969;
const RESPONSE_PORT: u16 = 6970;


#[derive(Serialize, Deserialize, Debug)]
struct Heartbeat {
    pub timestamp: SystemTime,
}

pub async fn cast() {
    let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    let multicast_addr = SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT);

    loop {
        let h = Heartbeat { timestamp: SystemTime::now() };
        let bin = bincode::serialize(&h).unwrap();
        socket.send_to(&bin, &multicast_addr.into()).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

pub async fn listen() {
    let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.set_multicast_loop_v4(true).unwrap();
    socket.set_multicast_ttl_v4(1).unwrap();
    let response_socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

    let addr = SocketAddrV4::new(LOCAL_ADDR, MULTICAST_PORT);
    socket.bind(&addr.into()).unwrap();

    socket
        .join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)
        .unwrap();

    let mut buf = [MaybeUninit::uninit(); 1024];

    loop {
        let (_, addr) = socket.recv_from(&mut buf).unwrap();

        let buf = unsafe { std::mem::transmute::<_, [u8; 1024]>(buf) };

        let response_addr = SocketAddrV4::new(addr.as_socket_ipv4().unwrap().ip().clone(), RESPONSE_PORT);
        response_socket.send_to(&buf, &response_addr.into()).unwrap();
    }
}

pub async fn response_listen(neighbors: Arc<Mutex<HashMap<String, u128>>>){
    let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    let addr = SocketAddrV4::new(LOCAL_ADDR, RESPONSE_PORT);
    socket.bind(&addr.into()).unwrap();

    let mut buf = [MaybeUninit::uninit(); 1024];

    loop {
        let (_, addr) = socket.recv_from(&mut buf).unwrap();

        let buf = unsafe { std::mem::transmute::<_, [u8; 1024]>(buf) };

        let heartbeat: Heartbeat = bincode::deserialize(&buf).unwrap();

        let sent_ms = heartbeat.timestamp.duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let in_ms = now_ms - sent_ms;

        neighbors.clone().lock().unwrap().insert(addr.as_socket_ipv4().unwrap().ip().to_string(), in_ms);
    }
}
