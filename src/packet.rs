use serde::{Deserialize, Serialize};

use crate::CHUNK_SIZE;

pub const MAX_PACKET_SIZE: usize = 2usize.pow(20);

/// Packet from user to server
#[derive(Serialize, Deserialize, Debug)]
pub enum UserPacket {
    JoinRequest { username: String },
    Ping,
    Leave,

    ChunkRequest { x: i32, y: i32, z: i32 },
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

    ChunkData {
        x: i32,
        y: i32,
        z: i32,
        blocks: Box<[[[u32; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
    },
}
