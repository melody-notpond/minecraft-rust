pub mod frustum;

#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub position: [f32; 3],
}

implement_vertex!(Position, position);

#[derive(Copy, Clone, Debug)]
pub struct TexCoord {
    pub tex_coords: [f32; 2],
}

implement_vertex!(TexCoord, tex_coords);

#[derive(Copy, Clone, Debug)]
pub struct Normal {
    pub normal: [f32; 3],
}

implement_vertex!(Normal, normal);
