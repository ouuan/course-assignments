use crate::cli::Cli;
use crate::session::{session, BidirectionalCipher, Cipher};
use aes::cipher::{
    generic_array::GenericArray, BlockSizeUser, IvSizeUser, KeyIvInit, StreamCipher,
};
use aes::{Aes128, Block};
use anyhow::{bail, Context, Result};
use rand::{thread_rng, RngCore};
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};
use std::io::Write;
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};

pub fn run(cli: Cli) -> Result<()> {
    let addr: SocketAddr = (cli.host, cli.port).into();
    let listener = TcpListener::bind(addr)?;
    println!("Listening on {addr}");

    for stream in listener.incoming() {
        let mut stream = stream?;
        println!("Connected to client");
        match handshake(&mut stream, &cli) {
            Ok(cipher) => {
                println!("Handshake finished");
                session(&mut stream, cipher);
            }
            Err(err) => {
                stream.shutdown(Shutdown::Both).ok();
                println!("handshake error: {err}");
            }
        }
    }

    Ok(())
}

fn handshake(stream: &mut TcpStream, cli: &Cli) -> Result<BidirectionalCipher> {
    let mut rng = thread_rng();

    // first message
    let (name, iv, e0_se_pka): (String, Vec<u8>, Vec<u8>) =
        bincode::deserialize_from(&*stream).context("invalid received message")?;
    if name != "A" {
        bail!("invalid client name");
    }
    if iv.len() != Cipher::iv_size() {
        bail!("invalid IV size received from client");
    }

    // init stream cipher with password
    let mut e0 = Cipher::new(&cli.pw, GenericArray::from_slice(&iv));

    // decrypt pka
    let mut se_pka = e0_se_pka;
    e0.apply_keystream(&mut se_pka);
    let pka: RsaPublicKey = bincode::deserialize(&se_pka)
        .context("failed to deserialize RSA public key received from client")?;

    // generate session key
    let mut ks = GenericArray::default();
    rng.fill_bytes(&mut ks);

    // second message
    let e_ks = pka.encrypt(&mut rng, Pkcs1v15Encrypt, ks.as_slice())?;
    let mut e0_e_ks = e_ks;
    e0.apply_keystream(&mut e0_e_ks);
    stream.write_all(&bincode::serialize(&e0_e_ks)?)?;

    // third message
    let (iv, e1_na): (Vec<u8>, Vec<u8>) =
        bincode::deserialize_from(&*stream).context("invalid received message")?;
    if iv.len() != Cipher::iv_size() {
        bail!("invalid IV size received from client");
    }
    if e1_na.len() != Aes128::block_size() {
        bail!("invalid client nonce size received from client");
    }

    // init session ciphers
    let mut e1_recv = Cipher::new(&ks, GenericArray::from_slice(&iv));
    let mut iv = GenericArray::default();
    rng.fill_bytes(&mut iv);
    let mut e1_send = Cipher::new(&ks, &iv);

    // decrypt na
    let mut na = e1_na;
    e1_recv.apply_keystream(&mut na);

    // generate nb
    let mut nb = Block::default();
    rng.fill_bytes(&mut nb);

    // fourth message
    let mut na_nb = na;
    na_nb.extend_from_slice(&nb);
    let mut e1_na_nb = na_nb;
    e1_send.apply_keystream(&mut e1_na_nb);
    stream.write_all(&bincode::serialize(&(iv.as_slice(), e1_na_nb))?)?;

    // fifth message
    let e1_nb: Vec<u8> = bincode::deserialize_from(&*stream).context("invalid received message")?;
    let mut nb_from_client = e1_nb;
    e1_recv.apply_keystream(&mut nb_from_client);
    if nb_from_client != nb.as_slice() {
        bail!("incorrect server nonce received from client");
    }

    Ok(BidirectionalCipher {
        send_cipher: e1_send,
        recv_cipher: e1_recv,
    })
}
