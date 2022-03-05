use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use glium::glutin::dpi::PhysicalPosition;
use glium::glutin::event::{ElementState, MouseButton, VirtualKeyCode};
use glium::glutin::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use glium::{Display, PolygonMode, Program, Surface};
use minecraft_rust::blocks::Block;
use minecraft_rust::client::light::LightSource;
use minecraft_rust::collision::DetectCollision;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use minecraft_rust::client::camera::{Camera, RaycastAction};
use minecraft_rust::client::chunk::{Chunk, ChunkWaiter, Mesh, InstanceData};
use minecraft_rust::client::player::Player;
use minecraft_rust::packet::{ServerPacket, UserPacket};

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

    let chunks_program =
        Program::from_source(&display, CHUNKS_VERTEX_SHADER, CHUNKS_FRAGMENT_SHADER, None).unwrap();
    let entity_program =
        Program::from_source(&display, ENTITY_VERTEX_SHADER, ENTITY_FRAGMENT_SHADER, None).unwrap();

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
    let lights = Arc::new(RwLock::new(vec![LightSource::new(15, 15, 15, camera.get_pos())]));
    let mut players = HashMap::new();
    let chunks = Arc::new(RwLock::new(HashMap::new()));
    let square = Mesh::square(&display);
    Block::register_defaults();
    let block_textures = Block::generate_atlas(&display);

    for x in -3..=3 {
        for y in -1..=1 {
            for z in -3..=3 {
                chunks.write().unwrap().insert((x, y, z), RwLock::new(ChunkWaiter::Timestamp(0)));
            }
        }
    }

    let (tx2, mut chunk_data_rx) = mpsc::channel(128);
    let (chunk_data_tx, rx2) = mpsc::channel(128);
    let chunks2 = chunks.clone();
    thread::spawn(|| mesh_loop(chunks2, tx2, rx2));

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
                        if locked && camera.move_self(input) => (),

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
                        camera.raycast(&display, &*chunks.read().unwrap(), RaycastAction::Unselect, &chunk_data_tx);
                        camera.turn_self(
                            position.x as i32 - centre.x as i32,
                            position.y as i32 - centre.y as i32,
                        );
                        camera.raycast(&display, &*chunks.read().unwrap(), RaycastAction::Select, &chunk_data_tx);
                        window.set_cursor_position(centre).unwrap();
                    }

                    WindowEvent::MouseInput { button, state, .. } if locked => {
                        if state == ElementState::Pressed {
                            match button {
                                MouseButton::Left => camera.raycast(
                                    &display,
                                    &chunks.read().unwrap(),
                                    RaycastAction::Place(
                                        Block::get("solid").unwrap_or_else(Block::air),
                                    ),
                                    &chunk_data_tx,
                                ),
                                MouseButton::Right => {
                                    camera.raycast(&display, &*chunks.read().unwrap(), RaycastAction::Remove, &chunk_data_tx);
                                }

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

        let mut to_send = vec![];
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
                    let coords = (
                        chunk.get_chunk_x(),
                        chunk.get_chunk_y(),
                        chunk.get_chunk_z(),
                    );
                  
                    let mut chunks = chunks.write().unwrap();
                    chunks.insert(coords, RwLock::new(ChunkWaiter::Chunk(Chunk::from_server_chunk(&display, chunk))));
                    let (x, y, z) = coords;
                    to_send.push((x, y, z));

                    if let Some(chunk) = chunks.get(&(x - 1, y, z)) {
                        if let ChunkWaiter::Chunk(_) = &mut *chunk.write().unwrap() {
                            to_send.push((x - 1, y, z));
                        }
                    }
                    if let Some(chunk) = chunks.get(&(x + 1, y, z)) {
                        if let ChunkWaiter::Chunk(_) = &mut *chunk.write().unwrap() {
                            to_send.push((x + 1, y, z));
                        }
                    }
                    if let Some(chunk) = chunks.get(&(x, y - 1, z)) {
                        if let ChunkWaiter::Chunk(_) = &mut *chunk.write().unwrap() {
                            to_send.push((x, y - 1, z));
                        }
                    }
                    if let Some(chunk) = chunks.get(&(x, y + 1, z)) {
                        if let ChunkWaiter::Chunk(_) = &mut *chunk.write().unwrap() {
                            to_send.push((x, y + 1, z));
                        }
                    }
                    if let Some(chunk) = chunks.get(&(x, y, z - 1)) {
                        if let ChunkWaiter::Chunk(_) = &mut *chunk.write().unwrap() {
                            to_send.push((x, y, z - 1));
                        }
                    }
                    if let Some(chunk) = chunks.get(&(x, y, z + 1)) {
                        if let ChunkWaiter::Chunk(_) = &mut *chunk.write().unwrap() {
                            to_send.push((x, y, z + 1));
                        }
                    }
                }
            }
        }
        if !to_send.is_empty() {
            chunk_data_tx.blocking_send(to_send).unwrap();
        }

        while let Ok(v) = chunk_data_rx.try_recv() {
            for (coords, instance_data) in v {
                if let Some(chunk) = chunks.read().unwrap().get(&coords) {
                    if let ChunkWaiter::Chunk(chunk) = &mut *chunk.write().unwrap() {
                        chunk.set_mesh(&display, instance_data);
                        chunk.invalidate_selection();
                    }
                }
            }
        }

        camera.raycast(&display, &*chunks.read().unwrap(), RaycastAction::Unselect, &chunk_data_tx);
        camera.tick(delta);
        camera.raycast(&display, &*chunks.read().unwrap(), RaycastAction::Select, &chunk_data_tx);
        if camera.is_moving() {
            let _ = tx.try_send(UserPacket::MoveSelf {
                pos: camera.get_pos(),
            });
            if let Some(light) = lights.write().unwrap().get_mut(0) {
                light.set_location(camera.get_pos());
            }

            camera.check_loaded_chunks(&mut *chunks.write().unwrap());
        }

        for (name, player) in players.iter() {
            if camera.aabb().is_colliding(&player.aabb()) {
                println!("im colliding with {}", name);
            }
        }

        // RENDERING

        let mut target = display.draw();
        target.clear_color_and_depth((0.53, 0.80, 0.92, 1.0), 1.0);

        let perspective = camera.perspective(&target);
        let view = camera.view_matrix();
        let frustum = camera.frustum(&target);

        for (_, chunk) in chunks.read().unwrap().iter() {
            if let ChunkWaiter::Chunk(chunk) = &mut *chunk.write().unwrap() {
                if chunk.loaded {
                    chunk.select(&display, None);

                    if chunk.aabb().is_in_frustum(&frustum) {
                        chunk.render(
                            &mut target,
                            &chunks_program,
                            perspective,
                            view,
                            &params,
                            &square,
                            &block_textures,
                            &lights.read().unwrap(),
                        );
                    }
                }
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        for ((x, y, z), chunk) in chunks.read().unwrap().iter() {
            let mut chunk = chunk.write().unwrap();
            if let Some(stamp) = chunk.timestamp() {
                if timestamp - stamp > 100_000 {
                    *chunk = ChunkWaiter::Timestamp(timestamp);
                    let _ = tx.try_send(UserPacket::RequestChunk {
                        x: *x,
                        y: *y,
                        z: *z,
                    });
                }
            }
        }

        for (_, player) in players.iter() {
            player.render(&mut target, &entity_program, perspective, view, &params);
        }

        target.finish().unwrap();

        let next_frame_time = Instant::now() + Duration::from_nanos(5);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
    });
}

#[allow(clippy::type_complexity)]
fn mesh_loop(chunks: Arc<RwLock<HashMap<(i32, i32, i32), RwLock<ChunkWaiter>>>>, tx: mpsc::Sender<Vec<((i32, i32, i32), Vec<InstanceData>)>>, mut rx: mpsc::Receiver<Vec<(i32, i32, i32)>>) {
    while let Some(v) = rx.blocking_recv() {
        let mut result = vec![];
        for coords in v {
            if let Some(chunk) = chunks.read().unwrap().get(&coords) {
                let chunk = chunk.read().unwrap();
                if let ChunkWaiter::Chunk(chunk) = &*chunk {
                    let mesh = chunk.generate_mesh(&*chunks.read().unwrap());
                    result.push((coords, mesh));
                }
            }
        }

        tx.blocking_send(result).unwrap();
    }
}

#[tokio::main]
async fn networking_loop(
    tx: mpsc::Sender<UserPacket>,
    rx: mpsc::Receiver<UserPacket>,
    recv_tx: mpsc::Sender<ServerPacket>,
) -> std::io::Result<()> {
    let sock = Arc::new(UdpSocket::bind(ADDRESS).await.unwrap());
    sock.connect("127.0.0.1:6429").await?;

    tokio::spawn(transmitting(rx, sock.clone()));
    receiving(tx, sock, recv_tx).await
}

async fn transmitting(
    mut rx: mpsc::Receiver<UserPacket>,
    sock: Arc<UdpSocket>,
) -> std::io::Result<()> {
    sock.send(
        &bincode::serialize(&UserPacket::ConnectionRequest {
            name: String::from(USERNAME),
        })
        .unwrap(),
    )
    .await?;

    while let Some(packet) = rx.recv().await {
        sock.send(&bincode::serialize(&packet).unwrap()).await?;
    }

    Ok(())
}

async fn receiving(
    tx: mpsc::Sender<UserPacket>,
    sock: Arc<UdpSocket>,
    recv_tx: mpsc::Sender<ServerPacket>,
) -> std::io::Result<()> {
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
                let now = SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let duration = now - timestamp;
                let duration = Duration::from_nanos(duration as u64);
                println!("Pong! {:?}", duration);
            }

            ServerPacket::UserJoin { name, pos } => {
                println!("{} joined the game", name);
                recv_tx
                    .send(ServerPacket::UserJoin { name, pos })
                    .await
                    .unwrap();
            }

            ServerPacket::UserLeave { name } => {
                println!("{} left the game", name);
                recv_tx
                    .send(ServerPacket::UserLeave { name })
                    .await
                    .unwrap();
            }

            ServerPacket::MoveUser { name, pos } => {
                recv_tx
                    .send(ServerPacket::MoveUser { name, pos })
                    .await
                    .unwrap();
            }

            ServerPacket::NewChunk { chunk } => {
                recv_tx
                    .send(ServerPacket::NewChunk { chunk })
                    .await
                    .unwrap();
            }
        }
    }
}

async fn ping(tx: mpsc::Sender<UserPacket>) {
    loop {
        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        tx.send(UserPacket::Ping { timestamp }).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
