use glium::{Display, DrawParameters, Frame, IndexBuffer, Program, Surface, VertexBuffer};

use crate::shapes::{Normal, Position, TexCoord, cube};

pub const CHUNK_SIZE: usize = 16;

#[derive(Copy, Clone, Debug)]
pub enum Block {
    Air,
    Solid
}

#[derive(Debug)]
pub struct Chunk {
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    blocks: Box<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,

    positions: VertexBuffer<Position>,
    tex_coords: VertexBuffer<TexCoord>,
    normals: VertexBuffer<Normal>,
    indices: IndexBuffer<u32>,
}

impl Chunk {
    pub fn new(display: &Display, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Chunk {
        let mut blocks = Box::new([[[Block::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);

        for square in blocks.iter_mut() {
            for line in square.iter_mut() {
                for block in line.iter_mut() {
                    if rand::random() {
                        *block = Block::Solid;
                    }
                }
            }
        }

        Chunk {
            chunk_x,
            chunk_y,
            chunk_z,
            blocks,
            positions: VertexBuffer::new(display, &cube::POSITIONS).unwrap(),
            tex_coords: VertexBuffer::new(display, &cube::TEX_COORDS).unwrap(),
            normals: VertexBuffer::new(display, &cube::NORMALS).unwrap(),
            indices: IndexBuffer::new(
                display,
                glium::index::PrimitiveType::TrianglesList,
                &cube::INDICES,
            )
            .unwrap(),

        }
    }

    pub fn render(&self, target: &mut Frame, program: &Program, perspective: [[f32; 4]; 4], view: [[f32; 4]; 4], params: &DrawParameters) {
        for (x, square) in self.blocks.iter().enumerate() {
            for (y, line) in square.iter().enumerate() {
                for (z, block) in line.iter().enumerate() {
                    if let Block::Solid = block {
                        let model = [
                            [1.0, 0.0, 0.0, 0.0],
                            [0.0, 1.0, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [(self.chunk_x * CHUNK_SIZE as i32 + x as i32) as f32, (self.chunk_y * CHUNK_SIZE as i32 + y as i32) as f32, (self.chunk_z * CHUNK_SIZE as i32 + z as i32) as f32, 1.0],
                        ];

                        let uniforms = uniform! {
                            model: model,
                            view: view,
                            perspective: perspective,
                            light: [-1.0, 0.4, 0.9f32],
                        };

                        target
                            .draw(
                                (&self.positions, &self.tex_coords, &self.normals),
                                &self.indices,
                                &program,
                                &uniforms,
                                &params,
                            )
                            .unwrap();
                    }
                }
            }
        }
    }
}
