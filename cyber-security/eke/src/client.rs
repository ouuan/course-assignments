use crate::cli::Cli;
use crate::session::{session, BidirectionalCipher, Cipher};
use aes::cipher::{
    generic_array::GenericArray, BlockSizeUser, IvSizeUser, KeyIvInit, KeySizeUser, StreamCipher,
};
use aes::{Aes128, Block};
use anyhow::{bail, Context, Result};
use rand::{thread_rng, RngCore};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use std::io::Write;
use std::net::{Shutdown, TcpStream};

pub fn run(cli: Cli) -> Result<()> {
    let mut stream =
        TcpStream::connect((cli.host, cli.port)).context("failed to establish TCP connection")?;
    println!("Connected to server");
    match handshake(&mut stream, cli) {
        Ok(cipher) => {
            println!("Handshake finished");
            session(&mut stream, cipher);
            Ok(())
        }
        Err(err) => {
            stream.shutdown(Shutdown::Both).ok();
            Err(err).context("handshake error")
        }
    }
}

fn handshake(stream: &mut TcpStream, cli: Cli) -> Result<BidirectionalCipher> {
    let mut rng = thread_rng();

    // init stream cipher with password
    let mut iv = GenericArray::default();
    rng.fill_bytes(&mut iv);
    let mut e0 = Cipher::new(&cli.pw, &iv);

    // generate RSA key pair
    let ska = RsaPrivateKey::new(&mut rng, 2048)?;
    let pka = RsaPublicKey::from(&ska);

    // first message
    let mut se_pka = bincode::serialize(&pka)?;
    e0.apply_keystream(&mut se_pka);
    let e0_se_pka = se_pka;
    stream.write_all(&bincode::serialize(&("A", iv.as_slice(), e0_se_pka))?)?;

    // second message
    let mut e0_e_ks: Vec<u8> =
        bincode::deserialize_from(&*stream).context("invalid received message")?;
    e0.apply_keystream(&mut e0_e_ks);
    let e_ks = e0_e_ks;
    let ks = ska
        .decrypt(Pkcs1v15Encrypt, &e_ks)
        .context("failed to decrypt session key received from server")?;
    if ks.len() != Aes128::key_size() {
        bail!("incorrect session key length received from server");
    }

    // init session send cipher
    let ks = GenericArray::from_slice(&ks);
    let mut iv = GenericArray::default();
    rng.fill_bytes(&mut iv);
    let mut e1_send = Cipher::new(ks, &iv);

    // generate nonce
    let mut na = Block::default();
    rng.fill_bytes(&mut na);

    // third message
    let mut e1_na = na;
    e1_send.apply_keystream(&mut e1_na);
    stream.write_all(&bincode::serialize(&(iv.as_slice(), e1_na.as_slice()))?)?;

    // fourth message
    let (iv, mut e1_na_nb): (Vec<u8>, Vec<u8>) =
        bincode::deserialize_from(&*stream).context("invalid received message")?;
    if iv.len() != Cipher::iv_size() {
        bail!("incorrect IV length received from server");
    }
    if e1_na_nb.len() != Aes128::block_size() * 2 {
        bail!("incorrect double nonce length received from server");
    }

    // init session recv cipher
    let mut e1_recv = Cipher::new(ks, GenericArray::from_slice(&iv));
    e1_recv.apply_keystream(&mut e1_na_nb);

    // decrypt and validate nonce
    let na_nb = e1_na_nb;
    let (na_from_server, nb) = na_nb.split_at(Aes128::block_size());
    if na_from_server != na.as_slice() {
        bail!("incorrect client nonce received from server");
    }

    // fifth message
    let mut e1_nb = nb.to_vec();
    e1_send.apply_keystream(&mut e1_nb);
    stream.write_all(&bincode::serialize(&e1_nb)?)?;

    Ok(BidirectionalCipher {
        send_cipher: e1_send,
        recv_cipher: e1_recv,
    })
}
