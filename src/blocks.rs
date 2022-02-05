use serde::{Serialize, Deserialize};

pub const CHUNK_SIZE: usize = 16;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Block {
    Air,
    Solid,
}

