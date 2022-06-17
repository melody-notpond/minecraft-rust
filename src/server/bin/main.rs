use std::{sync::{Arc, mpsc}, collections::{HashMap, HashSet}, net::SocketAddr, thread::JoinHandle};

use minecraft_rust::{server::NetworkServer, packet::{UserPacket, ServerPacket}};

struct ThreadInfo {
    handler: JoinHandle<()>,
    sender: mpsc::Sender<UserPacket>,
}

fn main() {
    let addr = "0.0.0.0:6429";
    let server = match NetworkServer::new(addr) {
        Ok(socket) => socket,
        Err(e) => {
            eprintln!("Could not bind socket to address {addr}: {e}");
            std::process::exit(1);
        }
    };

    let mut addr_map = HashMap::new();
    let mut username_set = HashSet::new();
    let server = Arc::new(server);

    println!("server started");

    while let Ok((packet, addr)) = server.recv_packet() {
        if let UserPacket::JoinRequest { username } = packet {
            println!("got join request from {username}");
            if username_set.contains(&username) {
                match server.send_packet(ServerPacket::Disconnect {
                    reason: String::from("username already taken!"),
                }, addr) {
                    Ok(_) => (),
                    Err(e) => eprintln!("could not send disconnect packet: {e}"),
                }
            } else {
                username_set.insert(username.clone());

                match server.send_packet(ServerPacket::ConnectionAccepted, addr) {
                    Ok(_) => (),
                    Err(e) => eprintln!("could not send connection accepted packet: {e}"),
                }

                addr_map.entry(addr).or_insert_with(|| {
                    let (tx, rx) = mpsc::channel();
                    let copy = server.clone();
                    let handler = std::thread::spawn(move || player_handler(copy, addr, username, rx));
                    ThreadInfo {
                        handler,
                        sender: tx,
                    }
                });
            }
        } else if let Some(thread_info) = addr_map.get(&addr) {
            match thread_info.sender.send(packet) {
                Ok(_) => (),
                Err(_) => {
                    let thread_info = addr_map.remove(&addr).unwrap();
                    match thread_info.handler.join() {
                        Ok(_) => (),
                        Err(e) => eprintln!("thread for {addr} panicked or experienced some other issue: {:?}", e),
                    }

                    match server.send_packet(ServerPacket::Disconnect {
                        reason: String::from("player thread panicked; check server logs"),
                    }, addr) {
                        Ok(_) => (),
                        Err(e) => eprintln!("could not send disconnect packet: {e}"),
                    }
                }
            }
        }
    }
}

fn player_handler(_server: Arc<NetworkServer>, _addr: SocketAddr, _username: String, rx: mpsc::Receiver<UserPacket>) {
    while let Ok(packet) = rx.recv() {
        match packet {
            UserPacket::JoinRequest { .. } => unreachable!("already handled"),
            UserPacket::Ping => todo!(),
            UserPacket::Leave => {
                break;
            }
        }
    }
}

