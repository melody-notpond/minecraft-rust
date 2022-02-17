use std::{sync::RwLock, collections::HashMap, iter::repeat};

use glium::{texture::{RawImage2d, SrgbTexture3d, RawImage3d}, Display};
use image::GenericImageView;
use serde::{Serialize, Deserialize};

use crate::client::chunk::BlockTextures;

pub const CHUNK_SIZE: usize = 16;

lazy_static! {
    static ref BLOCK_DATA_MAP: RwLock<Vec<BlockData>> = RwLock::new(Vec::new());
    static ref ID_BLOCK_MAP: RwLock<HashMap<String, Block>> = RwLock::new(HashMap::new());
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Block(u32);

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum FaceDirection {
    Up = 0,
    Down = 1,
    Front = 2,
    Back = 3,
    Left = 4,
    Right = 5,
}

impl Block {
    pub fn register_defaults() {
        Block::register(String::from("air"), BlockData::new(false, vec![]));
        Block::register(String::from("grass"), BlockData::new(true, vec![
            (String::from("assets/textures/PNG/Tiles/grass_top.png"), vec![FaceDirection::Up]),
            (String::from("assets/textures/PNG/Tiles/dirt_grass.png"), vec![FaceDirection::Front, FaceDirection::Back, FaceDirection::Left, FaceDirection::Right]),
            (String::from("assets/textures/PNG/Tiles/dirt.png"), vec![FaceDirection::Down]),
        ]));
        Block::register(String::from("dirt"), BlockData::new(true, vec![
            (String::from("assets/textures/PNG/Tiles/dirt.png"), vec![FaceDirection::Up, FaceDirection::Down, FaceDirection::Left, FaceDirection::Right, FaceDirection::Front, FaceDirection::Back]),
        ]));
    }

    pub fn generate_atlas(display: &Display) -> BlockTextures {
        let mut textures = vec![RawImage2d::from_raw_rgba(vec![], (0, 0))];

        for data in BLOCK_DATA_MAP.write().unwrap().iter_mut() {
            data.textures = [0; 6];
            for (asset, dirs) in data.texture_assets.iter() {
                let texture = image::open(asset).unwrap();
                let dims = texture.dimensions();
                let texture = RawImage2d::from_raw_rgba_reversed(&texture.into_rgba8(), dims);
                let index = textures.len() as u32;
                textures.push(texture);
                for &dir in dirs {
                    data.textures[dir as usize] = index;
                }
            }
        }

        if let Some(texture) = textures.get(1) {
            let width = texture.width;
            let height = texture.height;
            textures[0] = RawImage2d::from_raw_rgba_reversed(&repeat(0u8).take(width as usize * height as usize * 4).collect::<Vec<_>>(), (width, height));
        }

        let texture_count = textures.len() as u32;
        let textures = RawImage3d::from_vec_raw2d(&textures);
        let textures = SrgbTexture3d::new(display, textures).unwrap();
        BlockTextures {
            textures,
            texture_count,
        }
    }

    pub fn register(name: String, data: BlockData) -> Block {
        let mut lock = BLOCK_DATA_MAP.write().unwrap();
        let index = lock.len() as u32;
        lock.push(data);

        let mut lock = ID_BLOCK_MAP.write().unwrap();
        lock.insert(name, Block(index));
        Block(index)
    }

    pub fn air() -> Block {
        Block(0)
    }

    pub fn get(name: &str) -> Option<Block> {
        let lock = ID_BLOCK_MAP.read().unwrap();
        lock.get(name).cloned()
    }

    pub fn get_texture(&self, face: FaceDirection) -> Option<u32> {
        BLOCK_DATA_MAP.read().unwrap().get(self.0 as usize).map(|v| v.textures[face as usize])
    }

    pub fn is_solid(&self) -> Option<bool> {
        BLOCK_DATA_MAP.read().unwrap().get(self.0 as usize).map(|v| v.solid)
    }
}

pub struct BlockData {
    texture_assets: Vec<(String, Vec<FaceDirection>)>,
    textures: [u32; 6],
    solid: bool,
}

impl BlockData {
    pub fn new(solid: bool, texture_assets: Vec<(String, Vec<FaceDirection>)>) -> BlockData {
        BlockData {
            texture_assets,
            textures: [0; 6],
            solid,
        }
    }
}
