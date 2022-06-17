use serde::{Deserialize, Serialize};

/// Packet from user to server
#[derive(Serialize, Deserialize, Debug)]
pub enum UserPacket {
    JoinRequest {
        username: String,
    },
    Pong,
    Leave,
}

/// Packet from server to user
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket {
    ConnectionAccepted,
    Ping,
    Disconnect {
        reason: String,
    },
}
