use crate::blocks::Block;

#[derive(Debug)]
pub struct Aabb {
    pub centre: [f32; 3],
    pub extents: [f32; 3],
}

pub trait Chunk {
    fn get_block(&self, x: usize, y: usize, z: usize) -> Block;
}

