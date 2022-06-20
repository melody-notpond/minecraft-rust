use std::collections::HashMap;

use glium::{
    index::PrimitiveType, Display, DrawParameters, Frame, IndexBuffer, Program, Surface,
    VertexBuffer,
};

use crate::CHUNK_SIZE;

#[derive(Copy, Clone)]
struct Vertex {
    position: (f32, f32, f32),
}

implement_vertex!(Vertex, position);

const VERTICES: [Vertex; 4] = [
    Vertex {
        position: (-0.125, 0.125, -0.125),
    },
    Vertex {
        position: (-0.125, 0.125, 0.125),
    },
    Vertex {
        position: (0.125, 0.125, 0.125),
    },
    Vertex {
        position: (0.125, 0.125, -0.125),
    },
];

const INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];

pub struct SquareMesh {
    vertices: VertexBuffer<Vertex>,
    indices: IndexBuffer<u32>,
}

impl SquareMesh {
    pub fn init(display: &Display) {
        unsafe {
            if SQUARE_MESH.is_none() {
                SQUARE_MESH = Some(SquareMesh {
                    vertices: VertexBuffer::new(display, &VERTICES).unwrap(),
                    indices: IndexBuffer::new(display, PrimitiveType::TrianglesList, &INDICES).unwrap(),
                });
            }
        }
    }
}

static mut SQUARE_MESH: Option<SquareMesh> = None;

#[derive(Copy, Clone)]
pub struct InstanceData {
    ///   0-3: face direction
    ///   4-7: x
    ///  8-11: y
    /// 12-15: z
    /// 16-31: nothing
    /// 32-63: texture atlas index
    data: (u32, u32),
}

implement_vertex!(InstanceData, data);

impl InstanceData {
    fn new(dir: FaceDirection, x: u32, y: u32, z: u32, texture_index: u32) -> InstanceData {
        let dir = dir as u32;
        InstanceData {
            data: (dir | (x << 4) | (y << 8) | (z << 12), texture_index),
        }
    }
}

#[repr(u32)]
enum FaceDirection {
    Up = 0,
    Down = 1,
    Front = 2,
    Back = 3,
    Left = 4,
    Right = 5,
}

pub struct ChunkMesh {
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    mesh: VertexBuffer<InstanceData>,
}

impl ChunkMesh {
    pub fn new(display: &Display, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> ChunkMesh {
        ChunkMesh {
            chunk_x,
            chunk_y,
            chunk_z,
            mesh: VertexBuffer::new(display, &[]).expect("error creating chunk mesh"),
        }
    }

    pub fn replace_mesh(&mut self, display: &Display, new_mesh: Vec<InstanceData>) {
        self.mesh = VertexBuffer::new(display, &new_mesh).expect("error creating chunk mesh");
    }

    pub fn render(
        &self,
        target: &mut Frame,
        perspective: [[f32; 4]; 4],
        view: [[f32; 4]; 4],
        program: &Program,
        params: &DrawParameters,
    ) {
        let uniforms = uniform! {
            perspective: perspective,
            view: view,
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [self.chunk_x as f32 * 0.25 * CHUNK_SIZE as f32, self.chunk_y as f32 * 0.25 * CHUNK_SIZE as f32, self.chunk_z as f32 * 0.25 * CHUNK_SIZE as f32, 1.0f32],
            ],
            light: [1.0, 1.0, 1.0f32],
        };
        target
            .draw(
                (
                    unsafe { &SQUARE_MESH.as_ref().unwrap().vertices },
                    self.mesh.per_instance().unwrap(),
                ),
                unsafe { &SQUARE_MESH.as_ref().unwrap().indices },
                program,
                &uniforms,
                params,
            )
            .unwrap();
    }

    pub fn chunk_x(&self) -> i32 {
        self.chunk_x
    }

    pub fn chunk_y(&self) -> i32 {
        self.chunk_y
    }

    pub fn chunk_z(&self) -> i32 {
        self.chunk_z
    }
}

pub struct Chunk {
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    blocks: Box<[[[u32; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
}

impl Chunk {
    pub fn from_data(chunk_x: i32, chunk_y: i32, chunk_z: i32, blocks: Box<[[[u32; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>) -> Chunk {
            Chunk {
                chunk_x,
                chunk_y,
                chunk_z,
                blocks,
            }
    }

    fn get_block(
        &self,
        chunks: &HashMap<(i32, i32, i32), Chunk>,
        x: isize,
        y: isize,
        z: isize,
    ) -> u32 {
        if x < 0 {
            chunks
                .get(&(self.chunk_x - 1, self.chunk_y, self.chunk_z))
                .map(|v| v.blocks[CHUNK_SIZE - 1][y as usize][z as usize])
                .unwrap_or(0)
        } else if x > CHUNK_SIZE as isize - 1 {
            chunks
                .get(&(self.chunk_x + 1, self.chunk_y, self.chunk_z))
                .map(|v| v.blocks[0][y as usize][z as usize])
                .unwrap_or(0)
        } else if y < 0 {
            chunks
                .get(&(self.chunk_x, self.chunk_y - 1, self.chunk_z))
                .map(|v| v.blocks[x as usize][CHUNK_SIZE - 1][z as usize])
                .unwrap_or(0)
        } else if y > CHUNK_SIZE as isize - 1 {
            chunks
                .get(&(self.chunk_x, self.chunk_y + 1, self.chunk_z))
                .map(|v| v.blocks[x as usize][0][z as usize])
                .unwrap_or(0)
        } else if z < 0 {
            chunks
                .get(&(self.chunk_x, self.chunk_y, self.chunk_z - 1))
                .map(|v| v.blocks[x as usize][y as usize][CHUNK_SIZE - 1])
                .unwrap_or(0)
        } else if z > CHUNK_SIZE as isize - 1 {
            chunks
                .get(&(self.chunk_x, self.chunk_y, self.chunk_z + 1))
                .map(|v| v.blocks[x as usize][y as usize][0])
                .unwrap_or(0)
        } else {
            self.blocks[x as usize][y as usize][z as usize]
        }
    }

    pub fn generate_mesh(&self, chunks: &HashMap<(i32, i32, i32), Chunk>) -> Vec<InstanceData> {
        let mut data = vec![];

        for (x, square) in self.blocks.iter().enumerate() {
            let x = x as isize;
            for (y, row) in square.iter().enumerate() {
                let y = y as isize;
                for (z, block) in row.iter().enumerate() {
                    let z = z as isize;
                    if *block != 0 {
                        if self.get_block(chunks, x, y + 1, z) == 0 {
                            data.push(InstanceData::new(FaceDirection::Up, x as u32, y as u32, z as u32, 0));
                        }
                        if self.get_block(chunks, x, y - 1, z) == 0 {
                            data.push(InstanceData::new(FaceDirection::Down, x as u32, y as u32, z as u32, 0));
                        }
                        if self.get_block(chunks, x - 1, y, z) == 0 {
                            data.push(InstanceData::new(FaceDirection::Right, x as u32, y as u32, z as u32, 0));
                        }
                        if self.get_block(chunks, x + 1, y, z) == 0 {
                            data.push(InstanceData::new(FaceDirection::Left, x as u32, y as u32, z as u32, 0));
                        }
                        if self.get_block(chunks, x, y, z - 1) == 0 {
                            data.push(InstanceData::new(FaceDirection::Front, x as u32, y as u32, z as u32, 0));
                        }
                        if self.get_block(chunks, x, y, z + 1) == 0 {
                            data.push(InstanceData::new(FaceDirection::Back, x as u32, y as u32, z as u32, 0));
                        }
                    }
                }
            }
        }

        data
    }

    pub fn chunk_x(&self) -> i32 {
        self.chunk_x
    }

    pub fn chunk_y(&self) -> i32 {
        self.chunk_y
    }

    pub fn chunk_z(&self) -> i32 {
        self.chunk_z
    }
}

