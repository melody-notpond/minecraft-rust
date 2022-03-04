use std::collections::HashMap;

use glium::index::PrimitiveType;
use glium::texture::SrgbTexture3d;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, UniformBuffer};
use glium::{Display, DrawParameters, Frame, IndexBuffer, Program, Surface, VertexBuffer};

use crate::blocks::FaceDirection;

use super::super::blocks::{Block, CHUNK_SIZE};
use super::super::server::chunk::Chunk as ServerChunk;
use super::light::LightSource;
use super::shapes::frustum::Aabb;
use super::shapes::{Normal, Position, TexCoord};

const LIGHT_COUNT: usize = 5;

const NORM_UP: Normal = Normal {
    normal: [0.0, 1.0, 0.0],
};

const SQUARE_POSITIONS: [Position; 4] = [
    Position {
        position: [-0.25, 0.25, -0.25],
    },
    Position {
        position: [-0.25, 0.25, 0.25],
    },
    Position {
        position: [0.25, 0.25, -0.25],
    },
    Position {
        position: [0.25, 0.25, 0.25],
    },
];

const SQUARE_TEX_COORDS: [TexCoord; 4] = [
    TexCoord {
        tex_coords: [0.0, 0.0],
    },
    TexCoord {
        tex_coords: [0.0, 1.0],
    },
    TexCoord {
        tex_coords: [1.0, 0.0],
    },
    TexCoord {
        tex_coords: [1.0, 1.0],
    },
];

const SQUARE_NORMALS: [Normal; 4] = [NORM_UP, NORM_UP, NORM_UP, NORM_UP];

const SQUARE_INDICES: [u32; 6] = [0, 1, 2, 1, 3, 2];

#[derive(Debug)]
pub struct Mesh {
    positions: VertexBuffer<Position>,
    tex_coords: VertexBuffer<TexCoord>,
    normals: VertexBuffer<Normal>,
    indices: IndexBuffer<u32>,
}

impl Mesh {
    pub fn square(display: &Display) -> Mesh {
        let positions = VertexBuffer::new(display, &SQUARE_POSITIONS).unwrap();
        let tex_coords = VertexBuffer::new(display, &SQUARE_TEX_COORDS).unwrap();
        let normals = VertexBuffer::new(display, &SQUARE_NORMALS).unwrap();
        let indices =
            IndexBuffer::new(display, PrimitiveType::TrianglesList, &SQUARE_INDICES).unwrap();

        Mesh {
            positions,
            tex_coords,
            normals,
            indices,
        }
    }
}

pub struct BlockTextures {
    pub textures: SrgbTexture3d,
    pub texture_count: u32,
}

#[derive(Debug, Copy, Clone)]
struct InstanceData {
    /// 0..2   = FaceDirection
    /// 3..3   = nothing
    /// 4..7   = x
    /// 8..11  = y
    /// 12..15 = z
    /// 32..63 = texture map
    data: (u32, u32),
}

implement_vertex!(InstanceData, data);

impl InstanceData {
    fn new(dir: FaceDirection, block: Block, x: u32, y: u32, z: u32) -> InstanceData {
        let mut data = InstanceData {
            data: (0, block.get_texture(dir).unwrap_or(0)),
        };
        data.set_direction(dir);
        data.set_x(x);
        data.set_y(y);
        data.set_z(z);
        data
    }

    #[allow(unused)]
    fn direction(&self) -> FaceDirection {
        match self.data.0 & 0x000f {
            0 => FaceDirection::Up,
            1 => FaceDirection::Down,
            2 => FaceDirection::Front,
            3 => FaceDirection::Back,
            4 => FaceDirection::Left,
            5 => FaceDirection::Right,

            _ => unreachable!(),
        }
    }

    fn set_direction(&mut self, dir: FaceDirection) {
        self.data.0 = (self.data.0 & !0x000f) | dir as u32;
    }

    fn x(&self) -> u32 {
        (self.data.0 & 0x00f0) >> 4
    }

    fn set_x(&mut self, x: u32) {
        self.data.0 = (self.data.0 & !0x00f0) | (x << 4);
    }

    fn y(&self) -> u32 {
        (self.data.0 & 0x0f00) >> 8
    }

    fn set_y(&mut self, y: u32) {
        self.data.0 = (self.data.0 & !0x0f00) | (y << 8);
    }

    fn z(&self) -> u32 {
        (self.data.0 & 0xf000) >> 12
    }

    fn set_z(&mut self, z: u32) {
        self.data.0 = (self.data.0 & !0xf000) | (z << 12);
    }
}

#[derive(Copy, Clone, Debug)]
struct Selection {
    selected: u32,
}

implement_vertex!(Selection, selected);

#[derive(Copy, Clone, Debug)]
#[repr(C, align(128))]
struct Light {
    colour: u32,
    reserved: [u32; 3],
    position: [f32; 3],
}

implement_uniform_block!(Light, colour, position);

#[derive(Debug)]
pub struct Chunk {
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    blocks: Box<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
    mesh_raw: Option<Vec<InstanceData>>,
    mesh: Option<Box<VertexBuffer<InstanceData>>>,
    selected: Option<Box<VertexBuffer<Selection>>>,
    aabb: Aabb,
    light_buffer: UniformBuffer<[Light; LIGHT_COUNT]>,
    pub loaded: bool,
}

impl Chunk {
    pub fn from_server_chunk(display: &Display, chunk: ServerChunk) -> Chunk {
        Chunk {
            chunk_x: chunk.get_chunk_x(),
            chunk_y: chunk.get_chunk_y(),
            chunk_z: chunk.get_chunk_z(),
            blocks: Box::new(*chunk.get_blocks()),
            mesh_raw: None,
            mesh: None,
            selected: None,
            aabb: Aabb {
                centre: [
                    chunk.get_chunk_x() as f32 * CHUNK_SIZE as f32 * 0.5 + CHUNK_SIZE as f32 * 0.25,
                    chunk.get_chunk_y() as f32 * CHUNK_SIZE as f32 * 0.5 + CHUNK_SIZE as f32 * 0.25,
                    chunk.get_chunk_z() as f32 * CHUNK_SIZE as f32 * 0.5 + CHUNK_SIZE as f32 * 0.25,
                ],
                extents: [CHUNK_SIZE as f32 * 0.25; 3],
            },
            light_buffer: UniformBuffer::new(display, [Light {
                    colour: 0,
                    reserved: [0; 3],
                    position: [0.0; 3],
                }; 5]).unwrap(),
            loaded: true,
        }
    }

    pub fn world_to_chunk_coords(x: f32, y: f32, z: f32) -> (i32, i32, i32, usize, usize, usize) {
        let mut block_coords = [
            (x * 2.0).round() as i32,
            (y * 2.0).round() as i32,
            (z * 2.0).round() as i32,
        ];

        if block_coords[0] < 0 {
            block_coords[0] -= CHUNK_SIZE as i32 - 1;
        }
        if block_coords[1] < 0 {
            block_coords[1] -= CHUNK_SIZE as i32 - 1;
        }
        if block_coords[2] < 0 {
            block_coords[2] -= CHUNK_SIZE as i32 - 1;
        }

        let (chunk_x, chunk_y, chunk_z) = (
            block_coords[0] / CHUNK_SIZE as i32,
            block_coords[1] / CHUNK_SIZE as i32,
            block_coords[2] / CHUNK_SIZE as i32,
        );

        if block_coords[0] < 0 {
            block_coords[0] += CHUNK_SIZE as i32 - 1;
        }
        if block_coords[1] < 0 {
            block_coords[1] += CHUNK_SIZE as i32 - 1;
        }
        if block_coords[2] < 0 {
            block_coords[2] += CHUNK_SIZE as i32 - 1;
        }

        let (mut x, mut y, mut z) = (
            block_coords[0] % CHUNK_SIZE as i32,
            block_coords[1] % CHUNK_SIZE as i32,
            block_coords[2] % CHUNK_SIZE as i32,
        );
        if x < 0 {
            x += CHUNK_SIZE as i32;
        }
        if y < 0 {
            y += CHUNK_SIZE as i32;
        }
        if z < 0 {
            z += CHUNK_SIZE as i32;
        }
        let (x, y, z) = (x as usize, y as usize, z as usize);

        (chunk_x, chunk_y, chunk_z, x, y, z)
    }

    pub fn triangle_count(&self) -> usize {
        if let Some(v) = &self.mesh_raw {
            v.len() * 2
        } else {
            0
        }
    }

    pub fn block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        &mut self.blocks[x][y][z]
    }

    fn get_block<'a>(
        &'a self,
        chunks: &'a HashMap<(i32, i32, i32), ChunkWaiter>,
        x: isize,
        y: isize,
        z: isize,
    ) -> Block {
        if x < 0 {
            chunks
                .get(&(self.chunk_x - 1, self.chunk_y, self.chunk_z))
                .and_then(ChunkWaiter::chunk)
                .map(|v| v.blocks[CHUNK_SIZE - 1][y as usize][z as usize])
                .unwrap_or_else(Block::air)
        } else if x > CHUNK_SIZE as isize - 1 {
            chunks
                .get(&(self.chunk_x + 1, self.chunk_y, self.chunk_z))
                .and_then(ChunkWaiter::chunk)
                .map(|v| v.blocks[0][y as usize][z as usize])
                .unwrap_or_else(Block::air)
        } else if y < 0 {
            chunks
                .get(&(self.chunk_x, self.chunk_y - 1, self.chunk_z))
                .and_then(ChunkWaiter::chunk)
                .map(|v| v.blocks[x as usize][CHUNK_SIZE - 1][z as usize])
                .unwrap_or_else(Block::air)
        } else if y > CHUNK_SIZE as isize - 1 {
            chunks
                .get(&(self.chunk_x, self.chunk_y + 1, self.chunk_z))
                .and_then(ChunkWaiter::chunk)
                .map(|v| v.blocks[x as usize][0][z as usize])
                .unwrap_or_else(Block::air)
        } else if z < 0 {
            chunks
                .get(&(self.chunk_x, self.chunk_y, self.chunk_z - 1))
                .and_then(ChunkWaiter::chunk)
                .map(|v| v.blocks[x as usize][y as usize][CHUNK_SIZE - 1])
                .unwrap_or_else(Block::air)
        } else if z > CHUNK_SIZE as isize - 1 {
            chunks
                .get(&(self.chunk_x, self.chunk_y, self.chunk_z + 1))
                .and_then(ChunkWaiter::chunk)
                .map(|v| v.blocks[x as usize][y as usize][0])
                .unwrap_or_else(Block::air)
        } else {
            self.blocks[x as usize][y as usize][z as usize]
        }
    }

    pub fn generate_mesh(
        &mut self,
        display: &Display,
        chunks: &HashMap<(i32, i32, i32), ChunkWaiter>,
    ) -> bool {
        if self.mesh_raw.is_some() {
            return false;
        }

        let mut instance_data = vec![];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let x = x as isize;
                    let y = y as isize;
                    let z = z as isize;
                    let block = self.get_block(chunks, x, y, z);
                    if block.is_solid().unwrap_or(false) {
                        if self.get_block(chunks, x, y + 1, z) == Block::air() {
                            instance_data.push(InstanceData::new(
                                FaceDirection::Up,
                                block,
                                x as u32,
                                y as u32,
                                z as u32,
                            ));
                        }

                        if self.get_block(chunks, x, y - 1, z) == Block::air() {
                            instance_data.push(InstanceData::new(
                                FaceDirection::Down,
                                block,
                                x as u32,
                                y as u32,
                                z as u32,
                            ));
                        }

                        if self.get_block(chunks, x + 1, y, z) == Block::air() {
                            instance_data.push(InstanceData::new(
                                FaceDirection::Front,
                                block,
                                x as u32,
                                y as u32,
                                z as u32,
                            ));
                        }

                        if self.get_block(chunks, x - 1, y, z) == Block::air() {
                            instance_data.push(InstanceData::new(
                                FaceDirection::Back,
                                block,
                                x as u32,
                                y as u32,
                                z as u32,
                            ));
                        }

                        if self.get_block(chunks, x, y, z + 1) == Block::air() {
                            instance_data.push(InstanceData::new(
                                FaceDirection::Left,
                                block,
                                x as u32,
                                y as u32,
                                z as u32,
                            ));
                        }

                        if self.get_block(chunks, x, y, z - 1) == Block::air() {
                            instance_data.push(InstanceData::new(
                                FaceDirection::Right,
                                block,
                                x as u32,
                                y as u32,
                                z as u32,
                            ));
                        }
                    }
                }
            }
        }

        self.mesh = Some(Box::new(
            VertexBuffer::new(display, &instance_data).unwrap(),
        ));
        self.mesh_raw = Some(instance_data);
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        target: &mut Frame,
        program: &Program,
        perspective: [[f32; 4]; 4],
        view: [[f32; 4]; 4],
        params: &DrawParameters,
        square: &Mesh,
        textures: &BlockTextures,
        lights: &[LightSource],
    ) {
        if let Some(data) = &self.mesh {
            if let Some(selected) = &self.selected {
                let model = [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [
                        (self.chunk_x * CHUNK_SIZE as i32) as f32 * 0.5,
                        (self.chunk_y * CHUNK_SIZE as i32) as f32 * 0.5,
                        (self.chunk_z * CHUNK_SIZE as i32) as f32 * 0.5,
                        1.0,
                    ],
                ];

                let mut new = [Light {
                    colour: 0,
                    reserved: [0; 3],
                    position: [0.0; 3],
                }; LIGHT_COUNT];

                for (new, light) in new.iter_mut().zip(lights) {
                    *new = Light {
                    colour: light.as_uint(),
                    reserved: [0; 3],
                    position: light.location(),
                    };
                }

                self.light_buffer.write(&new);
                let uniforms = uniform! {
                    model: model,
                    view: view,
                    perspective: perspective,
                    Lights: &self.light_buffer,
                    textures: Sampler::new(&textures.textures).minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
                    texture_count: textures.texture_count,
                };

                target
                    .draw(
                        (
                            &square.positions,
                            &square.tex_coords,
                            &square.normals,
                            data.per_instance().unwrap(),
                            selected.per_instance().unwrap(),
                        ),
                        &square.indices,
                        program,
                        &uniforms,
                        params,
                    )
                    .unwrap();
            }
        }
    }

    pub fn invalidate_mesh(&mut self) {
        self.mesh_raw = None;
    }

    pub fn aabb(&self) -> &Aabb {
        &self.aabb
    }

    pub fn select(&mut self, display: &Display, coords: Option<(usize, usize, usize)>) {
        if self.selected.is_some() {
            return;
        }

        let coords = if let Some(coords) = coords {
            coords
        } else {
            (CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE)
        };

        if let Some(mesh) = &self.mesh_raw {
            let mut select_data = vec![];

            for data in mesh {
                let x = data.x() as usize;
                let y = data.y() as usize;
                let z = data.z() as usize;
                select_data.push(Selection {
                    selected: (x == coords.0 && y == coords.1 && z == coords.2) as u32,
                });
            }

            self.selected = Some(Box::new(VertexBuffer::new(display, &select_data).unwrap()));
        }
    }

    pub fn invalidate_selection(&mut self) {
        self.selected = None;
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
