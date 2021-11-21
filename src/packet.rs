use serde::{Deserialize, Serialize};

/// Packet from user to server
#[derive(Serialize, Deserialize, Debug)]
pub enum UserPacket {
    ConnectionRequest { name: String },
    Ping
}

/// Packet from server to user
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket {
    ConnectionAccepted,
    Disconnected { reason: String, },
    Pong,
}

