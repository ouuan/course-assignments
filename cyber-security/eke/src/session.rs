use aes::cipher::StreamCipher;
use aes::Aes128;
use ctr::Ctr128LE;
use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc;
use std::thread;

pub type Cipher = Ctr128LE<Aes128>;

pub struct BidirectionalCipher {
    pub send_cipher: Cipher,
    pub recv_cipher: Cipher,
}

enum Message {
    Send(String),
    SendEnd,
    Recv(Vec<u8>),
    RecvEnd,
}

pub fn session(stream: &mut TcpStream, mut cipher: BidirectionalCipher) {
    let (tx, rx) = mpsc::channel();

    let stdin_tx = tx.clone();
    let stream_tx = tx;

    let stdin_thread = thread::spawn(move || {
        let stdin = std::io::stdin();
        loop {
            let mut input = String::new();
            match stdin.read_line(&mut input) {
                Err(_) | Ok(0) => {
                    stdin_tx.send(Message::SendEnd).unwrap();
                    break;
                }
                _ => stdin_tx.send(Message::Send(input)).unwrap(),
            }
        }
    });

    let mut stream_rx = stream.try_clone().unwrap();
    let stream_thread = thread::spawn(move || {
        let mut buf = [0; 128];
        loop {
            match stream_rx.read(&mut buf) {
                Err(_) | Ok(0) => {
                    stream_tx.send(Message::RecvEnd).unwrap();
                    break;
                }
                Ok(size) => stream_tx.send(Message::Recv(buf[..size].to_vec())).unwrap(),
            }
        }
    });

    println!("Type and enter to send message, EOF to close session");

    while let Ok(message) = rx.recv() {
        match message {
            Message::Send(message) => {
                print!("Send: {message}");
                let mut buf = message.into_bytes();
                cipher.send_cipher.apply_keystream(&mut buf);
                if let Err(err) = stream.write_all(&buf) {
                    println!("Send Error: {err}");
                }
            }
            Message::SendEnd => {
                println!("Send End");
                stream.shutdown(Shutdown::Write).ok();
            }
            Message::Recv(mut buf) => {
                cipher.recv_cipher.apply_keystream(&mut buf);
                let message = String::from_utf8_lossy(&buf);
                for line in message.lines() {
                    println!("Recv: {line}");
                }
            }
            Message::RecvEnd => {
                println!("Recv End");
                stream.shutdown(Shutdown::Read).ok();
            }
        }
    }

    stdin_thread.join().ok();
    stream_thread.join().ok();

    println!("Session closed");
}
