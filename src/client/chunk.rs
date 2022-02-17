use std::collections::HashMap;
use std::io::Cursor;

use glium::index::PrimitiveType;
use glium::texture::{SrgbTexture3d, RawImage3d, RawImage2d};
use glium::uniforms::{Sampler, MinifySamplerFilter, MagnifySamplerFilter};
use glium::{Display, DrawParameters, Frame, IndexBuffer, Program, Surface, VertexBuffer};
use image::ImageFormat;

use crate::blocks::FaceDirection;

use super::light::LightSource;
use super::shapes::frustum::Aabb;
use super::shapes::{Normal, Position, TexCoord};
use super::super::blocks::{Block, CHUNK_SIZE};
use super::super::server::chunk::Chunk as ServerChunk;

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

const SQUARE_NORMALS: [Normal; 4] = [
    NORM_UP,
    NORM_UP,
    NORM_UP,
    NORM_UP,
];

const SQUARE_INDICES: [u32; 6] = [
    0, 2, 1,
    1, 2, 3,
];

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
        let indices = IndexBuffer::new(
            display,
            PrimitiveType::TrianglesList,
            &SQUARE_INDICES,
        )
        .unwrap();

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
        let mut data = InstanceData { data: (0, block.get_texture(dir).unwrap_or(0)) };
        data.set_direction(dir);
        data.set_x(x);
        data.set_y(y);
        data.set_z(z);
        data
    }

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
struct Light {
    light: u32,
}

implement_vertex!(Light, light);

impl Light {
    fn new(red: u32, green: u32, blue: u32, intensity: u32) -> Light {
        Light {
            light: ((red & 0xf) << 12) | ((green & 0xf) << 8) | ((blue & 0xf) << 4) | (intensity & 0xf)
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Selection {
    selected: u32,
}

implement_vertex!(Selection, selected);

#[derive(Debug)]
pub struct Chunk {
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    blocks: Box<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
    mesh_raw: Option<Vec<InstanceData>>,
    mesh: Option<Box<VertexBuffer<InstanceData>>>,
    lights: Option<Box<VertexBuffer<Light>>>,
    selected: Option<Box<VertexBuffer<Selection>>>,
    aabb: Aabb,
}

impl Chunk {
    pub fn from_server_chunk(chunk: ServerChunk) -> Chunk {
        Chunk {
            chunk_x: chunk.get_chunk_x(),
            chunk_y: chunk.get_chunk_y(),
            chunk_z: chunk.get_chunk_z(),
            blocks: Box::new(*chunk.get_blocks()),
            mesh_raw: None,
            mesh: None,
            lights: None,
            selected: None,
            aabb: Aabb {
                centre: [chunk.get_chunk_x() as f32 * CHUNK_SIZE as f32 * 0.5 + CHUNK_SIZE as f32 * 0.25, chunk.get_chunk_y() as f32 *  CHUNK_SIZE as f32 * 0.5 + CHUNK_SIZE as f32 * 0.25, chunk.get_chunk_z() as f32 * CHUNK_SIZE as f32 * 0.5 + CHUNK_SIZE as f32 * 0.25],
                extents: [CHUNK_SIZE as f32 * 0.25; 3],
            }

        }
    }

    pub fn block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        &mut self.blocks[x][y][z]
    }

    fn get_block<'a>(&'a self, chunks: &'a HashMap<(i32, i32, i32), ChunkWaiter>, x: isize, y: isize, z: isize) -> Block {
        if x < 0 {
            chunks.get(&(self.chunk_x - 1, self.chunk_y, self.chunk_z)).and_then(ChunkWaiter::chunk).map(|v| v.blocks[CHUNK_SIZE - 1][y as usize][z as usize]).unwrap_or_else(Block::air)
        } else if x > CHUNK_SIZE as isize - 1 {
            chunks.get(&(self.chunk_x + 1, self.chunk_y, self.chunk_z)).and_then(ChunkWaiter::chunk).map(|v| v.blocks[0][y as usize][z as usize]).unwrap_or_else(Block::air) } else if y < 0 {
            chunks.get(&(self.chunk_x, self.chunk_y - 1, self.chunk_z)).and_then(ChunkWaiter::chunk).map(|v| v.blocks[x as usize][CHUNK_SIZE - 1][z as usize]).unwrap_or_else(Block::air)
        } else if y > CHUNK_SIZE as isize - 1 {
            chunks.get(&(self.chunk_x, self.chunk_y + 1, self.chunk_z)).and_then(ChunkWaiter::chunk).map(|v| v.blocks[x as usize][0][z as usize]).unwrap_or_else(Block::air)
        } else if z < 0 {
            chunks.get(&(self.chunk_x, self.chunk_y, self.chunk_z - 1)).and_then(ChunkWaiter::chunk).map(|v| v.blocks[x as usize][y as usize][CHUNK_SIZE - 1]).unwrap_or_else(Block::air)
        } else if z > CHUNK_SIZE as isize - 1 {
            chunks.get(&(self.chunk_x, self.chunk_y, self.chunk_z + 1)).and_then(ChunkWaiter::chunk).map(|v| v.blocks[x as usize][y as usize][0]).unwrap_or_else(Block::air)
        } else {
            self.blocks[x as usize][y as usize][z as usize]
        }
    }

    pub fn generate_mesh(&mut self, display: &Display, chunks: &HashMap<(i32, i32, i32), ChunkWaiter>) -> bool {
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
                            instance_data.push(InstanceData::new(FaceDirection::Up, block, x as u32, y as u32, z as u32));
                        }

                        if self.get_block(chunks, x, y - 1, z) == Block::air() {
                            instance_data.push(InstanceData::new(FaceDirection::Down, block, x as u32, y as u32, z as u32));
                        }

                        if self.get_block(chunks, x + 1, y, z) == Block::air() {
                            instance_data.push(InstanceData::new(FaceDirection::Front, block, x as u32, y as u32, z as u32));
                        }

                        if self.get_block(chunks, x - 1, y, z) == Block::air() {
                            instance_data.push(InstanceData::new(FaceDirection::Back, block, x as u32, y as u32, z as u32));
                        }

                        if self.get_block(chunks, x, y, z + 1) == Block::air() {
                            instance_data.push(InstanceData::new(FaceDirection::Left, block, x as u32, y as u32, z as u32));
                        }

                        if self.get_block(chunks, x, y, z - 1) == Block::air() {
                            instance_data.push(InstanceData::new(FaceDirection::Right, block, x as u32, y as u32, z as u32));
                        }
                    }
                }
            }
        }

        self.mesh = Some(Box::new(VertexBuffer::new(display, &instance_data).unwrap()));
        self.mesh_raw = Some(instance_data);
        true
    }

    pub fn populate_lights(&mut self, display: &Display, lights: &[LightSource]) {
        if self.lights.is_some() {
            return;
        }

        if let Some(mesh) = &self.mesh_raw {
            let mut light_data = vec![];

            for data in mesh {
                let x = data.x() as i32;
                let y = data.y() as i32;
                let z = data.z() as i32;
                let dir = data.direction();

                let mut red = 0;
                let mut green = 0;
                let mut blue = 0;
                let mut intensity = 0;

                for light in lights {
                    red += light.red();
                    green += light.green();
                    blue += light.blue();
                    intensity += light.calculate_light_intensity(self.chunk_x * CHUNK_SIZE as i32 + x, self.chunk_y * CHUNK_SIZE as i32 + y, self.chunk_z * CHUNK_SIZE as i32 + z, dir);
                }

                light_data.push(Light::new(red as u32 & 0xf, green as u32 & 0xf, blue as u32 & 0xf, intensity & 0xf));
            }

            self.lights = Some(Box::new(VertexBuffer::new(display, &light_data).unwrap()));
        }
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
    ) {
        if let Some(data) = &self.mesh {
            if let Some(lights) = &self.lights {
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

                    let uniforms = uniform! {
                        model: model,
                        view: view,
                        perspective: perspective,
                        textures: Sampler::new(&textures.textures).minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
                        texture_count: textures.texture_count,
                    };

                    target
                        .draw(
                            (&square.positions, &square.tex_coords, &square.normals, data.per_instance().unwrap(), lights.per_instance().unwrap(), selected.per_instance().unwrap()),
                            &square.indices,
                            program,
                            &uniforms,
                            params,
                        ).unwrap();
                }
            }
        }
    }

    pub fn invalidate_mesh(&mut self) {
        self.mesh_raw = None;
    }

    pub fn invalidate_lights(&mut self) {
        self.lights = None;
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
