use serde::{Deserialize, Serialize};

pub const MAX_PACKET_SIZE: usize = 1024;

/// Packet from user to server
#[derive(Serialize, Deserialize, Debug)]
pub enum UserPacket {
    JoinRequest {
        username: String,
    },
    Ping,
    Leave,
}

/// Packet from server to user
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket {
    ConnectionAccepted,
    Pong,
    PlayerJoined {
        username: String,
    },
    PlayerLeft {
        username: String,
    },
    Disconnect {
        reason: String,
    },
}
