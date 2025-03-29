#![warn(rust_2018_idioms)]
#![allow(unused)]

use tokio::net::UdpSocket;

mod header;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sock = UdpSocket::bind("127.0.0.1:2053").await?;
    let mut buf = [0; 512];

    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;
        let response = [];

        println!("Received {} bytes from {}", len, addr);
        sock.send_to(&response, addr).await?;
    }
}
