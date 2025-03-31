#![warn(rust_2018_idioms)]
#![allow(unused)]

use std::{net::SocketAddr, sync::Arc};

use tokio::{net::UdpSocket, sync::mpsc};

mod header;

async fn handle(sock: Arc<UdpSocket>, bytes: Vec<u8>, addr: SocketAddr) {
    match sock.send_to(&bytes, &addr).await {
        Ok(len) => println!("Sent {} bytes to {}", len, addr),
        Err(e) => eprintln!("Error sending to {}: {}", addr, e),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sock = UdpSocket::bind("127.0.0.1:2053").await?;
    let sock = Arc::new(sock);
    let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1000);

    tokio::spawn({
        let send_sock = sock.clone();

        async move {
            while let Some((bytes, addr)) = rx.recv().await {
                tokio::spawn(handle(send_sock.clone(), bytes, addr));
            }
        }
    });

    let mut buf = [0; 512];
    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;
        println!("Received {} bytes from {}", len, addr);

        tx.send((buf[..len].to_vec(), addr)).await?;
    }
}
