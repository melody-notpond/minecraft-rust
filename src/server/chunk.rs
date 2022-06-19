use crate::{CHUNK_SIZE, packet::ServerPacket};

pub trait ChunkGenerator: Default {
    fn from_seed(seed: u32) -> Self;

    fn generate(&mut self, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Box<[[[u32; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>;
}

#[derive(Default)]
pub struct RandomChunkGenerator;

impl ChunkGenerator for RandomChunkGenerator {
    fn from_seed(_seed: u32) -> Self {
        RandomChunkGenerator
    }

    fn generate(&mut self, _chunk_x: i32, _chunk_y: i32, _chunk_z: i32) -> Box<[[[u32; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]> {
        let mut blocks = Box::new([[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);
        for square in blocks.iter_mut() {
            for row in square.iter_mut() {
                for block in row.iter_mut() {
                    if rand::random() {
                        *block = 1;
                    }
                }
            }
        }

        blocks
    }
}

pub struct Chunk {
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    blocks: Box<[[[u32; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
}

impl Chunk {
    pub fn new<G>(gen: &mut G, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Chunk
        where G: ChunkGenerator
    {
        Chunk {
            chunk_x,
            chunk_y,
            chunk_z,
            blocks: gen.generate(chunk_x, chunk_y, chunk_z),
        }
    }

    pub fn into_packet(&self) -> ServerPacket {
        ServerPacket::ChunkData {
            x: self.chunk_x,
            y: self.chunk_y,
            z: self.chunk_z,
            blocks: self.blocks.clone(),
        }
    }
}
