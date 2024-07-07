mod cli;
mod client;
mod server;
mod session;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Server => server::run(cli),
        cli::Commands::Client => client::run(cli),
    }
}
