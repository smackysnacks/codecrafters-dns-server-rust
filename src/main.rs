#![warn(rust_2018_idioms)]
#![allow(unused)]

mod error;
mod message;

use message::{
    Class, DnsAnswer, DnsHeader, DnsMessage, DnsQuestion, Label, Opcode, RData, ResourceRecord,
    Type,
};

use std::{net::SocketAddr, sync::Arc};

use tokio::{net::UdpSocket, sync::mpsc};

async fn handle(sock: Arc<UdpSocket>, bytes: Vec<u8>, addr: SocketAddr) {
    let result = DnsHeader::try_parse(&bytes);
    if let Err(e) = result {
        eprintln!("Error parsing DNS packet: {}", e);
        return;
    }
    let mut header = result.unwrap();

    header.qr_indicator = true;
    header.authoritative_answer = false;
    header.truncation = false;
    header.recursion_available = false;
    header.reserved = 0;
    if header.opcode == Opcode::StandardQuery {
        header.response_code = 0;
    } else {
        header.response_code = 4;
    }
    header.answer_record_count = 1;

    let question = DnsQuestion {
        name: vec![
            Label {
                content: String::from("codecrafters"),
            },
            Label {
                content: String::from("io"),
            },
        ],
        qtype: Type::A,
        class: Class::IN,
    };

    let answer = DnsAnswer {
        resource_records: vec![ResourceRecord {
            name: vec![
                Label {
                    content: String::from("codecrafters"),
                },
                Label {
                    content: String::from("io"),
                },
            ],
            atype: Type::A,
            class: Class::IN,
            ttl: 60,
            rdata: RData::A {
                address: u32::from_be_bytes([8, 8, 8, 8]),
            },
        }],
    };

    let message = DnsMessage {
        header,
        question,
        answer,
    };

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
        let (len, addr) = sock.recv_from(&mut buf).await?;
        println!("Received {} bytes from {}", len, addr);

        tx.send((buf[..len].to_vec(), addr)).await?;
    }
}
