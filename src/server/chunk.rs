use noise::{Perlin, NoiseFn, Seedable};
use serde::{Serialize, Deserialize};

use super::super::blocks::{Block, CHUNK_SIZE};

pub trait ChunkGenerator: Default {
    fn from_seed(seed: u32) -> Self;

    fn generate(&mut self, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Box<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>;
}

#[derive(Default)]
pub struct PerlinChunkGenerator(Perlin);

impl ChunkGenerator for PerlinChunkGenerator {
    fn from_seed(seed: u32) -> Self {
        PerlinChunkGenerator(Perlin::new().set_seed(seed))
    }

    fn generate(&mut self, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Box<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]> {
        let mut blocks = Box::new([[[Block::air(); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);

        let chunk_x = chunk_x as f64;
        let chunk_y = chunk_y as f64;
        let chunk_z = chunk_z as f64;
        for (x, square) in blocks.iter_mut().enumerate() {
            let x = x as f64;
            for (y, line) in square.iter_mut().enumerate() {
                let y = y as f64;
                for (z, block) in line.iter_mut().enumerate() {
                    let z = z as f64;
                    if self.0.get([(chunk_x * 16.0 + x) / 20.0, (chunk_y * 16.0 + y) / 20.0, (chunk_z * 16.0 + z) / 20.0]) > 0.3 {
                        *block = Block::get("dirt").unwrap_or_else(Block::air);
                    } else if self.0.get([(chunk_x * 16.0 + x) / 20.0, (chunk_y * 16.0 + y) / 20.0, (chunk_z * 16.0 + z) / 20.0]) > 0.1 {
                        *block = Block::get("grass").unwrap_or_else(Block::air);
                    }
                }
            }
        }

        blocks
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chunk {
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    blocks: Box<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
}

impl Chunk {
    pub fn new<G>(chunk_x: i32, chunk_y: i32, chunk_z: i32, gen: &mut G) -> Chunk
        where G: ChunkGenerator
    {
        let blocks = gen.generate(chunk_x, chunk_y, chunk_z);

        Chunk {
            chunk_x,
            chunk_y,
            chunk_z,
            blocks,
        }
    }

    pub fn get_chunk_x(&self) -> i32 {
        self.chunk_x
    }

    pub fn get_chunk_y(&self) -> i32 {
        self.chunk_y
    }

    pub fn get_chunk_z(&self) -> i32 {
        self.chunk_z
    }

    pub fn get_blocks(&self) -> &[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] {
        &*self.blocks
    }
}
