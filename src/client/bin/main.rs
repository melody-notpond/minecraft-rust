use std::{time::{Duration, Instant}, sync::{mpsc, Arc}, collections::HashMap};

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
use minecraft_rust::{client::{camera::Camera, chunk::{SquareMesh, Chunk, InstanceData, ChunkMesh}, NetworkClient}, packet::{UserPacket, ServerPacket}, CHUNK_SIZE};

fn main() {
    let addr = "0.0.0.0:6942";
    let client = NetworkClient::new("uwu", addr).expect("could not start client");
    let server = "127.0.0.1:6429";
    client.connect_to_server(server).expect("could not connect to server");
    let (packet_tx, rx) = mpsc::channel();
    let client = Arc::new(client);
    let c = client.clone();
    std::thread::spawn(move || transmitting_thread(c, rx));
    let (tx, packet_rx) = mpsc::channel();
    std::thread::spawn(move || receiving_thread(client, tx));
    let (chunk_tx, rx) = mpsc::channel();
    let (tx, chunk_rx) = mpsc::channel();
    std::thread::spawn(move || chunk_handler_thread(rx, tx));

    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new();
    let cb = ContextBuilder::new().with_depth_buffer(24);
    let display = Display::new(wb, cb, &event_loop).expect("could not create window");
    SquareMesh::init(&display);

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

    let mut chunks = HashMap::new();
    let mut chunk_waits = HashMap::new();

    for x in -10..10 {
        for y in 0..1 {
            for z in -10..10 {
                chunk_waits.insert((x, y, z), Instant::now());
                packet_tx.send(UserPacket::ChunkRequest { x, y, z }).expect("must be open");
            }
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
                        packet_tx.send(UserPacket::Leave).expect("must be open");
                        chunk_tx.send(ChunkHandlerInstruction::Stop).expect("must be open");
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
            packet_tx.send(UserPacket::Ping).expect("must be open");
            frame_count = 0;
            last_frame = last;
        }

        let delta = Instant::now() - last;
        last = Instant::now();

        while let Ok(packet) = packet_rx.try_recv() {
            match packet {
                ServerPacket::ConnectionAccepted => println!("connection accepted"),
                ServerPacket::Pong => println!("ping"),
                ServerPacket::PlayerJoined { username } => println!("player {username} joined"),
                ServerPacket::PlayerLeft { username } => println!("player {username} left"),
                ServerPacket::Disconnect { reason } => println!("disconnected for reason: {reason}"),

                ServerPacket::ChunkData { x, y, z, blocks } => {
                    chunk_waits.remove(&(x, y, z));
                    chunks.insert((x, y, z), ChunkMesh::new(&display, x, y, z));
                    chunk_tx.send(ChunkHandlerInstruction::ChunkData { x, y, z, blocks }).expect("must be open");
                }
            }
        }

        while let Ok((coords, chunk_data)) = chunk_rx.try_recv() {
            let chunk = chunks.get_mut(&coords).unwrap();
            match chunk_data {
                ChunkHandlerResult::Mesh { mesh } => chunk.replace_mesh(&display, mesh),
            }
        }

        for (&(x, y, z), timestamp) in chunk_waits.iter_mut() {
            if timestamp.elapsed().as_millis() > 800 {
                *timestamp = Instant::now();
                packet_tx.send(UserPacket::ChunkRequest { x, y, z }).expect("must be open");
            }
        }

        camera.tick(delta);

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 1.0, 0.0, 1.0), 1.0);

        let perspective = camera.perspective(&target);
        let view = camera.view_matrix();

        for (_, chunk) in chunks.iter() {
            chunk.render(&mut target, perspective, view, &program, &params);
        }

        target.finish().unwrap();
    })
}

fn transmitting_thread(client: Arc<NetworkClient>, main_rx: mpsc::Receiver<UserPacket>) {
    while let Ok(packet) = main_rx.recv() {
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

fn receiving_thread(client: Arc<NetworkClient>, main_tx: mpsc::Sender<ServerPacket>) {
    loop {
        match client.recv_packet() {
            Ok(packet) => {
                let disconnected = matches!(packet, ServerPacket::Disconnect { .. });
                main_tx.send(packet).expect("must be open");
                if disconnected {
                    break;
                }
            }
            Err(e) => eprintln!("failed to receive packet: {e}"),
        }
    }
}

enum ChunkHandlerInstruction {
    Stop,
    ChunkData {
        x: i32,
        y: i32,
        z: i32,
        blocks: Box<[[[u32; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
    }
}

enum ChunkHandlerResult {
    Mesh {
        mesh: Vec<InstanceData>,
    }
}

fn chunk_handler_thread(main_rx: mpsc::Receiver<ChunkHandlerInstruction>, main_tx: mpsc::Sender<((i32, i32, i32), ChunkHandlerResult)>) {
    let mut chunks = HashMap::new();
    while let Ok(instr) = main_rx.recv() {
        match instr {
            ChunkHandlerInstruction::Stop => break,
            ChunkHandlerInstruction::ChunkData { x, y, z, blocks } => {
                let chunk = Chunk::from_data(x, y, z, blocks);
                main_tx.send(((x, y, z), ChunkHandlerResult::Mesh { mesh: chunk.generate_mesh() })).expect("must be open");
                chunks.insert((x, y, z), chunk);
            }
        }
    }
}
