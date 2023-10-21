use socket2::{Domain, Protocol, Socket, Type};
use std::io::Result;
use std::{
    mem::MaybeUninit,
    net::{Ipv4Addr, SocketAddrV4},
    thread::spawn,
};

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 69);
const LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const MULTICAST_PORT: u16 = 6969;

fn main() -> Result<()> {
    // Spawn a thread to listen for incoming messages
    spawn(move || {
        let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
        socket.set_reuse_address(true).unwrap();
        socket.set_multicast_loop_v4(true).unwrap();
        socket.set_multicast_ttl_v4(1).unwrap();

        let addr = SocketAddrV4::new(LOCAL_ADDR, MULTICAST_PORT);
        socket.bind(&addr.into()).unwrap();

        socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED).unwrap();

        let mut buf = [MaybeUninit::<u8>::uninit(); 10];

        loop {
            let (_, addr) = socket.recv_from(&mut buf).unwrap();

            let msg = unsafe { std::mem::transmute::<[MaybeUninit<u8>; 10], [u8; 10]>(buf) };

            let msg = String::from_utf8_lossy(&msg);

            let address = addr.as_socket().unwrap();

            println!("Received {} from {}", msg, address);
        }
    });

    // Spawn a thread to send a timestamp message every second
    spawn(move || {

        let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

        loop {

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();


            let addr = SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT);

            let buf = format!("{}", timestamp).into_bytes();

            socket.send_to(&buf, &addr.into()).unwrap();

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    println!("Press enter to exit...");

    let mut input = String::new();

    std::io::stdin().read_line(&mut input)?;

    Ok(())
}
