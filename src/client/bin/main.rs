use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use glium::glutin::dpi::PhysicalPosition;
use glium::glutin::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use glium::{Display, Program, Surface};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use minecraft_rust::client::camera::Camera;
use minecraft_rust::client::chunk::Chunk;
use minecraft_rust::packet::{ServerPacket, UserPacket};

const VERTEX_SHADER: &str = include_str!("../shaders/vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("../shaders/fragment.glsl");

fn main() -> std::io::Result<()> {
    thread::spawn(networking_loop);
    gui();
    Ok(())
}

fn gui() {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new();
    let cb = ContextBuilder::new().with_depth_buffer(24);
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
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        ..Default::default()
    };

    let mut camera = Camera::new(10.0, 0.01, 90.0);
    let mut chunk = Chunk::new(0, 0, 0);

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
        camera.tick(delta);

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 1.0, 0.0, 1.0), 1.0);

        let perspective = camera.perspective(&target);

        let view = camera.view_matrix();

        chunk.generate_mesh(&display);
        chunk.render(&mut target, &program, perspective, view, &params);

        target.finish().unwrap();

        last = Instant::now();
        let next_frame_time = last + Duration::from_nanos(16_666_667);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
    });
}

#[tokio::main]
async fn networking_loop() -> std::io::Result<()> {
    let sock = Arc::new(UdpSocket::bind("0.0.0.0:4269").await?);
    sock.connect("127.0.0.1:6942").await?;

    let (tx, rx) = mpsc::channel(128);

    tokio::spawn(transmitting(rx, sock.clone()));
    receiving(tx, sock).await
}

async fn transmitting(mut rx: mpsc::Receiver<UserPacket>, sock: Arc<UdpSocket>) -> std::io::Result<()> {
    sock.send(&bincode::serialize(&UserPacket::ConnectionRequest { name: String::from("uwu"), }).unwrap()).await.unwrap();
    while let Some(packet) = rx.recv().await {
        sock.send(&bincode::serialize(&packet).unwrap()).await?;
    }

    Ok(())
}

async fn receiving(tx: mpsc::Sender<UserPacket>, sock: Arc<UdpSocket>) -> std::io::Result<()> {
    let mut buf = Box::new([0; 1024]);

    loop {
        let len = sock.recv(&mut *buf).await?;
        let packet: ServerPacket = bincode::deserialize(&buf[..len]).unwrap();
        println!("{:?}", packet);

        match packet {
            ServerPacket::ConnectionAccepted => {
                println!("Connected to server!");
                tx.send(UserPacket::Ping).await.unwrap();
            }

            ServerPacket::Disconnected { .. } => {
                println!("Disconnected");
            }

            ServerPacket::Pong => {
                println!("Pong!");
                tx.send(UserPacket::Ping).await.unwrap();
            }
        }
    }
}
