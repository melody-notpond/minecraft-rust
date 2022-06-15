use std::time::Instant;

use glium::{glutin::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder, ContextBuilder}, Display, Surface, VertexBuffer, Program, uniform, IndexBuffer, index::PrimitiveType, DrawParameters, Depth, DepthTest, BackfaceCullingMode};
use minecraft_rust::client::rendering::{Vertex, self};

fn main() {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new();
    let cb = ContextBuilder::new()
        .with_depth_buffer(24);
    let display = Display::new(wb, cb, &event_loop).expect("could not create window");

    let positions = VertexBuffer::new(&display, &rendering::VERTICES).unwrap();
    let normals = VertexBuffer::new(&display, &rendering::NORMALS).unwrap();
    let indices = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &rendering::INDICES).unwrap();

    let vs_source = std::fs::read_to_string("src/client/shaders/vertex.glsl").unwrap();
    let fs_source = std::fs::read_to_string("src/client/shaders/fragment.glsl").unwrap();
    let program = Program::from_source(&display, &vs_source, &fs_source, None).unwrap();

    event_loop.run(move |event, _, control_flow| {
        let next_frame_time = Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);

        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::CloseRequested = event {
                *control_flow = ControlFlow::Exit;
            }
            return;
        }

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 1.0, 0.0, 1.0), 1.0);

        let params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let light = [-1.0, 0.4, 0.9f32];
        let uniforms = uniform! {
            perspective: rendering::perspective_matrix(&target),
            view: rendering::view_matrix(&[2.0, -1.0, 1.0], &[-2.0, 1.0, 1.0], &[0.0, 1.0, 0.0]),
            model: [
                [0.01, 0.0, 0.0, 0.0],
                [0.0, 0.01, 0.0, 0.0],
                [0.0, 0.0, 0.01, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
            ],
            light: light,
        };

        target.draw((&positions, &normals), &indices, &program, &uniforms, &params).unwrap();

        target.finish().unwrap();
    })
}

