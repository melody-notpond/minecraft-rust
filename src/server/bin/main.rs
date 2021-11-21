use std::io;

use tokio::net::UdpSocket;

use minecraft_rust::packet::{ServerPacket, UserPacket};

const BIND: &str = "127.0.0.1:6942";

#[tokio::main]
async fn main() -> io::Result<()> {
    let sock = UdpSocket::bind(BIND).await?;
    let mut buf = Box::new([0; 1024]);

    println!("Server started");

    loop {
        let (len, addr) = sock.recv_from(&mut *buf).await?;
        let packet: UserPacket = bincode::deserialize(&buf[..len]).unwrap();

        match packet {
            UserPacket::ConnectionRequest { name } => {
                let buf = bincode::serialize(&ServerPacket::ConnectionAccepted).unwrap();
                println!("Connection requested from {} at address {}", name, addr);
                sock.send_to(&buf, addr).await?;
            }

            UserPacket::Ping { timestamp } => {
                let buf = bincode::serialize(&ServerPacket::Pong { timestamp }).unwrap();
                println!("Ping! ({})", addr);
                sock.send_to(&buf, addr).await?;
            }
        }
    }
}
