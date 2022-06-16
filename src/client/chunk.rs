use glium::{VertexBuffer, Display, IndexBuffer, index::PrimitiveType, Frame, Surface, Program, DrawParameters};

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

const INDICES: [u32; 6] = [
    0, 1, 2,
    2, 3, 0,
];

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

#[repr(u32)]
enum FaceDirection {
    Up = 0,
    Down = 1,
    Front = 2,
    Back = 3,
    Left = 4,
    Right = 5,
}

impl InstanceData {
    fn new(dir: FaceDirection, x: u32, y: u32, z: u32, texture_index: u32) -> InstanceData {
        let dir = dir as u32;
        InstanceData {
            data: (dir | (x << 4) | (y << 8) | (z << 12), texture_index)
        }
    }
}

pub struct Cube {
    mesh: SquareMesh,
    instance_data: VertexBuffer<InstanceData>,
}

impl Cube {
    pub fn new(display: &Display, x: u32, y: u32, z: u32) -> Cube {
        let data = vec![
            InstanceData::new(FaceDirection::Up, x, y, z, 0),
            InstanceData::new(FaceDirection::Down, x, y, z, 0),
            InstanceData::new(FaceDirection::Left, x, y, z, 0),
            InstanceData::new(FaceDirection::Right, x, y, z, 0),
            InstanceData::new(FaceDirection::Front, x, y, z, 0),
            InstanceData::new(FaceDirection::Back, x, y, z, 0),
        ];
        Cube {
            mesh: SquareMesh::new(display),
            instance_data: VertexBuffer::new(display, &data).unwrap(),
        }
    }

    pub fn render(&self, target: &mut Frame, perspective: [[f32; 4]; 4], view: [[f32; 4]; 4], program: &Program, params: &DrawParameters) {
        let uniforms = uniform! {
            perspective: perspective,
            view: view,
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
            ],
            light: [1.0, 1.0, 1.0f32],
        };
        target.draw((&self.mesh.vertices, self.instance_data.per_instance().unwrap()), &self.mesh.indices, program, &uniforms, params).unwrap();
    }
}

implement_vertex!(InstanceData, data);

