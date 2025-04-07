#![warn(rust_2018_idioms)]
#![allow(unused)]

mod error;
mod message;

use message::{
    ByteSerialize, Class, DnsAnswer, DnsHeader, DnsMessage, DnsQuestion, Label, Name, Opcode,
    RData, ResourceRecord, Type,
};

use std::{net::SocketAddr, sync::Arc};

use tokio::{net::UdpSocket, sync::mpsc};

async fn handle(sock: Arc<UdpSocket>, bytes: Vec<u8>, addr: SocketAddr) {
    let mut bytes = &*bytes;

    let mut header = match DnsHeader::try_parse(&mut bytes) {
        Ok(header) => header,
        Err(e) => {
            eprintln!("Error parsing DnsHeader: {}", e);
            return;
        }
    };

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

    let mut questions = Vec::new();
    let mut question = match DnsQuestion::try_parse(&mut bytes) {
        Ok(question) => question,
        Err(e) => {
            eprintln!("Error parsing DnsQuestion: {}", e);
            return;
        }
    };
    questions.push(question);

    let mut answers = Vec::new();
    let answer = DnsAnswer {
        resource_records: vec![ResourceRecord {
            name: questions[0].name.clone(),
            atype: Type::A,
            class: Class::IN,
            ttl: 60,
            rdata: RData::A {
                address: u32::from_be_bytes([8, 8, 8, 8]),
            },
        }],
    };
    answers.push(answer);

    let message = DnsMessage {
        header,
        questions,
        answers,
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
        match sock.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                println!("Received {} bytes from {}", len, addr);
                tx.send((buf[..len].to_vec(), addr)).await?;
            }
            Err(e) => eprintln!("Error receiving: {e}"),
        }
    }
}
