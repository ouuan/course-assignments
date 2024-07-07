use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use enigma::*;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Encrypt(EncryptArgs),
    Crack(CrackArgs),
}

#[derive(Args)]
pub struct EncryptArgs {
    /// Set rotor types from left to right, e.g. "123", "213"
    #[arg(long)]
    rotor_types: String,

    /// Set ring settings from left to right, e.g. "AAA", "AES"
    #[arg(long)]
    ring_settings: String,

    /// Set initial positions from left to right, e.g. "AAA", "ADU"
    #[arg(long)]
    initial_positions: String,

    /// Set plugboard pairs, separated by commas e.g. "BX,GK,WY"
    #[arg(long, value_delimiter = ',')]
    plugboard: Option<Vec<String>>,

    /// The plaintext to be encrypted. Read from stdin if not provided.
    pub plaintext: Option<String>,
}

impl EncryptArgs {
    pub fn build_enigma(&self) -> Result<Enigma> {
        let mut rotor_settings = Vec::new();
        let mut ring_settings = self.ring_settings.chars();
        let mut initial_positions = self.initial_positions.chars();
        for (i, rotor_type) in self.rotor_types.chars().enumerate() {
            let rotor_type = rotor_type.try_into().context("invalid rotor type")?;
            let ring_setting = ring_settings
                .next()
                .context("ring setting count less than rotor type count")?
                .try_into()
                .context("invalid ring setting")?;
            let initial_position = initial_positions
                .next()
                .context("initial position count less than rotor type count")?
                .try_into()
                .context("invalid initial position")?;
            let setting = RotorSetting {
                rotor_type,
                ring_setting,
                initial_position,
            };
            log::info!("rotor {}: {setting}", i + 1);
            rotor_settings.push(setting);
        }
        if ring_settings.next().is_some() {
            bail!("rotor type count less than ring setting count");
        }
        if initial_positions.next().is_some() {
            bail!("rotor type count less than initial position count");
        }

        let mut plugboard = Vec::new();
        if let Some(pairs) = &self.plugboard {
            for pair in pairs {
                if pair.len() != 2 {
                    bail!("plugboard must consist with pairs of letters separated by commas");
                }
                let mut chars = pair.chars();
                plugboard.push((
                    chars
                        .next()
                        .unwrap()
                        .try_into()
                        .context("invalid plugboard setting")?,
                    chars
                        .next()
                        .unwrap()
                        .try_into()
                        .context("invalid plugboard setting")?,
                ));
            }
        }

        Enigma::new(plugboard, rotor_settings)
    }
}

#[derive(Args)]
pub struct CrackArgs {
    /// Map from 1st character to 4th character
    p14: String,
    /// Map from 2nd character to 5th character
    p25: String,
    /// Map from 3rd character to 6th character
    p36: String,
}

impl CrackArgs {
    pub fn crack(&self) -> Result<Vec<Vec<RotorSetting>>> {
        enigma::crack(&self.p14, &self.p25, &self.p36)
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
