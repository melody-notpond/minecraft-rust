use std::{
    collections::{HashMap, hash_map::Entry, HashSet},
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};

use glium::{
    glutin::{
        dpi::PhysicalPosition,
        event::{ElementState, Event, StartCause, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    BackfaceCullingMode, Depth, DepthTest, Display, DrawParameters, PolygonMode, Program, Surface,
};
use minecraft_rust::{
    client::{
        camera::Camera,
        chunk::{Chunk, ChunkMesh, InstanceData, SquareMesh},
        NetworkClient,
    },
    packet::{ServerPacket, UserPacket},
    CHUNK_SIZE,
};

fn main() {
    let addr = "0.0.0.0:6942";
    let client = NetworkClient::new("uwu", addr).expect("could not start client");
    let server = "127.0.0.1:6429";
    client
        .connect_to_server(server)
        .expect("could not connect to server");
    let (packet_tx, rx) = mpsc::channel();
    let client = Arc::new(client);
    let c = client.clone();
    std::thread::spawn(move || transmitting_thread(c, rx));
    let (tx, packet_rx) = mpsc::channel();
    let (chunk_tx, rx) = mpsc::channel();
    let ctx = chunk_tx.clone();
    std::thread::spawn(move || receiving_thread(client, tx, ctx));
    let (tx, chunk_rx) = mpsc::sync_channel(10);
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

    for x in -5..5 {
        for y in -5..5 {
            for z in -5..5 {
                chunk_waits.insert((x, y, z), Instant::now());
                packet_tx
                    .send(UserPacket::ChunkRequest { x, y, z })
                    .expect("must be open");
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
                        chunk_tx
                            .send(ChunkHandlerInstruction::Stop)
                            .expect("must be open");
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

        let delta = last.elapsed();
        last = Instant::now();

        while let Ok(packet) = packet_rx.try_recv() {
            match packet {
                PacketData::Server(ServerPacket::ConnectionAccepted) => println!("connection accepted"),
                PacketData::Server(ServerPacket::Pong) => println!("ping"),
                PacketData::Server(ServerPacket::PlayerJoined { username }) => println!("player {username} joined"),
                PacketData::Server(ServerPacket::PlayerLeft { username }) => println!("player {username} left"),
                PacketData::Server(ServerPacket::Disconnect { reason }) => {
                    println!("disconnected for reason: {reason}")
                }

                PacketData::Server(ServerPacket::ChunkData { .. }) => unreachable!("already handled"),

                PacketData::ChunkRequest { x, y, z } => {
                    chunk_waits.remove(&(x, y, z));
                    chunks.insert((x, y, z), ChunkMesh::new(x, y, z));
                    chunk_tx.send(ChunkHandlerInstruction::Request { x, y, z }).expect("must be open");
                }
            }
        }

        while let Ok((coords, chunk_data)) = chunk_rx.try_recv() {
            match chunk_data {
                ChunkHandlerResult::Mesh { mesh } => {
                    match chunks.entry(coords) {
                        Entry::Vacant(v) => {
                            v.insert(ChunkMesh::new(coords.0, coords.1, coords.2)).replace_mesh(&display, mesh);
                        }

                        Entry::Occupied(mut v) => {
                            v.get_mut().replace_mesh(&display, mesh);
                        }
                    }
                }
            }
        }

        for (&(x, y, z), timestamp) in chunk_waits.iter_mut() {
            if timestamp.elapsed().as_millis() > 800 {
                *timestamp = Instant::now();
                packet_tx
                    .send(UserPacket::ChunkRequest { x, y, z })
                    .expect("must be open");
            }
        }

        camera.tick(delta);

        if camera.is_moving() {
            for (_, chunk) in chunks.iter_mut() {
                chunk.set_rendering(false);
            }

            let coords = camera.get_pos();
            let coords = ((coords[0] / CHUNK_SIZE as f32 / 0.25).round() as i32, (coords[1] / CHUNK_SIZE as f32 / 0.25).round() as i32, (coords[2] / CHUNK_SIZE as f32 / 0.25).round() as i32);

            for x in -5..5 {
                for y in -5..5 {
                    for z in -5..5 {
                        if let Some(chunk) = chunks.get_mut(&(coords.0 + x, coords.1 + y, coords.2 + z)) {
                            chunk.set_rendering(true);
                            if !chunk.has_mesh() {
                                chunk_tx.send(ChunkHandlerInstruction::Request { x: coords.0 + x, y: coords.1 + y, z: coords.2 + z }).expect("must be open");
                            }
                        } else {
                            chunk_waits.insert((coords.0 + x, coords.1 + y, coords.2 + z), Instant::now());
                            packet_tx.send(UserPacket::ChunkRequest { x: coords.0 + x, y: coords.1 + y, z: coords.2 + z }).expect("must be open");
                        }
                    }
                }
            }

            for (_, chunk) in chunks.iter_mut() {
                chunk.invalidate_mesh();
            }
        }

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

enum PacketData {
    Server(ServerPacket),
    ChunkRequest { x: i32, y: i32, z: i32 },
}

fn receiving_thread(client: Arc<NetworkClient>, main_tx: mpsc::Sender<PacketData>, chunk_tx: mpsc::Sender<ChunkHandlerInstruction>) {
    loop {
        match client.recv_packet() {
            Ok(packet) => {
                let disconnected = matches!(packet, ServerPacket::Disconnect { .. });
                if let ServerPacket::ChunkData { x, y, z, blocks } = packet {
                    chunk_tx.send(ChunkHandlerInstruction::ChunkData { x, y, z, blocks }).expect("must be open");
                    main_tx.send(PacketData::ChunkRequest { x, y, z }).expect("must be open");
                } else {
                    main_tx.send(PacketData::Server(packet)).expect("must be open");
                }
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
    },
    Request {
        x: i32,
        y: i32,
        z: i32,
    }
}

enum ChunkHandlerResult {
    Mesh { mesh: Vec<InstanceData> },
}

fn send_chunk_data(x: i32, y: i32, z: i32, chunk: &Chunk, chunks: &HashMap<(i32, i32, i32), Chunk>, main_tx: &mpsc::SyncSender<((i32, i32, i32), ChunkHandlerResult)>) {
    main_tx
        .send((
            (x, y, z),
            ChunkHandlerResult::Mesh {
                mesh: chunk.generate_mesh(chunks),
            },
        ))
        .expect("must be open");

    if let Some(chunk) = chunks.get(&(x - 1, y, z)) {
        main_tx
            .send((
                (x - 1, y, z),
                ChunkHandlerResult::Mesh {
                    mesh: chunk.generate_mesh(chunks),
                },
            ))
            .expect("must be open");
    }
    if let Some(chunk) = chunks.get(&(x + 1, y, z)) {
        main_tx
            .send((
                (x + 1, y, z),
                ChunkHandlerResult::Mesh {
                    mesh: chunk.generate_mesh(chunks),
                },
            ))
            .expect("must be open");
    }
    if let Some(chunk) = chunks.get(&(x, y - 1, z)) {
        main_tx
            .send((
                (x, y - 1, z),
                ChunkHandlerResult::Mesh {
                    mesh: chunk.generate_mesh(chunks),
                },
            ))
            .expect("must be open");
    }
    if let Some(chunk) = chunks.get(&(x, y + 1, z)) {
        main_tx
            .send((
                (x, y + 1, z),
                ChunkHandlerResult::Mesh {
                    mesh: chunk.generate_mesh(chunks),
                },
            ))
            .expect("must be open");
    }
    if let Some(chunk) = chunks.get(&(x, y, z - 1)) {
        main_tx
            .send((
                (x, y, z - 1),
                ChunkHandlerResult::Mesh {
                    mesh: chunk.generate_mesh(chunks),
                },
            ))
            .expect("must be open");
    }
    if let Some(chunk) = chunks.get(&(x, y, z + 1)) {
        main_tx
            .send((
                (x, y, z + 1),
                ChunkHandlerResult::Mesh {
                    mesh: chunk.generate_mesh(chunks),
                },
            ))
            .expect("must be open");
    }
}

fn chunk_handler_thread(
    main_rx: mpsc::Receiver<ChunkHandlerInstruction>,
    main_tx: mpsc::SyncSender<((i32, i32, i32), ChunkHandlerResult)>,
) {
    let mut chunks = HashMap::new();
    let mut requested_but_unavailable = HashSet::new();

    while let Ok(instr) = main_rx.recv() {
        match instr {
            ChunkHandlerInstruction::Stop => break,
            ChunkHandlerInstruction::ChunkData { x, y, z, blocks } => {
                let chunk = Chunk::from_data(x, y, z, blocks);
                if requested_but_unavailable.remove(&(x, y, z)) {
                    send_chunk_data(x, y, z, &chunk, &chunks, &main_tx);
                }
                chunks.insert((x, y, z), chunk);
            }

            ChunkHandlerInstruction::Request { x, y, z } => {
                if let Some(chunk) = chunks.get(&(x, y, z)) {
                    send_chunk_data(x, y, z, chunk, &chunks, &main_tx);
                } else {
                    requested_but_unavailable.insert((x, y, z));
                }
            }
        }
    }
}
