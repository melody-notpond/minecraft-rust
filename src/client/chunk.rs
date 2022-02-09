use std::collections::HashMap;

use glium::{Display, DrawParameters, Frame, IndexBuffer, Program, Surface, VertexBuffer};

use super::shapes::{Normal, Position, TexCoord};
use super::super::blocks::{Block, CHUNK_SIZE};
use super::super::server::chunk::Chunk as ServerChunk;

const TEX_COORDS_EMPTY: TexCoord = TexCoord {
    tex_coords: [0.0, 0.0],
};

const NORM_UP: Normal = Normal {
    normal: [0.0, 1.0, 0.0],
};

const NORM_DOWN: Normal = Normal {
    normal: [0.0, -1.0, 0.0],
};

const NORM_FRONT: Normal = Normal {
    normal: [1.0, 0.0, 0.0],
};

const NORM_BACK: Normal = Normal {
    normal: [-1.0, 0.0, 0.0],
};

const NORM_LEFT: Normal = Normal {
    normal: [0.0, 0.0, 1.0],
};

const NORM_RIGHT: Normal = Normal {
    normal: [0.0, 0.0, -1.0],
};

#[derive(Debug)]
pub struct Mesh {
    positions: VertexBuffer<Position>,
    tex_coords: VertexBuffer<TexCoord>,
    normals: VertexBuffer<Normal>,
    indices: IndexBuffer<u32>,
}

#[derive(Debug)]
pub struct Chunk {
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    blocks: Box<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,

    mesh: Option<Box<Mesh>>,
}

impl Chunk {
    pub fn from_server_chunk(chunk: ServerChunk) -> Chunk {
        Chunk {
            chunk_x: chunk.get_chunk_x(),
            chunk_y: chunk.get_chunk_y(),
            chunk_z: chunk.get_chunk_z(),
            blocks: Box::new(*chunk.get_blocks()),
            mesh: None,
        }
    }

    fn get_block<'a>(&'a self, chunks: &'a HashMap<(i32, i32, i32), ChunkWaiter>, x: isize, y: isize, z: isize) -> &'a Block {
        if x < 0 {
            chunks.get(&(self.chunk_x - 1, self.chunk_y, self.chunk_z)).and_then(ChunkWaiter::chunk).map(|v| &v.blocks[CHUNK_SIZE - 1][y as usize][z as usize]).unwrap_or(&Block::Air)
        } else if x > CHUNK_SIZE as isize - 1 {
            chunks.get(&(self.chunk_x + 1, self.chunk_y, self.chunk_z)).and_then(ChunkWaiter::chunk).map(|v| &v.blocks[0][y as usize][z as usize]).unwrap_or(&Block::Air) } else if y < 0 {
            chunks.get(&(self.chunk_x, self.chunk_y - 1, self.chunk_z)).and_then(ChunkWaiter::chunk).map(|v| &v.blocks[x as usize][CHUNK_SIZE - 1][z as usize]).unwrap_or(&Block::Air)
        } else if y > CHUNK_SIZE as isize - 1 {
            chunks.get(&(self.chunk_x, self.chunk_y + 1, self.chunk_z)).and_then(ChunkWaiter::chunk).map(|v| &v.blocks[x as usize][0][z as usize]).unwrap_or(&Block::Air)
        } else if z < 0 {
            chunks.get(&(self.chunk_x, self.chunk_y, self.chunk_z - 1)).and_then(ChunkWaiter::chunk).map(|v| &v.blocks[x as usize][y as usize][CHUNK_SIZE - 1]).unwrap_or(&Block::Air)
        } else if z > CHUNK_SIZE as isize - 1 {
            chunks.get(&(self.chunk_x, self.chunk_y, self.chunk_z + 1)).and_then(ChunkWaiter::chunk).map(|v| &v.blocks[x as usize][y as usize][0]).unwrap_or(&Block::Air)
        } else {
            &self.blocks[x as usize][y as usize][z as usize]
        }
    }

    pub fn generate_mesh(&mut self, display: &Display, chunks: &HashMap<(i32, i32, i32), ChunkWaiter>) {
        if self.mesh.is_some() {
            return;
        }

        let mut positions = vec![];
        let mut tex_coords = vec![];
        let mut normals = vec![];
        let mut indices = vec![];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let x = x as isize;
                    let y = y as isize;
                    let z = z as isize;
                    if *self.get_block(chunks, x, y, z) == Block::Solid {
                        if *self.get_block(chunks, x, y + 1, z) == Block::Air {
                            let x = x as f32 * 0.5;
                            let y = y as f32 * 0.5;
                            let z = z as f32 * 0.5;

                            let i = positions.len() as u32;
                            positions.push(Position {
                                position: [x as f32, y as f32 + 0.5, z as f32],
                            });
                            positions.push(Position {
                                position: [x as f32, y as f32 + 0.5, z as f32 + 0.5],
                            });
                            positions.push(Position {
                                position: [x as f32 + 0.5, y as f32 + 0.5, z as f32],
                            });
                            positions.push(Position {
                                position: [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5],
                            });

                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);

                            normals.push(NORM_UP);
                            normals.push(NORM_UP);
                            normals.push(NORM_UP);
                            normals.push(NORM_UP);

                            indices.push(i);
                            indices.push(i + 2);
                            indices.push(i + 1);
                            indices.push(i + 1);
                            indices.push(i + 2);
                            indices.push(i + 3);
                        }

                        if *self.get_block(chunks, x, y - 1, z) == Block::Air {
                            let x = x as f32 * 0.5;
                            let y = y as f32 * 0.5;
                            let z = z as f32 * 0.5;

                            let i = positions.len() as u32;
                            positions.push(Position {
                                position: [x as f32, y as f32, z as f32],
                            });
                            positions.push(Position {
                                position: [x as f32, y as f32, z as f32 + 0.5],
                            });
                            positions.push(Position {
                                position: [x as f32 + 0.5, y as f32, z as f32],
                            });
                            positions.push(Position {
                                position: [x as f32 + 0.5, y as f32, z as f32 + 0.5],
                            });

                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);

                            normals.push(NORM_DOWN);
                            normals.push(NORM_DOWN);
                            normals.push(NORM_DOWN);
                            normals.push(NORM_DOWN);

                            indices.push(i);
                            indices.push(i + 1);
                            indices.push(i + 2);
                            indices.push(i + 1);
                            indices.push(i + 3);
                            indices.push(i + 2);
                        }

                        if *self.get_block(chunks, x + 1, y, z) == Block::Air {
                            let x = x as f32 * 0.5;
                            let y = y as f32 * 0.5;
                            let z = z as f32 * 0.5;

                            let i = positions.len() as u32;
                            positions.push(Position {
                                position: [x as f32 + 0.5, y as f32, z as f32],
                            });
                            positions.push(Position {
                                position: [x as f32 + 0.5, y as f32, z as f32 + 0.5],
                            });
                            positions.push(Position {
                                position: [x as f32 + 0.5, y as f32 + 0.5, z as f32],
                            });
                            positions.push(Position {
                                position: [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5],
                            });

                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);

                            normals.push(NORM_FRONT);
                            normals.push(NORM_FRONT);
                            normals.push(NORM_FRONT);
                            normals.push(NORM_FRONT);

                            indices.push(i);
                            indices.push(i + 1);
                            indices.push(i + 2);
                            indices.push(i + 1);
                            indices.push(i + 3);
                            indices.push(i + 2);
                        }

                        if *self.get_block(chunks, x - 1, y, z) == Block::Air {
                            let x = x as f32 * 0.5;
                            let y = y as f32 * 0.5;
                            let z = z as f32 * 0.5;

                            let i = positions.len() as u32;
                            positions.push(Position {
                                position: [x as f32, y as f32, z as f32],
                            });
                            positions.push(Position {
                                position: [x as f32, y as f32, z as f32 + 0.5],
                            });
                            positions.push(Position {
                                position: [x as f32, y as f32 + 0.5, z as f32],
                            });
                            positions.push(Position {
                                position: [x as f32, y as f32 + 0.5, z as f32 + 0.5],
                            });

                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);

                            normals.push(NORM_BACK);
                            normals.push(NORM_BACK);
                            normals.push(NORM_BACK);
                            normals.push(NORM_BACK);

                            indices.push(i);
                            indices.push(i + 2);
                            indices.push(i + 1);
                            indices.push(i + 1);
                            indices.push(i + 2);
                            indices.push(i + 3);
                        }

                        if *self.get_block(chunks, x, y, z + 1) == Block::Air {
                            let x = x as f32 * 0.5;
                            let y = y as f32 * 0.5;
                            let z = z as f32 * 0.5;

                            let i = positions.len() as u32;
                            positions.push(Position {
                                position: [x as f32, y as f32, z as f32 + 0.5],
                            });
                            positions.push(Position {
                                position: [x as f32, y as f32 + 0.5, z as f32 + 0.5],
                            });
                            positions.push(Position {
                                position: [x as f32 + 0.5, y as f32, z as f32 + 0.5],
                            });
                            positions.push(Position {
                                position: [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5],
                            });

                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);

                            normals.push(NORM_LEFT);
                            normals.push(NORM_LEFT);
                            normals.push(NORM_LEFT);
                            normals.push(NORM_LEFT);

                            indices.push(i);
                            indices.push(i + 1);
                            indices.push(i + 2);
                            indices.push(i + 1);
                            indices.push(i + 3);
                            indices.push(i + 2);
                        }

                        if *self.get_block(chunks, x, y, z - 1) == Block::Air {
                            let x = x as f32 * 0.5;
                            let y = y as f32 * 0.5;
                            let z = z as f32 * 0.5;

                            let i = positions.len() as u32;
                            positions.push(Position {
                                position: [x as f32, y as f32, z as f32],
                            });
                            positions.push(Position {
                                position: [x as f32, y as f32 + 0.5, z as f32],
                            });
                            positions.push(Position {
                                position: [x as f32 + 0.5, y as f32, z as f32],
                            });
                            positions.push(Position {
                                position: [x as f32 + 0.5, y as f32 + 0.5, z as f32],
                            });

                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);
                            tex_coords.push(TEX_COORDS_EMPTY);

                            normals.push(NORM_RIGHT);
                            normals.push(NORM_RIGHT);
                            normals.push(NORM_RIGHT);
                            normals.push(NORM_RIGHT);

                            indices.push(i);
                            indices.push(i + 2);
                            indices.push(i + 1);
                            indices.push(i + 1);
                            indices.push(i + 2);
                            indices.push(i + 3);
                        }
                    }
                }
            }
        }

        let positions = VertexBuffer::new(display, &positions).unwrap();
        let tex_coords = VertexBuffer::new(display, &tex_coords).unwrap();
        let normals = VertexBuffer::new(display, &normals).unwrap();
        let indices = IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &indices,
        )
        .unwrap();

        self.mesh = Some(Box::new(Mesh {
            positions,
            tex_coords,
            normals,
            indices,
        }));
    }

    pub fn render(
        &self,
        target: &mut Frame,
        program: &Program,
        perspective: [[f32; 4]; 4],
        view: [[f32; 4]; 4],
        params: &DrawParameters,
    ) {
        if let Some(mesh) = &self.mesh {
            let model = [
                [2.0, 0.0, 0.0, 0.0],
                [0.0, 2.0, 0.0, 0.0],
                [0.0, 0.0, 2.0, 0.0],
                [
                    (self.chunk_x * CHUNK_SIZE as i32) as f32,
                    (self.chunk_y * CHUNK_SIZE as i32) as f32,
                    (self.chunk_z * CHUNK_SIZE as i32) as f32,
                    1.0,
                ],
            ];

            let uniforms = uniform! {
                model: model,
                view: view,
                perspective: perspective,
                light: [-1.0, 0.4, 0.9f32],
                colour: [1.0, 0.0, 0.0f32],
            };

            target
                .draw(
                    (&mesh.positions, &mesh.tex_coords, &mesh.normals),
                    &mesh.indices,
                    program,
                    &uniforms,
                    params,
                )
                .unwrap();
        }
    }

    pub fn invalidate_mesh(&mut self) {
        self.mesh = None;
    }
}

pub enum ChunkWaiter {
    Timestamp(u128),
    Chunk(Chunk),
}

impl ChunkWaiter {
    pub fn chunk(&self) -> Option<&Chunk> {
        match self {
            ChunkWaiter::Timestamp(_) => None,
            ChunkWaiter::Chunk(chunk) => Some(chunk),
        }
    }

    pub fn timestamp(&self) -> Option<u128> {
        match self {
            ChunkWaiter::Timestamp(ts) => Some(*ts),
            ChunkWaiter::Chunk(_) => None,
        }
    }
}
