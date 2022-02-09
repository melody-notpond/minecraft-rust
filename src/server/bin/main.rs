use std::{collections::{HashMap, HashSet, hash_map::Entry}, io, net::SocketAddr, sync::Arc};

use tokio::{net::UdpSocket, sync::{mpsc, Mutex}};

use minecraft_rust::{packet::{ServerPacket, UserPacket}, server::{chunk::{Chunk, PerlinChunkGenerator}, player::Player}};

const BIND: &str = "127.0.0.1:6942";

#[tokio::main]
async fn main() -> io::Result<()> {
    let run = Arc::new(Mutex::new(true));
    let run2 = run.clone();
    ctrlc::set_handler(move || {
        *run2.blocking_lock() = false;
    }).unwrap();

    let sock = Arc::new(UdpSocket::bind(BIND).await.unwrap());
    let players = Arc::new(Mutex::new(HashMap::new()));
    let player_names = HashSet::new();
    let chunks = Arc::new(Mutex::new(HashMap::new()));
    let (packet_tx, packet_rx) = mpsc::channel(128);
    let (chunk_tx, chunk_rx) = mpsc::channel(128);

    println!("Server started");
    tokio::spawn(transmitting(packet_rx, sock.clone()));
    tokio::spawn(receiving(packet_tx.clone(), sock.clone(), players.clone(), player_names, chunk_tx));
    tokio::spawn(chunk_generator(packet_tx.clone(), chunk_rx, chunks.clone()));

    while *run.lock().await { }

    println!("Closing server");
    for (_, player) in players.lock().await.iter() {
        packet_tx.send((player.addr, ServerPacket::Disconnected { reason: String::from("Server closed") })).await.unwrap();
    }

    Ok(())
}

async fn chunk_generator(tx: mpsc::Sender<(SocketAddr, ServerPacket)>, mut rx: mpsc::Receiver<(SocketAddr, UserPacket)>, chunks: Arc<Mutex<HashMap<(i32, i32, i32), Chunk>>>) {
    let mut gen = PerlinChunkGenerator::default();
    let mut i = 0;

    while let Some((addr, packet)) = rx.recv().await {
        match packet {
            UserPacket::ConnectionRequest { .. } => (),
            UserPacket::Disconnect => (),
            UserPacket::Ping { .. } => (),
            UserPacket::MoveSelf { .. } => (),

            UserPacket::RequestChunk { x, y, z } => {
                let coords = (x, y, z);
                let mut chunks = chunks.lock().await;
                match chunks.entry(coords) {
                    Entry::Occupied(e) => {
                        let chunk = e.get();
                        tx.send((addr, ServerPacket::NewChunk { chunk: chunk.clone() })).await.unwrap();
                    }

                    Entry::Vacant(e) => {
                        let chunk = Chunk::new(x, y, z, &mut gen);
                        e.insert(chunk.clone());
                        i += 1;
                        println!("generated {} chunks", i);
                        tx.send((addr, ServerPacket::NewChunk { chunk })).await.unwrap();
                    }
                }
            }
        }
    }
}

async fn transmitting(mut rx: mpsc::Receiver<(SocketAddr, ServerPacket)>, sock: Arc<UdpSocket>) -> io::Result<()> {
    while let Some((addr, packet)) = rx.recv().await {
        let buf = bincode::serialize(&packet).unwrap();
        let send = sock.send_to(&buf, addr).await;
        if let Err(err) = send {
            println!("{:?} {}", err, buf.len());
            return Err(err);
        }
    }

    Ok(())
}

async fn receiving(packet_tx: mpsc::Sender<(SocketAddr, ServerPacket)>, sock: Arc<UdpSocket>, players: Arc<Mutex<HashMap<SocketAddr, Player>>>, mut player_names: HashSet<String>, chunk_tx: mpsc::Sender<(SocketAddr, UserPacket)>) -> io::Result<()> {
    let mut buf = Box::new([0; 4096]);

    loop {
        let (len, addr) = sock.recv_from(&mut *buf).await?;
        let packet: UserPacket = bincode::deserialize(&buf[..len]).unwrap();

        match packet {
            UserPacket::ConnectionRequest { name } => {
                let mut players = players.lock().await;

                if player_names.contains(&name) {
                    println!("Duplicate connection for {} at address {}", name, addr);
                    packet_tx.send((addr, ServerPacket::Disconnected { reason: format!("Player {} is already on the server!", name) })).await.unwrap();
                } else if let Entry::Vacant(e) = players.entry(addr) {
                    println!("Connection requested from {} at address {}", name, addr);

                    packet_tx.send((addr, ServerPacket::ConnectionAccepted)).await.unwrap();

                    let position = [0.0, 0.0, 0.0];
                    e.insert(Player {
                        name: name.clone(),
                        addr,
                        position,
                    });

                    for (_, player) in players.iter() {
                        if player.addr != addr {
                            packet_tx.send((player.addr, ServerPacket::UserJoin { name: name.clone(), pos: position, })).await.unwrap();
                            packet_tx.send((addr, ServerPacket::UserJoin { name: player.name.clone(), pos: player.position, })).await.unwrap();
                        }
                    }

                    player_names.insert(name);
                } else {
                    println!("Duplicate connection for address {} by {}", addr, name);
                    packet_tx.send((addr, ServerPacket::Disconnected { reason: String::from("Address already in use") })).await.unwrap();
                }
            }

            UserPacket::Disconnect => {
                let mut players = players.lock().await;

                if let Some(player) = players.remove(&addr) {
                    println!("Player {} at address {} disconnected from the server", player.name, addr);
                    player_names.remove(&player.name);

                    for (_, player2) in players.iter() {
                        packet_tx.send((player2.addr, ServerPacket::UserLeave { name: player.name.clone() })).await.unwrap();
                    }
                }
            }

            UserPacket::Ping { timestamp } => {
                if players.lock().await.contains_key(&addr) {
                    packet_tx.send((addr, ServerPacket::Pong { timestamp })).await.unwrap();
                    println!("Ping! ({})", addr);
                }
            }

            UserPacket::MoveSelf { pos } => {
                let mut players = players.lock().await;
                if let Some(mut player) = players.remove(&addr) {
                    println!("Player {} moved from {:?} to {:?}", player.name, player.position, pos);
                    player.position = pos;

                    for (_, player2) in players.iter() {
                        packet_tx.send((player2.addr, ServerPacket::MoveUser { name: player.name.clone(), pos: player.position })).await.unwrap();
                    }

                    players.insert(addr, player);
                }
            }
            UserPacket::RequestChunk { x, y, z } => chunk_tx.send((addr, UserPacket::RequestChunk { x, y, z })).await.unwrap(),
        }
    }
}

