use super::{Normal, Position, TexCoord};

pub const POSITIONS: [Position; 24] = [
    Position {
        position: [1.0, -1.0, 1.0],
    },
    Position {
        position: [-1.0, -1.0, 1.0],
    },
    Position {
        position: [-1.0, -1.0, -1.0],
    },
    Position {
        position: [-1.0, 1.0, -1.0],
    },
    Position {
        position: [-1.0, 1.0, 1.0],
    },
    Position {
        position: [0.999999, 1.0, 1.000001],
    },
    Position {
        position: [1.0, 1.0, -0.999999],
    },
    Position {
        position: [0.999999, 1.0, 1.000001],
    },
    Position {
        position: [1.0, -1.0, 1.0],
    },
    Position {
        position: [0.999999, 1.0, 1.000001],
    },
    Position {
        position: [-1.0, 1.0, 1.0],
    },
    Position {
        position: [-1.0, -1.0, 1.0],
    },
    Position {
        position: [-1.0, -1.0, 1.0],
    },
    Position {
        position: [-1.0, 1.0, 1.0],
    },
    Position {
        position: [-1.0, 1.0, -1.0],
    },
    Position {
        position: [1.0, -1.0, -1.0],
    },
    Position {
        position: [-1.0, -1.0, -1.0],
    },
    Position {
        position: [-1.0, 1.0, -1.0],
    },
    Position {
        position: [1.0, -1.0, -1.0],
    },
    Position {
        position: [1.0, 1.0, -0.999999],
    },
    Position {
        position: [1.0, -1.0, -1.0],
    },
    Position {
        position: [1.0, -1.0, 1.0],
    },
    Position {
        position: [-1.0, -1.0, -1.0],
    },
    Position {
        position: [1.0, 1.0, -0.999999],
    },
];

pub const TEX_COORDS: [TexCoord; 24] = [
    TexCoord {
        tex_coords: [1.0, 0.333333],
    },
    TexCoord {
        tex_coords: [1.0, 0.666667],
    },
    TexCoord {
        tex_coords: [0.666667, 0.666667],
    },
    TexCoord {
        tex_coords: [1.0, 0.333333],
    },
    TexCoord {
        tex_coords: [0.666667, 0.333333],
    },
    TexCoord {
        tex_coords: [0.666667, 0.0],
    },
    TexCoord {
        tex_coords: [0.0, 0.333333],
    },
    TexCoord {
        tex_coords: [0.0, 0.0],
    },
    TexCoord {
        tex_coords: [0.333333, 0.0],
    },
    TexCoord {
        tex_coords: [0.333333, 0.0],
    },
    TexCoord {
        tex_coords: [0.666667, 0.0],
    },
    TexCoord {
        tex_coords: [0.666667, 0.333333],
    },
    TexCoord {
        tex_coords: [0.333333, 1.0],
    },
    TexCoord {
        tex_coords: [0.0, 1.0],
    },
    TexCoord {
        tex_coords: [0.0, 0.666667],
    },
    TexCoord {
        tex_coords: [0.333333, 0.333333],
    },
    TexCoord {
        tex_coords: [0.333333, 0.666667],
    },
    TexCoord {
        tex_coords: [0.0, 0.666667],
    },
    TexCoord {
        tex_coords: [0.666667, 0.333333],
    },
    TexCoord {
        tex_coords: [1.0, 0.0],
    },
    TexCoord {
        tex_coords: [0.333333, 0.333333],
    },
    TexCoord {
        tex_coords: [0.333333, 0.333333],
    },
    TexCoord {
        tex_coords: [0.333333, 0.666667],
    },
    TexCoord {
        tex_coords: [0.0, 0.333333],
    },
];

pub const NORMALS: [Normal; 24] = [
    Normal {
        normal: [0.0, -1.0, 0.0],
    },
    Normal {
        normal: [0.0, -1.0, 0.0],
    },
    Normal {
        normal: [0.0, -1.0, 0.0],
    },
    Normal {
        normal: [0.0, 1.0, 0.0],
    },
    Normal {
        normal: [0.0, 1.0, 0.0],
    },
    Normal {
        normal: [0.0, 1.0, 0.0],
    },
    Normal {
        normal: [1.0, 0.0, 0.0],
    },
    Normal {
        normal: [1.0, 0.0, 0.0],
    },
    Normal {
        normal: [1.0, 0.0, 0.0],
    },
    Normal {
        normal: [-0.0, 0.0, 1.0],
    },
    Normal {
        normal: [-0.0, 0.0, 1.0],
    },
    Normal {
        normal: [-0.0, 0.0, 1.0],
    },
    Normal {
        normal: [-1.0, -0.0, -0.0],
    },
    Normal {
        normal: [-1.0, -0.0, -0.0],
    },
    Normal {
        normal: [-1.0, -0.0, -0.0],
    },
    Normal {
        normal: [0.0, 0.0, -1.0],
    },
    Normal {
        normal: [0.0, 0.0, -1.0],
    },
    Normal {
        normal: [0.0, 0.0, -1.0],
    },
    Normal {
        normal: [0.0, -1.0, 0.0],
    },
    Normal {
        normal: [0.0, 1.0, 0.0],
    },
    Normal {
        normal: [1.0, 0.0, 0.0],
    },
    Normal {
        normal: [-0.0, 0.0, 1.0],
    },
    Normal {
        normal: [-1.0, -0.0, -0.0],
    },
    Normal {
        normal: [0.0, 0.0, -1.0],
    },
];

pub const INDICES: [u32; 36] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 0, 2, 19, 3, 5, 20, 6, 8, 21,
    9, 11, 22, 12, 14, 23, 15, 17,
];
