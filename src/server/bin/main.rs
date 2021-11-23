use std::{collections::{HashMap, HashSet, hash_map::Entry}, io, net::SocketAddr, sync::Arc};

use tokio::{net::UdpSocket, sync::{mpsc, Mutex}};

use minecraft_rust::{packet::{ServerPacket, UserPacket}, server::player::Player};

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
    let player_names = Arc::new(Mutex::new(HashSet::new()));
    let (tx, rx) = mpsc::channel(128);

    println!("Server started");
    tokio::spawn(transmitting(rx, sock.clone()));
    tokio::spawn(receiving(tx.clone(), sock.clone(), players.clone(), player_names));

    while *run.lock().await { }

    println!("Closing server");
    for (_, player) in players.lock().await.iter() {
        tx.send((player.addr, ServerPacket::Disconnected { reason: String::from("Server closed") })).await.unwrap();
    }

    Ok(())
}

async fn transmitting(mut rx: mpsc::Receiver<(SocketAddr, ServerPacket)>, sock: Arc<UdpSocket>) -> io::Result<()> {
    while let Some((addr, packet)) = rx.recv().await {
        let buf = bincode::serialize(&packet).unwrap();
        sock.send_to(&buf, addr).await?;
    }

    Ok(())
}

async fn receiving(tx: mpsc::Sender<(SocketAddr, ServerPacket)>, sock: Arc<UdpSocket>, players: Arc<Mutex<HashMap<SocketAddr, Player>>>, player_names: Arc<Mutex<HashSet<String>>>) -> io::Result<()> {
    let mut buf = Box::new([0; 1024]);

    loop {
        let (len, addr) = sock.recv_from(&mut *buf).await?;
        let packet: UserPacket = bincode::deserialize(&buf[..len]).unwrap();

        match packet {
            UserPacket::ConnectionRequest { name } => {
                let mut players = players.lock().await;

                if player_names.lock().await.contains(&name) {
                    println!("Duplicate connection for {} at address {}", name, addr);
                    tx.send((addr, ServerPacket::Disconnected { reason: format!("Player {} is already on the server!", name) })).await.unwrap();
                } else if let Entry::Vacant(e) = players.entry(addr) {
                    println!("Connection requested from {} at address {}", name, addr);

                    tx.send((addr, ServerPacket::ConnectionAccepted)).await.unwrap();

                    let position = [0.0, 0.0, 0.0];
                    e.insert(Player {
                        name: name.clone(),
                        addr,
                        position,
                    });

                    for (_, player) in players.iter() {
                        if player.addr != addr {
                            tx.send((player.addr, ServerPacket::UserJoin { name: name.clone(), pos: position, })).await.unwrap();
                            tx.send((addr, ServerPacket::UserJoin { name: player.name.clone(), pos: player.position, })).await.unwrap();
                        }
                    }

                    player_names.lock().await.insert(name);
                } else {
                    println!("Duplicate connection for address {} by {}", addr, name);
                    tx.send((addr, ServerPacket::Disconnected { reason: String::from("Address already in use") })).await.unwrap();
                }
            }

            UserPacket::Disconnect => {
                let mut players = players.lock().await;

                if let Some(player) = players.remove(&addr) {
                    println!("Player {} at address {} disconnected from the server", player.name, addr);
                    player_names.lock().await.remove(&player.name);

                    for (_, player2) in players.iter() {
                        tx.send((player2.addr, ServerPacket::UserLeave { name: player.name.clone() })).await.unwrap();
                    }
                }
            }

            UserPacket::Ping { timestamp } => {
                if players.lock().await.contains_key(&addr) {
                    tx.send((addr, ServerPacket::Pong { timestamp })).await.unwrap();
                    println!("Ping! ({})", addr);
                }
            }

            UserPacket::MoveSelf { pos } => {
                let mut players = players.lock().await;
                if let Some(mut player) = players.remove(&addr) {
                    println!("Player {} moved from {:?} to {:?}", player.name, player.position, pos);
                    player.position = pos;

                    for (_, player2) in players.iter() {
                        tx.send((player2.addr, ServerPacket::MoveUser { name: player.name.clone(), pos: player.position })).await.unwrap();
                    }

                    players.insert(addr, player);
                }
            }
        }
    }
}

