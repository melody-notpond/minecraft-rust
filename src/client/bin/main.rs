use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use glium::glutin::dpi::PhysicalPosition;
use glium::glutin::event::{ElementState, VirtualKeyCode};
use glium::glutin::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use glium::{Display, Program, Surface};
use minecraft_rust::client::player::Player;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use minecraft_rust::client::camera::Camera;
use minecraft_rust::client::chunk::Chunk;
use minecraft_rust::packet::{ServerPacket, UserPacket};

const USERNAME: &str = "uwu";
const ADDRESS: &str = "0.0.0.0:4269";

const VERTEX_SHADER: &str = include_str!("../shaders/vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("../shaders/fragment.glsl");

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
    let mut locked = true;

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
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        ..Default::default()
    };

    let mut camera = Camera::new(10.0, 0.01, 90.0);
    let mut chunk = Chunk::new(0, 0, 0);
    let mut players = HashMap::new();

    let mut last = Instant::now();
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
                    let next_frame_time = last + Duration::from_nanos(16_666_667);
                    *control_flow = ControlFlow::WaitUntil(next_frame_time);
                    return;
                }

                StartCause::ResumeTimeReached { .. } => (),

                _ => return,
            },

            _ => return,
        }

        let delta = Instant::now() - last;

        while let Some(packet) = rx.try_recv().ok() {
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
            }
        }

        camera.tick(delta);
        if camera.is_moving() {
            tx.blocking_send(UserPacket::MoveSelf { pos: camera.get_pos() }).unwrap();
        }

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 1.0, 0.0, 1.0), 1.0);

        let perspective = camera.perspective(&target);

        let view = camera.view_matrix();

        chunk.generate_mesh(&display);
        chunk.render(&mut target, &program, perspective, view, &params);

        for (_, player) in players.iter() {
            player.render(&mut target, &program, perspective, view, &params);
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
    sock.connect("127.0.0.1:6942").await?;

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
    let mut buf = Box::new([0; 1024]);

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
