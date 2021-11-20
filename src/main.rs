#[macro_use]
extern crate glium;

use std::time::{Instant, Duration};

use glium::glutin::dpi::PhysicalPosition;
use glium::{Display, Program, Surface};
use glium::glutin::{event::{Event, StartCause, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder, ContextBuilder};

use minecraft_rust::camera::Camera;
use minecraft_rust::shapes::{Normal, Position, TexCoord};
use tobj::{LoadOptions, Mesh};

const VERTEX_SHADER: &str = include_str!("shaders/vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("shaders/fragment.glsl");

fn main() {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new();
    let cb = ContextBuilder::new()
        .with_depth_buffer(24);
    let display = Display::new(wb, cb, &event_loop).unwrap();

    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        window.set_cursor_grab(true).unwrap();
        let size = window.inner_size();
        let centre = PhysicalPosition::new(size.width / 2, size.height / 2);
        window.set_cursor_position(centre).unwrap();
        window.set_cursor_visible(false);
    }

    let program = Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None).unwrap();

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        //backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        ..Default::default()
    };

    let Mesh { positions, normals, texcoords, indices, .. } = tobj::load_obj("cube.obj", &LoadOptions {
        single_index: true,
        triangulate: true,
        ..Default::default()
    }).unwrap().0.swap_remove(0).mesh;

    let positions = glium::VertexBuffer::new(&display, &positions.chunks(3).map(|v| Position { position: [v[0], v[1], v[2]] }).collect::<Vec<_>>()).unwrap();
    let tex_coords = glium::VertexBuffer::new(&display, &texcoords.chunks(2).map(|v| TexCoord { tex_coords: [v[0], v[1]] }).collect::<Vec<_>>()).unwrap();
    let normals = glium::VertexBuffer::new(&display, &normals.chunks(3).map(|v| Normal { normal: [v[0], v[1], v[2]] }).collect::<Vec<_>>()).unwrap();

    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap();

    let mut camera = Camera::new(1.0, 0.01, 100.0);
    let mut last = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }

                    WindowEvent::KeyboardInput { input, .. } if camera.move_self(input) => (),

                    WindowEvent::CursorMoved { position, .. } => {
                        let gl_window = display.gl_window();
                        let window = gl_window.window();
                        let size = window.inner_size();
                        let centre = PhysicalPosition::new(size.width / 2, size.height / 2);
                        camera.turn_self(position.x as i32 - centre.x as i32, position.y as i32 - centre.y as i32);
                        window.set_cursor_position(centre).unwrap();
                    }

                    _ => (),
                }

                return;
            }

            Event::NewEvents(cause) => {
                match cause {
                    StartCause::Init => {
                        last = Instant::now();
                        let next_frame_time = last + Duration::from_nanos(16_666_667);
                        *control_flow = ControlFlow::WaitUntil(next_frame_time);
                        return;
                    }

                    StartCause::ResumeTimeReached { .. }=> (),

                    _ => return,
                }
            }

            _ => return,
        }

        let delta = Instant::now() - last;
        camera.tick(delta);

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 1.0, 0.0, 1.0), 1.0);

        let perspective = camera.perspective(&target);

        let model = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 1.5, 1.0f32],
        ];
        let view = camera.view_matrix();
        let uniforms = uniform! {
            model: model,
            view: view,
            perspective: perspective,
            light: [-1.0, 0.4, 0.9f32],
        };

        target.draw((&positions, &tex_coords, &normals), &indices, &program, &uniforms, &params).unwrap();
        target.finish().unwrap();

        last = Instant::now();
        let next_frame_time = last + Duration::from_nanos(16_666_667);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
    });
}
