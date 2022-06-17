use std::net::UdpSocket;

fn main() {
    let addr = "0.0.0.0:6429";
    let socket = match UdpSocket::bind(addr) {
        Ok(socket) => socket,
        Err(e) => {
            eprintln!("Could not bind socket to address {addr}: {e}");
            std::process::exit(1);
        }
    };

    println!("server started");

    let mut data = Box::new([0; 1024]);
    while let Ok((size, addr)) = socket.recv_from(&mut *data) {
        println!("received data from {addr} of size {size}");
    }
}
