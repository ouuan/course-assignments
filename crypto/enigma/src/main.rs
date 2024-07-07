use anyhow::{Context, Result};
use clap::Parser;
use cli::Commands;

mod cli;

fn main() -> Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    let cli = cli::Cli::parse();

    match cli.command {
        Commands::Encrypt(args) => {
            let mut enigma = args.build_enigma()?;

            if let Some(plaintext) = args.plaintext {
                println!(
                    "{}",
                    enigma
                        .translate_str(&plaintext)
                        .context("encryption error")?
                );
            } else {
                let stdin = std::io::stdin();
                loop {
                    let mut buf = String::new();
                    stdin.read_line(&mut buf)?;
                    let input = buf.trim();
                    if input.is_empty() {
                        break;
                    }
                    match enigma.translate_str(input) {
                        Ok(ciphertext) => println!("{ciphertext}"),
                        Err(error) => log::error!("{error}"),
                    }
                }
            }
        }
        Commands::Crack(args) => {
            let results = args.crack()?;
            for (i, result) in results.iter().enumerate() {
                if results.len() > 1 {
                    println!("result {}:", i + 1);
                }
                for (j, rotor) in result.iter().enumerate() {
                    println!("rotor {}: {rotor}", j + 1);
                }
            }
        }
    }

    Ok(())
}
