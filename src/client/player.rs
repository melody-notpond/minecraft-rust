use glium::{Display, DrawParameters, Frame, IndexBuffer, Program, Surface, VertexBuffer, index::PrimitiveType};

use super::shapes::{Normal, Position, TexCoord, cube::*};

pub struct Player {
    pub name: String,
    pub position: [f32; 3],
    positions: VertexBuffer<Position>,
    tex_coords: VertexBuffer<TexCoord>,
    normals: VertexBuffer<Normal>,
    indices: IndexBuffer<u32>,
}

impl Player {
    pub fn new(name: String, position: [f32; 3], display: &Display) -> Player {
        Player {
            name,
            position,
            positions: VertexBuffer::new(display, &POSITIONS).unwrap(),
            tex_coords: VertexBuffer::new(display, &TEX_COORDS).unwrap(),
            normals: VertexBuffer::new(display, &NORMALS).unwrap(),
            indices: IndexBuffer::new(display, PrimitiveType::TrianglesList, &INDICES).unwrap(),
        }
    }

    pub fn render(
        &self,
        target: &mut Frame,
        program: &Program,
        perspective: [[f32; 4]; 4],
        view: [[f32; 4]; 4],
        params: &DrawParameters,
    ) {
        let model = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [
                self.position[0],
                self.position[1],
                self.position[2],
                1.0,
            ],
        ];

        let uniforms = uniform! {
            model: model,
            view: view,
            perspective: perspective,
            light: [-1.0, 0.4, 0.9f32],
            colour: [0.0, 0.0, 1.0f32],
        };

        target
            .draw(
                (&self.positions, &self.tex_coords, &self.normals),
                &self.indices,
                program,
                &uniforms,
                params,
            )
            .unwrap();
    }
}
