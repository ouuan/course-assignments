use aes::cipher::Key;
use aes::Aes128;
use anyhow::Result;
use clap::{Parser, Subcommand};
use hkdf::Hkdf;
use sha2::Sha256;
use std::net::IpAddr;

const HKDF_INFO: &[u8] = b"Bellovin-Merritt EKE - Cybersecurity Fundamentals";

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// pre-shared password
    #[arg(long, value_parser = parse_password)]
    pub pw: Key<Aes128>,

    /// server host
    #[arg(long, default_value = "127.0.0.1")]
    pub host: IpAddr,

    /// server port
    #[arg(long, default_value_t = 3393)]
    pub port: u16,
}

#[derive(Subcommand)]
pub enum Commands {
    Server,
    Client,
}

fn parse_password(pw: &str) -> Result<Key<Aes128>> {
    let hk = Hkdf::<Sha256>::new(None, pw.as_bytes());
    let mut key = Key::<Aes128>::default();
    hk.expand(HKDF_INFO, &mut key).unwrap();
    Ok(key)
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
