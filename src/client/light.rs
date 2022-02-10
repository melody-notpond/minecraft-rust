use std::collections::HashMap;

use crate::blocks::{FaceDirection, CHUNK_SIZE};

use super::chunk::ChunkWaiter;

pub struct LightSource {
    red: u8,
    green: u8,
    blue: u8,
    intensity: u8,
    location: [f32; 3],
    updated: bool,
}

impl LightSource {
    pub fn new(red: u8, green: u8, blue: u8, intensity: u8, location: [f32; 3]) -> LightSource {
        LightSource {
            red,
            green,
            blue,
            intensity,
            location,
            updated: true,
        }
    }

    pub fn invalidate_chunk_lighting(&self, chunks: &mut HashMap<(i32, i32, i32), ChunkWaiter>) {
        let (x, y, z) = ((self.location[0] * 2.0) as i32, (self.location[1] * 2.0) as i32, (self.location[2] * 2.0) as i32);
        let (x, y, z) = (x / CHUNK_SIZE as i32, y / CHUNK_SIZE as i32, z / CHUNK_SIZE as i32);
        for i in -3..=3 {
            for j in -3..=3 {
                for k in -3..=3 {
                    let (chunk_x, chunk_y, chunk_z) = (x + i, y + j, z + k);
                    let (x, y, z) = (chunk_x as f32 * CHUNK_SIZE as f32 * 0.5, chunk_y as f32 * CHUNK_SIZE as f32 * 0.5, chunk_z as f32 * CHUNK_SIZE as f32 * 0.5);

                    if ((x - self.location[0]).abs() < 15.0 || (x + CHUNK_SIZE as f32 / 2.0 - self.location[0]).abs() < 15.0)
                        && ((y - self.location[1]).abs() < 15.0 || (y + CHUNK_SIZE as f32 / 2.0 - self.location[1]).abs() < 15.0)
                        && ((z - self.location[2]).abs() < 15.0 || (z + CHUNK_SIZE as f32 / 2.0 - self.location[2]).abs() < 15.0)
                    {
                        if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(chunk_x, chunk_y, chunk_z)) {
                            chunk.invalidate_lights();
                        }
                    }
                }
            }
        }
    }

    pub fn calculate_light_intensity(&self, x: i32, y: i32, z: i32, _dir: FaceDirection) -> u32 {
        let dx = self.location[0] - x as f32 * 0.5;
        let dy = self.location[1] - y as f32 * 0.5;
        let dz = self.location[2] - z as f32 * 0.5;
        (self.intensity as f32 - (dx * dx + dy * dy + dz * dz).sqrt()) as u32
    }

    pub fn red(&self) -> u8 {
        self.red
    }

    pub fn green(&self) -> u8 {
        self.green
    }

    pub fn blue(&self) -> u8 {
        self.blue
    }

    pub fn intensity(&self) -> u8 {
        self.intensity
    }

    pub fn location(&self) -> [f32; 3] {
        self.location
    }

    pub fn updated(&self) -> bool {
        self.updated
    }

    pub fn reset_updated(&mut self) {
        self.updated = false;
    }

    pub fn set_red(&mut self, red: u8) {
        self.red = red;
        self.updated = true;
    }

    pub fn set_green(&mut self, green: u8) {
        self.green = green;
        self.updated = true;
    }

    pub fn set_blue(&mut self, blue: u8) {
        self.blue = blue;
        self.updated = true;
    }

    pub fn set_intensity(&mut self, intensity: u8) {
        self.intensity = intensity;
        self.updated = true;
    }

    pub fn set_location(&mut self, location: [f32; 3]) {
        self.location = location;
        self.updated = true;
    }
}
