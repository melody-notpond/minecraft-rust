use glium::{
    index::PrimitiveType, Display, DrawParameters, Frame, IndexBuffer, Program, Surface,
    VertexBuffer,
};

#[derive(Copy, Clone)]
struct Vertex {
    position: (f32, f32, f32),
}

implement_vertex!(Vertex, position);

const VERTICES: [Vertex; 4] = [
    Vertex {
        position: (-0.25, 0.25, -0.25),
    },
    Vertex {
        position: (-0.25, 0.25, 0.25),
    },
    Vertex {
        position: (0.25, 0.25, 0.25),
    },
    Vertex {
        position: (0.25, 0.25, -0.25),
    },
];

const INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];

pub struct SquareMesh {
    vertices: VertexBuffer<Vertex>,
    indices: IndexBuffer<u32>,
}

impl SquareMesh {
    pub fn new(display: &Display) -> SquareMesh {
        SquareMesh {
            vertices: VertexBuffer::new(display, &VERTICES).unwrap(),
            indices: IndexBuffer::new(display, PrimitiveType::TrianglesList, &INDICES).unwrap(),
        }
    }
}

#[derive(Copy, Clone)]
struct InstanceData {
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

const CHUNK_SIZE: usize = 16;

pub struct Chunk {
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    blocks: Box<[[[bool; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
    mesh: SquareMesh,
    data: Option<VertexBuffer<InstanceData>>,
}

impl Chunk {
    pub fn new(display: &Display, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Chunk {
        let mesh = SquareMesh::new(display);
        let mut blocks = Box::new([[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);
        for square in blocks.iter_mut() {
            for row in square.iter_mut() {
                for block in row.iter_mut() {
                    *block = rand::random();
                }
            }
        }

        let mut chunk = Chunk {
            chunk_x,
            chunk_y,
            chunk_z,
            blocks,
            mesh,
            data: None,
        };

        chunk.generate_mesh(display);

        chunk
    }

    pub fn generate_mesh(&mut self, display: &Display) {
        self.data = None;
        let mut data = vec![];

        for (x, square) in self.blocks.iter().enumerate() {
            for (y, row) in square.iter().enumerate() {
                for (z, block) in row.iter().enumerate() {
                    if *block {
                        if y >= CHUNK_SIZE - 1 || !self.blocks[x][y + 1][z] {
                            data.push(InstanceData::new(FaceDirection::Up, x as u32, y as u32, z as u32, 0));
                        }
                        if y == 0 || !self.blocks[x][y - 1][z] {
                            data.push(InstanceData::new(FaceDirection::Down, x as u32, y as u32, z as u32, 0));
                        }
                        if x == 0 || !self.blocks[x - 1][y][z] {
                            data.push(InstanceData::new(FaceDirection::Right, x as u32, y as u32, z as u32, 0));
                        }
                        if x >= CHUNK_SIZE - 1 || !self.blocks[x + 1][y][z] {
                            data.push(InstanceData::new(FaceDirection::Left, x as u32, y as u32, z as u32, 0));
                        }
                        if z == 0 || !self.blocks[x][y][z - 1] {
                            data.push(InstanceData::new(FaceDirection::Front, x as u32, y as u32, z as u32, 0));
                        }
                        if z >= CHUNK_SIZE - 1 || !self.blocks[x][y][z + 1] {
                            data.push(InstanceData::new(FaceDirection::Back, x as u32, y as u32, z as u32, 0));
                        }
                    }
                }
            }
        }

        self.data = Some(VertexBuffer::new(display, &data).unwrap());
    }

    pub fn render(
        &self,
        target: &mut Frame,
        perspective: [[f32; 4]; 4],
        view: [[f32; 4]; 4],
        program: &Program,
        params: &DrawParameters,
    ) {
        if let Some(data) = &self.data {
            let uniforms = uniform! {
                perspective: perspective,
                view: view,
                model: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [self.chunk_x as f32 * 0.5, self.chunk_y as f32 * 0.5, self.chunk_z as f32 * 0.5, 1.0f32],
                ],
                light: [1.0, 1.0, 1.0f32],
            };
            target
                .draw(
                    (
                        &self.mesh.vertices,
                        data.per_instance().unwrap(),
                    ),
                    &self.mesh.indices,
                    program,
                    &uniforms,
                    params,
                )
                .unwrap();
        }
    }
}
