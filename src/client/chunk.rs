use std::collections::HashMap;
use std::io::Cursor;

use glium::index::PrimitiveType;
use glium::texture::{SrgbTexture3d, RawImage3d, RawImage2d};
use glium::uniforms::{Sampler, MinifySamplerFilter, MagnifySamplerFilter};
use glium::{Display, DrawParameters, Frame, IndexBuffer, Program, Surface, VertexBuffer};
use image::ImageFormat;

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
    textures: SrgbTexture3d,
    texture_count: u32,
}

impl BlockTextures {
    pub fn generate_textures(display: &Display) -> BlockTextures {
        let mut textures = vec![];

        let texture = image::load(Cursor::new(&include_bytes!("../../assets/textures/PNG/Tiles/dirt_grass.png")), ImageFormat::Png).unwrap().to_rgba8();
        let dims = texture.dimensions();
        let texture = RawImage2d::from_raw_rgba_reversed(&texture.into_raw(), dims);
        textures.push(texture);

        let texture = image::load(Cursor::new(&include_bytes!("../../assets/textures/PNG/Tiles/grass_top.png")), ImageFormat::Png).unwrap().to_rgba8();
        let dims = texture.dimensions();
        let texture = RawImage2d::from_raw_rgba_reversed(&texture.into_raw(), dims);
        textures.push(texture);

        let texture = image::load(Cursor::new(&include_bytes!("../../assets/textures/PNG/Tiles/dirt.png")), ImageFormat::Png).unwrap().to_rgba8();
        let dims = texture.dimensions();
        let texture = RawImage2d::from_raw_rgba_reversed(&texture.into_raw(), dims);
        textures.push(texture);

        let textures = RawImage3d::from_vec_raw2d(&textures);
        let textures = SrgbTexture3d::new(display, textures).unwrap();
        BlockTextures {
            textures,
            texture_count: 3,
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone)]
enum FaceDirection {
    Up = 0,
    Down = 1,
    Front = 2,
    Back = 3,
    Left = 4,
    Right = 5,
}

impl Block {
    fn get_texture(&self, face: FaceDirection) -> u32 {
        match self {
            Block::Air => 0,

            Block::Solid => {
                match face {
                    FaceDirection::Up => 1,
                    FaceDirection::Down => 2,
                    _ => 0,
                }
            }
        }
    }
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
        let mut data = InstanceData { data: (0, block.get_texture(dir)) };
        data.set_direction(dir);
        data.set_x(x);
        data.set_y(y);
        data.set_z(z);
        data
    }

    fn _direction(&self) -> FaceDirection {
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

    fn _x(&self) -> u32 {
        (self.data.0 & 0x00f0) >> 4
    }

    fn set_x(&mut self, x: u32) {
        self.data.0 = (self.data.0 & !0x00f0) | (x << 4);
    }

    fn _y(&self) -> u32 {
        (self.data.0 & 0x0f00) >> 8
    }

    fn set_y(&mut self, y: u32) {
        self.data.0 = (self.data.0 & !0x0f00) | (y << 8);
    }

    fn _z(&self) -> u32 {
        (self.data.0 & 0xf000) >> 12
    }

    fn set_z(&mut self, z: u32) {
        self.data.0 = (self.data.0 & !0xf000) | (z << 12);
    }
}

#[derive(Debug)]
pub struct Chunk {
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    blocks: Box<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
    mesh: Option<Box<VertexBuffer<InstanceData>>>,
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

    pub fn generate_mesh(&mut self, display: &Display, chunks: &HashMap<(i32, i32, i32), ChunkWaiter>) -> bool {
        if self.mesh.is_some() {
            return false;
        }

        let mut instance_data = vec![];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let x = x as isize;
                    let y = y as isize;
                    let z = z as isize;
                    if *self.get_block(chunks, x, y, z) == Block::Solid {
                        if *self.get_block(chunks, x, y + 1, z) == Block::Air {
                            instance_data.push(InstanceData::new(FaceDirection::Up, Block::Solid, x as u32, y as u32, z as u32));
                        }

                        if *self.get_block(chunks, x, y - 1, z) == Block::Air {
                            instance_data.push(InstanceData::new(FaceDirection::Down, Block::Solid, x as u32, y as u32, z as u32));
                        }

                        if *self.get_block(chunks, x + 1, y, z) == Block::Air {
                            instance_data.push(InstanceData::new(FaceDirection::Front, Block::Solid, x as u32, y as u32, z as u32));
                        }

                        if *self.get_block(chunks, x - 1, y, z) == Block::Air {
                            instance_data.push(InstanceData::new(FaceDirection::Back, Block::Solid, x as u32, y as u32, z as u32));
                        }

                        if *self.get_block(chunks, x, y, z + 1) == Block::Air {
                            instance_data.push(InstanceData::new(FaceDirection::Left, Block::Solid, x as u32, y as u32, z as u32));
                        }

                        if *self.get_block(chunks, x, y, z - 1) == Block::Air {
                            instance_data.push(InstanceData::new(FaceDirection::Right, Block::Solid, x as u32, y as u32, z as u32));
                        }
                    }
                }
            }
        }

        self.mesh = Some(Box::new(VertexBuffer::new(display, &instance_data).unwrap()));
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
    ) {
        if let Some(data) = &self.mesh {
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
                light: [-1.0, 0.4, 0.9f32],
                textures: Sampler::new(&textures.textures).minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
                texture_count: textures.texture_count,
            };

            target
                .draw(
                    (&square.positions, &square.tex_coords, &square.normals, data.per_instance().unwrap()),
                    &square.indices,
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
