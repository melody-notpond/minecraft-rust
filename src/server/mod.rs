use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use crate::packet::{ServerPacket, UserPacket, MAX_PACKET_SIZE};

pub mod chunk;

pub struct NetworkServer {
    socket: UdpSocket,
}

impl NetworkServer {
    pub fn new<A>(addr: A) -> std::io::Result<NetworkServer>
    where
        A: ToSocketAddrs,
    {
        Ok(NetworkServer {
            socket: UdpSocket::bind(addr)?,
        })
    }

    pub fn send_packet<A>(&self, packet: ServerPacket, addr: A) -> std::io::Result<()>
    where
        A: ToSocketAddrs,
    {
        let packet = bincode::serialize(&packet).expect("could not serialise packet");

        if packet.len() > MAX_PACKET_SIZE {
            todo!(
                "figure out what to do with packets of size > MAX_PACKET_SIZE ({MAX_PACKET_SIZE})"
            );
        }

        self.socket.send_to(&packet, addr)?;

        Ok(())
    }

    pub fn recv_packet(&self) -> std::io::Result<(UserPacket, SocketAddr)> {
        let mut data = Box::new([0; MAX_PACKET_SIZE]);
        let (length, addr) = self.socket.recv_from(&mut *data)?;
        let packet = bincode::deserialize(&data[..length]).expect("could not deserialise packet");
        Ok((packet, addr))
    }
}
