use serde::{Deserialize, Serialize};

/// Packet from user to server
#[derive(Serialize, Deserialize, Debug)]
pub enum UserPacket {
    ConnectionRequest { name: String },
    Ping { timestamp: u128 }
}

/// Packet from server to user
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket {
    ConnectionAccepted,
    Disconnected { reason: String, },
    Pong { timestamp: u128 },
}

