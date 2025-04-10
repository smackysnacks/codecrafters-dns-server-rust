#![warn(rust_2018_idioms)]
// #![allow(unused)]

mod error;
mod message;

use message::{ByteSerialize, DnsMessage, Opcode};

use std::{io::Cursor, net::SocketAddr, sync::Arc};

use tokio::{net::UdpSocket, sync::mpsc};

async fn handle(sock: Arc<UdpSocket>, bytes: Vec<u8>, addr: SocketAddr) {
    let mut bytes = Cursor::new(&*bytes);

    let mut message = match DnsMessage::try_parse(&mut bytes) {
        Ok(message) => message,
        Err(e) => {
            eprintln!("failed parsing packet as DnsMessage: {}", e);
            return;
        }
    };

    // TODO: forward request request

    message.header.qr_indicator = true;
    message.header.authoritative_answer = false;
    message.header.truncation = false;
    message.header.recursion_available = false;
    message.header.reserved = 0;
    if message.header.opcode == Opcode::StandardQuery {
        message.header.response_code = 0;
    } else {
        message.header.response_code = 4;
    }
    message.header.answer_record_count = message.header.question_count;

    let mut buf = Vec::with_capacity(128);
    message.serialize(&mut buf).unwrap();
    match sock.send_to(&buf, &addr).await {
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
        match sock.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                println!("Received {} bytes from {}", len, addr);
                tx.send((buf[..len].to_vec(), addr)).await?;
            }
            Err(e) => eprintln!("Error receiving: {e}"),
        }
    }
}
