use serde::{Deserialize, Serialize};

use crate::server::chunk::Chunk;

/// Packet from user to server
#[derive(Serialize, Deserialize, Debug)]
pub enum UserPacket {
    ConnectionRequest { name: String },
    Disconnect,
    Ping { timestamp: u128 },
    MoveSelf { pos: [f32; 3] },
    RequestChunk { x: i32, y: i32, z: i32 },
}

/// Packet from server to user
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket {
    ConnectionAccepted,
    Disconnected { reason: String, },
    Pong { timestamp: u128 },
    UserJoin { name: String, pos: [f32; 3] },
    UserLeave { name: String },
    MoveUser { name: String, pos: [f32; 3] },
    NewChunk { chunk: Chunk },
}

