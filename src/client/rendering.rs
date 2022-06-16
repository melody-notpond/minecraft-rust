#[derive(Copy, Clone)]
pub struct Vertex {
    position: (f32, f32, f32),
}

implement_vertex!(Vertex, position);

pub const VERTICES: [Vertex; 4] = [
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

#[derive(Copy, Clone)]
pub struct Normal {
    normal: (f32, f32, f32),
}

implement_vertex!(Normal, normal);

pub const NORMALS: [Normal; 4] = [
    Normal {
        normal: (0.0, 1.0, 0.0),
    },
    Normal {
        normal: (0.0, 1.0, 0.0),
    },
    Normal {
        normal: (0.0, 1.0, 0.0),
    },
    Normal {
        normal: (0.0, 1.0, 0.0),
    },
];

pub const INDICES: [u32; 6] = [
    0, 1, 2,
    2, 3, 0,
];
