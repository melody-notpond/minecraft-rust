use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use glium::glutin::dpi::PhysicalPosition;
use glium::glutin::event::{ElementState, VirtualKeyCode, MouseButton};
use glium::glutin::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use glium::{Display, Program, Surface, PolygonMode};
use minecraft_rust::blocks::Block;
use minecraft_rust::client::light::LightSource;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use minecraft_rust::client::camera::{Camera, RaycastAction};
use minecraft_rust::client::chunk::{Chunk, ChunkWaiter, Mesh};
use minecraft_rust::packet::{ServerPacket, UserPacket};
use minecraft_rust::client::player::Player;

const USERNAME: &str = "uwu";
const ADDRESS: &str = "0.0.0.0:6942";

const CHUNKS_VERTEX_SHADER: &str = include_str!("../shaders/chunks-vertex.glsl");
const CHUNKS_FRAGMENT_SHADER: &str = include_str!("../shaders/chunks-fragment.glsl");
const ENTITY_VERTEX_SHADER: &str = include_str!("../shaders/entity-vertex.glsl");
const ENTITY_FRAGMENT_SHADER: &str = include_str!("../shaders/entity-fragment.glsl");

fn main() {
    let (send_tx, send_rx) = mpsc::channel(128);
    let (recv_tx, recv_rx) = mpsc::channel(128);
    let send_tx2 = send_tx.clone();
    thread::spawn(|| networking_loop(send_tx2, send_rx, recv_tx));
    main_loop(send_tx, recv_rx);
}

fn main_loop(tx: mpsc::Sender<UserPacket>, mut rx: mpsc::Receiver<ServerPacket>) {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new();
    let cb = ContextBuilder::new().with_depth_buffer(24);
    let display = Display::new(wb, cb, &event_loop).unwrap();
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

    let chunks_program = Program::from_source(&display, CHUNKS_VERTEX_SHADER, CHUNKS_FRAGMENT_SHADER, None).unwrap();
    let entity_program = Program::from_source(&display, ENTITY_VERTEX_SHADER, ENTITY_FRAGMENT_SHADER, None).unwrap();

    let mut params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
        ..Default::default()
    };

    let mut camera = Camera::new(10.0, 0.001, 90.0);
    let mut chunks = HashMap::new();
    let mut lights = vec![LightSource::new(15, 15, 15, camera.get_pos())];
    let mut players = HashMap::new();
    let square = Mesh::square(&display);
    Block::register_defaults();
    let block_textures = Block::generate_atlas(&display);

    for x in -2..=2 {
        for y in -2..=2 {
            for z in -2..=2 {
                chunks.insert((x, y, z), ChunkWaiter::Timestamp(0));
            }
        }
    }

    let mut frame_count = 0;
    let mut last = Instant::now();
    let mut last_frame = last;
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                        tx.blocking_send(UserPacket::Disconnect).unwrap();
                    }

                    WindowEvent::Focused(false) => {
                        let gl_window = display.gl_window();
                        let window = gl_window.window();
                        window.set_cursor_visible(true);
                        window.set_cursor_grab(false).unwrap();
                        locked = false;
                    }

                    WindowEvent::KeyboardInput { input, .. } if matches!(input.virtual_keycode, Some(VirtualKeyCode::Escape)) && matches!(input.state, ElementState::Released) => {
                        let gl_window = display.gl_window();
                        let window = gl_window.window();
                        let size = window.inner_size();
                        let centre = PhysicalPosition::new(size.width / 2, size.height / 2);
                        window.set_cursor_position(centre).unwrap();
                        window.set_cursor_visible(locked);
                        locked ^= true;
                        window.set_cursor_grab(locked).unwrap();
                    }

                    WindowEvent::KeyboardInput { input, .. } if locked && camera.move_self(input) => (),

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
                        camera.raycast(&display, &mut chunks, RaycastAction::Unselect);
                        camera.turn_self(
                            position.x as i32 - centre.x as i32,
                            position.y as i32 - centre.y as i32,
                        );
                        camera.raycast(&display, &mut chunks, RaycastAction::Select);
                        window.set_cursor_position(centre).unwrap();
                    }

                    WindowEvent::MouseInput { button, state, .. } if locked => {
                        if state == ElementState::Pressed {
                            match button {
                                MouseButton::Left => camera.raycast(&display, &mut chunks, RaycastAction::Place(Block::get("solid").unwrap_or_else(Block::air))),
                                MouseButton::Right => camera.raycast(&display, &mut chunks, RaycastAction::Remove),

                                _ => (),
                            }
                        }
                    }

                    _ => (),
                }

                return;
            }

            Event::NewEvents(cause) => match cause {
                StartCause::Init => {
                    last = Instant::now();
                    last_frame = last;
                    let next_frame_time = last + Duration::from_nanos(16_666_667);
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
        let mut keys: Vec<(i32, i32, i32)> = Vec::new();

        while let Ok(packet) = rx.try_recv() {
            match packet {
                ServerPacket::ConnectionAccepted => (),
                ServerPacket::Disconnected { .. } => (),
                ServerPacket::Pong { .. } => (),

                ServerPacket::UserJoin { name, pos } => {
                    players.insert(name.clone(), Player::new(name, pos, &display));
                }

                ServerPacket::UserLeave { name } => {
                    players.remove(&name);
                }

                ServerPacket::MoveUser { name, pos } => {
                    if let Some(player) = players.get_mut(&name) {
                        player.position = pos;
                    }
                }

                ServerPacket::NewChunk { chunk } => {
                    let coords = (chunk.get_chunk_x(), chunk.get_chunk_y(), chunk.get_chunk_z());
                    chunks.insert(coords, ChunkWaiter::Chunk(Chunk::from_server_chunk(chunk)));
                    let (x, y, z) = coords;

                    if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(x - 1, y, z)) {
                        chunk.invalidate_mesh();
                    }
                    if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(x + 1, y, z)) {
                        chunk.invalidate_mesh();
                    }
                    if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(x, y - 1, z)) {
                        chunk.invalidate_mesh();
                    }
                    if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(x, y + 1, z)) {
                        chunk.invalidate_mesh();
                    }
                    if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(x, y, z - 1)) {
                        chunk.invalidate_mesh();
                    }
                    if let Some(ChunkWaiter::Chunk(chunk)) = chunks.get_mut(&(x, y, z + 1)) {
                        chunk.invalidate_mesh();
                    }
                }
            }
        }

        camera.raycast(&display, &mut chunks, RaycastAction::Unselect);
        camera.tick(delta);
        camera.raycast(&display, &mut chunks, RaycastAction::Select);
        if camera.is_moving() {
            let _ = tx.try_send(UserPacket::MoveSelf { pos: camera.get_pos() });
            if let Some(light) = lights.get_mut(0) {
                light.invalidate_chunk_lighting(&mut chunks);
                light.set_location(camera.get_pos());
                light.invalidate_chunk_lighting(&mut chunks);
            }

            //camera.check_loaded_chunks(&mut chunks);
        }

        let mut target = display.draw();
        target.clear_color_and_depth((0.53, 0.80, 0.92, 1.0), 1.0);

        let perspective = camera.perspective(&target);
        let view = camera.view_matrix();
        let frustum = camera.frustum(&target);

        let mut changed = 0;
        keys.extend(chunks.keys());
        for key in keys.iter() {
            let mut chunk = chunks.remove(key).unwrap();
            if let ChunkWaiter::Chunk(chunk) = &mut chunk {
                if chunk.loaded {
                    if changed < 50 && chunk.generate_mesh(&display, &chunks) {
                        changed += 1;
                        chunk.invalidate_lights();
                        chunk.invalidate_selection();
                        chunk.select(&display, None);
                    }

                    chunk.populate_lights(&display, &lights);

                    if chunk.aabb().is_in_frustum(&frustum) {
                        chunk.render(&mut target, &chunks_program, perspective, view, &params, &square, &block_textures);
                    }
                }
            }
            chunks.insert(*key, chunk);
        }

        keys.clear();

        let timestamp = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        for ((x, y, z), chunk) in chunks.iter_mut() {
            if let Some(stamp) = chunk.timestamp() {
                if timestamp - stamp > 100_000 {
                    *chunk = ChunkWaiter::Timestamp(timestamp);
                    let _ = tx.try_send(UserPacket::RequestChunk { x: *x, y: *y, z: *z });
                }
            }
        }

        for (_, player) in players.iter() {
            player.render(&mut target, &entity_program, perspective, view, &params);
        }

        target.finish().unwrap();

        last = Instant::now();
        let next_frame_time = last + Duration::from_nanos(16_666_667);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
    });
}

#[tokio::main]
async fn networking_loop(tx: mpsc::Sender<UserPacket>, rx: mpsc::Receiver<UserPacket>, recv_tx: mpsc::Sender<ServerPacket>) -> std::io::Result<()> {
    let sock = Arc::new(UdpSocket::bind(ADDRESS).await.unwrap());
    sock.connect("127.0.0.1:6429").await?;

    tokio::spawn(transmitting(rx, sock.clone()));
    receiving(tx, sock, recv_tx).await
}

async fn transmitting(mut rx: mpsc::Receiver<UserPacket>, sock: Arc<UdpSocket>) -> std::io::Result<()> {
    sock.send(&bincode::serialize(&UserPacket::ConnectionRequest { name: String::from(USERNAME), }).unwrap()).await?;

    while let Some(packet) = rx.recv().await {
        sock.send(&bincode::serialize(&packet).unwrap()).await?;
    }

    Ok(())
}

async fn receiving(tx: mpsc::Sender<UserPacket>, sock: Arc<UdpSocket>, recv_tx: mpsc::Sender<ServerPacket>) -> std::io::Result<()> {
    let mut buf = Box::new([0; 2usize.pow(20)]);
    loop {
        let len = sock.recv(&mut *buf).await?;
        let packet: ServerPacket = bincode::deserialize(&buf[..len]).unwrap();

        match packet {
            ServerPacket::ConnectionAccepted => {
                println!("Connected to server!");
                tokio::spawn(ping(tx.clone()));
            }

            ServerPacket::Disconnected { reason } => {
                println!("Disconnected from server for reason {}", reason);
                break Ok(());
            }

            ServerPacket::Pong { timestamp } => {
                let now = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
                let duration = now - timestamp;
                let duration = Duration::from_nanos(duration as u64);
                println!("Pong! {:?}", duration);
            }

            ServerPacket::UserJoin { name, pos } => {
                println!("{} joined the game", name);
                recv_tx.send(ServerPacket::UserJoin { name, pos }).await.unwrap();
            }

            ServerPacket::UserLeave { name } => {
                println!("{} left the game", name);
                recv_tx.send(ServerPacket::UserLeave { name }).await.unwrap();
            }

            ServerPacket::MoveUser { name, pos } => {
                recv_tx.send(ServerPacket::MoveUser { name, pos }).await.unwrap();
            }

            ServerPacket::NewChunk { chunk } => {
                recv_tx.send(ServerPacket::NewChunk { chunk }).await.unwrap();
            }
        }
    }
}

async fn ping(tx: mpsc::Sender<UserPacket>) {
    loop {
        let timestamp = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        tx.send(UserPacket::Ping{ timestamp }).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
