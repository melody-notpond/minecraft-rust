use serde::{Deserialize, Serialize};

/// Packet from user to server
#[derive(Serialize, Deserialize)]
pub enum UserPacket {
    ConnectionRequest,
    Ping
}

/// Packet from server to user
#[derive(Serialize, Deserialize)]
pub enum ServerPacket {
    ConnectionAccepted,
    Disconnected,
    Pong,
}

