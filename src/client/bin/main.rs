use std::time::{Instant, Duration};

use glium::{glutin::{event::{Event, WindowEvent, StartCause, VirtualKeyCode, ElementState}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder, ContextBuilder, dpi::PhysicalPosition}, Display, Surface, VertexBuffer, Program, uniform, IndexBuffer, index::PrimitiveType, DrawParameters, Depth, DepthTest, BackfaceCullingMode, PolygonMode};
use minecraft_rust::client::{rendering, camera::Camera};

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

    let mut params = DrawParameters {
        depth: Depth {
            test: DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        backface_culling: BackfaceCullingMode::CullClockwise,
        ..Default::default()
    };

    let mut camera = Camera::new(2.0, 0.01, 110.0);
    let mut locked;

    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        locked = window.set_cursor_grab(true).is_ok();

        if locked {
            let size = window.inner_size();
            let centre = PhysicalPosition::new(size.width / 2, size.height / 2);
            window.set_cursor_position(centre).unwrap();
            window.set_cursor_visible(false);
        }
    }

    let mut frame_count = 0;
    let mut last = Instant::now();
    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }

                    WindowEvent::Focused(false) => {
                        let gl_window = display.gl_window();
                        let window = gl_window.window();
                        window.set_cursor_visible(true);
                        window.set_cursor_grab(false).unwrap();
                        locked = false;
                    }

                    WindowEvent::KeyboardInput { input, .. }
                        if matches!(input.virtual_keycode, Some(VirtualKeyCode::Escape))
                            && matches!(input.state, ElementState::Released) =>
                    {
                        let gl_window = display.gl_window();
                        let window = gl_window.window();
                        let size = window.inner_size();
                        let centre = PhysicalPosition::new(size.width / 2, size.height / 2);
                        window.set_cursor_position(centre).unwrap();
                        window.set_cursor_visible(locked);
                        locked ^= true;
                        window.set_cursor_grab(locked).unwrap();
                    }

                    WindowEvent::KeyboardInput { input, .. }
                        if locked && camera.move_self(input) => {},

                    WindowEvent::KeyboardInput { input, .. } if locked => {
                        if let Some(VirtualKeyCode::Semicolon) = input.virtual_keycode {
                            if input.state == ElementState::Released {
                                match params.polygon_mode {
                                    PolygonMode::Point => params.polygon_mode = PolygonMode::Line,
                                    PolygonMode::Line => params.polygon_mode = PolygonMode::Fill,
                                    PolygonMode::Fill => params.polygon_mode = PolygonMode::Point,
                                }
                            }
                        }
                    }

                    WindowEvent::CursorMoved { position, .. } if locked => {
                        let gl_window = display.gl_window();
                        let window = gl_window.window();
                        let size = window.inner_size();
                        let centre = PhysicalPosition::new(size.width / 2, size.height / 2);
                        camera.turn_self(
                            position.x as i32 - centre.x as i32,
                            position.y as i32 - centre.y as i32,
                        );
                        window.set_cursor_position(centre).unwrap();
                    }
                    _ => (),
                }
                return;
            }

            Event::NewEvents(cause) => match cause {
                StartCause::Init => {
                    last = Instant::now();
                    last_frame = last;
                    let next_frame_time = last + Duration::from_nanos(5);
                    *control_flow = ControlFlow::WaitUntil(next_frame_time);
                    return;
                }

                StartCause::ResumeTimeReached { .. } => (),

                _ => return,
            },

            _ => return,
        }

        frame_count += 1;
        if last - last_frame >= Duration::from_secs(1) {
            println!("{} frames per second", frame_count);
            frame_count = 0;
            last_frame = last;
        }

        let delta = Instant::now() - last;
        last = Instant::now();

        camera.tick(delta);

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 1.0, 0.0, 1.0), 1.0);

        let light = [-1.0, 0.4, 0.9f32];
        let uniforms = uniform! {
            perspective: camera.perspective(&target),
            view: camera.view_matrix(),
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

