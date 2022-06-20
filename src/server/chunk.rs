use noise::{NoiseFn, Perlin, Seedable};

use crate::{CHUNK_SIZE, packet::ServerPacket};

pub trait ChunkGenerator {
    fn from_seed(seed: u32) -> Self;

    fn generate(&mut self, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Box<[[[u32; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>;
}
pub struct PerlinChunkGenerator(Perlin);

impl ChunkGenerator for PerlinChunkGenerator {
    fn from_seed(seed: u32) -> Self {
        PerlinChunkGenerator(Perlin::new().set_seed(seed))
    }

    fn generate(
        &mut self,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
    ) -> Box<[[[u32; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]> {
        let mut blocks = Box::new([[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);

        let chunk_x = chunk_x as f64;
        let chunk_y = chunk_y as f64;
        let chunk_z = chunk_z as f64;
        for (x, square) in blocks.iter_mut().enumerate() {
            let x = x as f64;
            for (y, line) in square.iter_mut().enumerate() {
                let y = y as f64;
                for (z, block) in line.iter_mut().enumerate() {
                    let z = z as f64;
                    let coords = [
                        (chunk_x * CHUNK_SIZE as f64 + x) / 40.0,
                        (chunk_y * CHUNK_SIZE as f64 + y) / 40.0,
                        (chunk_z * CHUNK_SIZE as f64 + z) / 40.0,
                    ];
                    let height = self.0.get(coords);

                    if height >= 0.5 {
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
