use std::net::{ToSocketAddrs, UdpSocket};

use crate::packet::{ServerPacket, UserPacket, MAX_PACKET_SIZE};

pub mod camera;
pub mod chunk;

pub struct NetworkClient {
    name: String,
    socket: UdpSocket,
}

impl NetworkClient {
    pub fn new<A>(name: &str, addr: A) -> std::io::Result<NetworkClient>
    where
        A: ToSocketAddrs,
    {
        Ok(NetworkClient {
            name: String::from(name),
            socket: UdpSocket::bind(addr)?,
        })
    }

    pub fn connect_to_server<A>(&self, addr: A) -> std::io::Result<()>
    where
        A: ToSocketAddrs,
    {
        self.socket.connect(addr)?;
        self.send_packet(UserPacket::JoinRequest {
            username: self.name.clone(),
        })
    }

    pub fn send_packet(&self, packet: UserPacket) -> std::io::Result<()> {
        let packet = bincode::serialize(&packet).expect("could not serialise packet");

        if packet.len() > MAX_PACKET_SIZE {
            todo!(
                "figure out what to do with packets of size > MAX_PACKET_SIZE ({MAX_PACKET_SIZE})"
            );
        }

        self.socket.send(&packet)?;

        Ok(())
    }

    pub fn recv_packet(&self) -> std::io::Result<ServerPacket> {
        let mut data = Box::new([0; MAX_PACKET_SIZE]);
        let length = self.socket.recv(&mut *data)?;
        let packet = bincode::deserialize(&data[..length]).expect("could not deserialise packet");
        Ok(packet)
    }
}
