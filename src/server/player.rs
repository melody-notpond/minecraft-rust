use std::net::SocketAddr;

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub addr: SocketAddr,
    pub position: [f32; 3],
}
