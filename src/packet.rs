use serde::{Deserialize, Serialize};

/// Packet from user to server
#[derive(Serialize, Deserialize, Debug)]
pub enum UserPacket {
    ConnectionRequest { name: String },
    Disconnect,
    Ping { timestamp: u128 },
    MoveSelf { pos: [f32; 3] },
}

/// Packet from server to user
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket {
    ConnectionAccepted,
    Disconnected { reason: String, },
    Pong { timestamp: u128 },
}

