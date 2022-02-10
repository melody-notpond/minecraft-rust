use serde::{Serialize, Deserialize};

pub const CHUNK_SIZE: usize = 16;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Block {
    Air,
    Solid,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum FaceDirection {
    Up = 0,
    Down = 1,
    Front = 2,
    Back = 3,
    Left = 4,
    Right = 5,
}

impl Block {
    pub fn get_texture(&self, face: FaceDirection) -> u32 {
        match self {
            Block::Air => 0,

            Block::Solid => {
                match face {
                    FaceDirection::Up => 1,
                    FaceDirection::Down => 2,
                    _ => 0,
                }
            }
        }
    }
}

