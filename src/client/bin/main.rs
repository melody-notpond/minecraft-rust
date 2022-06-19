use std::{time::{Duration, Instant}, sync::{mpsc, Arc}};

use glium::{
    glutin::{
        dpi::PhysicalPosition,
        event::{ElementState, Event, StartCause, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    BackfaceCullingMode, Depth, DepthTest, Display, DrawParameters,
    PolygonMode, Program, Surface,
};
use minecraft_rust::{client::{camera::Camera, chunk::Chunk, NetworkClient}, packet::{UserPacket, ServerPacket}};

fn main() {
    let addr = "0.0.0.0:6942";
    let client = NetworkClient::new("uwu", addr).expect("could not start client");
    let server = "127.0.0.1:6429";
    client.connect_to_server(server).expect("could not connect to server");
    let (tx, rx) = mpsc::channel();
    let client = Arc::new(client);
    let c = client.clone();
    std::thread::spawn(move || transmitting_thread(c, rx));
    let (tx2, rx) = mpsc::channel();
    std::thread::spawn(move || receiving_thread(client, tx2));

    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new();
    let cb = ContextBuilder::new().with_depth_buffer(24);
    let display = Display::new(wb, cb, &event_loop).expect("could not create window");

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

    let mut camera = Camera::new(2.0, 0.005, 90.0);
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

    let mut chunk = Chunk::new(&display, 0, 0, 0);
    tx.send(UserPacket::ChunkRequest { x: 0, y: 0, z: 0 }).expect("temporary");

    let mut frame_count = 0;
    let mut last = Instant::now();
    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        tx.send(UserPacket::Leave).expect("must be open");
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
                        if locked && camera.move_self(input) => {}

                    WindowEvent::KeyboardInput { input, .. } if locked => {
                        if let Some(VirtualKeyCode::Semicolon) = input.virtual_keycode {
                            if input.state == ElementState::Released {
                                match params.polygon_mode {
                                    PolygonMode::Line => params.polygon_mode = PolygonMode::Fill,
                                    PolygonMode::Fill => params.polygon_mode = PolygonMode::Line,
                                    _ => params.polygon_mode = PolygonMode::Fill,
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
            tx.send(UserPacket::Ping).expect("must be open");
            frame_count = 0;
            last_frame = last;
        }

        let delta = Instant::now() - last;
        last = Instant::now();

        while let Ok(packet) = rx.try_recv() {
            match packet {
                ServerPacket::ConnectionAccepted => println!("connection accepted"),
                ServerPacket::Pong => println!("ping"),
                ServerPacket::PlayerJoined { username } => println!("player {username} joined"),
                ServerPacket::PlayerLeft { username } => println!("player {username} left"),
                ServerPacket::Disconnect { reason } => println!("disconnected for reason: {reason}"),

                ServerPacket::ChunkData { x: _, y: _, z: _, blocks } => {
                    chunk.set_blocks(blocks);
                    chunk.generate_mesh(&display);
                }
            }
        }

        camera.tick(delta);

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 1.0, 0.0, 1.0), 1.0);

        let perspective = camera.perspective(&target);
        let view = camera.view_matrix();

        chunk.render(&mut target, perspective, view, &program, &params);

        target.finish().unwrap();
    })
}

fn transmitting_thread(client: Arc<NetworkClient>, rx: mpsc::Receiver<UserPacket>) {
    while let Ok(packet) = rx.recv() {
        let leave = matches!(packet, UserPacket::Leave);
        match client.send_packet(packet) {
            Ok(_) => (),
            Err(e) => eprintln!("could not send packet: {e}"),
        }

        if leave {
            break;
        }
    }
}

fn receiving_thread(client: Arc<NetworkClient>, tx: mpsc::Sender<ServerPacket>) {
    loop {
        match client.recv_packet() {
            Ok(packet) => {
                let disconnected = matches!(packet, ServerPacket::Disconnect { .. });
                tx.send(packet).expect("must be open");
                if disconnected {
                    break;
                }
            }
            Err(e) => eprintln!("failed to receive packet: {e}"),
        }
    }
}
