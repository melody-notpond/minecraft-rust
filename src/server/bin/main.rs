use std::io;

use tokio::net::UdpSocket;

const BIND: &str = "0.0.0.0:6942";

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("hewwo");

    let sock = UdpSocket::bind(BIND).await?;

    Ok(())
}
