//! Parse command-line options.

use clap::Parser;

/// The command-line options.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Set path to the config file
    #[clap(short, long)]
    pub config: String,

    /// Delete all data in the database on startup
    #[clap(short, long)]
    pub flush_data: bool,
}

/// Just a wrapper of clap::Parser::parse, but the caller doesn't need to use clap::Parser
pub fn parse_args() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// <https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html#testing>
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
