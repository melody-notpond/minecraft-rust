use glium::{Display, DrawParameters, Frame, IndexBuffer, Program, Surface, VertexBuffer, index::PrimitiveType};
use tobj::LoadOptions;

use crate::collision::Aabb;

use super::shapes::{Normal, Position, TexCoord};

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
        let model = tobj::load_obj("assets/models/player.obj", &LoadOptions {
            single_index: true,
            triangulate: true,
            ignore_points: true,
            ignore_lines: true,
        }).unwrap();

        let mut positions = vec![];
        let mut tex_coords = vec![];
        let mut normals = vec![];

        for i in (0..model.0[0].mesh.positions.len()).step_by(3) {
            positions.push(Position {
                position: [model.0[0].mesh.positions[i], model.0[0].mesh.positions[i + 1], model.0[0].mesh.positions[i + 2]],
            });
        }

        for i in (0..model.0[0].mesh.texcoords.len()).step_by(2) {
            tex_coords.push(TexCoord {
                tex_coords: [model.0[0].mesh.texcoords[i], model.0[0].mesh.texcoords[i + 1]],
            });
        }

        for i in (0..model.0[0].mesh.normals.len()).step_by(3) {
            normals.push(Normal {
                normal: [model.0[0].mesh.normals[i], model.0[0].mesh.normals[i + 1], model.0[0].mesh.normals[i + 2]],
            });
        }

        Player {
            name,
            position,
            positions: VertexBuffer::new(display, &positions).unwrap(),
            tex_coords: VertexBuffer::new(display, &tex_coords).unwrap(),
            normals: VertexBuffer::new(display, &normals).unwrap(),
            indices: IndexBuffer::new(display, PrimitiveType::TrianglesList, &model.0[0].mesh.indices).unwrap(),
        }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb {
            centre: [self.position[0], self.position[1] + 1.0, self.position[2]],
            extents: [0.5, 1.0, 0.5],
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
