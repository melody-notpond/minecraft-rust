use serde::{Deserialize, Serialize};

/// Packet from user to server
#[derive(Serialize, Deserialize, Debug)]
pub enum UserPacket {
}

/// Packet from server to user
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket {
}
