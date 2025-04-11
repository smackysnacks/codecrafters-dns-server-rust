#![warn(rust_2018_idioms)]

mod error;
mod message;

use message::{ByteSerialize, Class, DnsMessage, Opcode, RData, ResourceRecord, Type};

use std::{env, io::Cursor, net::SocketAddr, sync::Arc};

use tokio::{net::UdpSocket, sync::mpsc};

async fn handle_forward_query(
    sock: Arc<UdpSocket>,
    addr: SocketAddr,
    message: DnsMessage<'_>,
    resolver: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bufs = Vec::new();
    for _ in 0..message.header.question_count {
        bufs.push(Vec::with_capacity(128));
    }

    let mut answers = Vec::new();
    for (i, buf) in bufs.iter_mut().enumerate() {
        let mut header = message.header.clone();
        header.question_count = 1;
        let forward_message = DnsMessage {
            header,
            questions: vec![message.questions[i].clone()],
            records: vec![],
        };

        forward_message.serialize(buf)?;

        let forward_sock = UdpSocket::bind("0.0.0.0:2054").await?;
        forward_sock.connect(resolver).await?;
        forward_sock.send(buf).await?;

        buf.resize(128, 0);
        let len = forward_sock.recv(buf).await?;
        let reply = DnsMessage::try_parse(&mut Cursor::new(&buf[..len]))?;

        answers.extend(reply.records);
    }

    let mut header = message.header.clone();
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
    header.answer_record_count = answers.len() as u16;
    let reply_message = DnsMessage {
        header,
        questions: message.questions,
        records: answers,
    };
    let mut reply_buf = Vec::with_capacity(1024);
    reply_message.serialize(&mut reply_buf)?;
    sock.send_to(&reply_buf, addr).await?;

    Ok(())
}

async fn handle(
    sock: Arc<UdpSocket>,
    bytes: Vec<u8>,
    addr: SocketAddr,
    resolver: Option<&'static str>,
) {
    let mut bytes = Cursor::new(&*bytes);

    let mut message = match DnsMessage::try_parse(&mut bytes) {
        Ok(message) => message,
        Err(e) => {
            eprintln!("failed parsing packet as DnsMessage: {}", e);
            return;
        }
    };

    match resolver {
        Some(address) => {
            if let Err(e) = handle_forward_query(sock, addr, message, address).await {
                eprintln!("failed forwarding query: {}", e);
            }
        }

        None => {
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

            let mut records = Vec::new();
            for i in 0..message.header.question_count {
                records.push(ResourceRecord {
                    name: message.questions[i as usize].name.clone(),
                    atype: Type::A,
                    class: Class::IN,
                    ttl: 60,
                    rdata: RData::A {
                        address: u32::from_be_bytes([8, 8, 8, 8]),
                    },
                });
            }
            message.records = records;

            let mut buf = Vec::with_capacity(128);
            message.serialize(&mut buf).unwrap();
            match sock.send_to(&buf, &addr).await {
                Ok(len) => println!("Sent {} bytes to {}", len, addr),
                Err(e) => eprintln!("Error sending to {}: {}", addr, e),
            }
        }
    }
}

fn usage(program: &str) {
    println!("Usage: {program} [--resolver <address>]");
}

fn parse_args() -> Option<&'static str> {
    let mut resolver: Option<&'static str> = None;

    let program = env::args().next().unwrap();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                usage(&program);
                std::process::exit(0);
            }

            "--resolver" => match args.next() {
                Some(addr) => {
                    resolver = Some(addr.leak());
                }
                None => {
                    usage(&program);
                    std::process::exit(1);
                }
            },

            _ => {
                usage(&program);
                std::process::exit(1);
            }
        }
    }

    resolver
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = parse_args();

    let sock = UdpSocket::bind("127.0.0.1:2053").await?;
    let sock = Arc::new(sock);
    let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1000);

    tokio::spawn({
        let send_sock = sock.clone();
        async move {
            while let Some((bytes, addr)) = rx.recv().await {
                tokio::spawn(handle(send_sock.clone(), bytes, addr, resolver));
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
