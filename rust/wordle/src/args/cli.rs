use clap::{AppSettings, Parser};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct Cli {
    /// (non-random mode only) Set the target word for the only round
    #[clap(short, long)]
    pub word: Option<String>,

    /// Enter the random mode
    #[clap(short, long)]
    pub random: bool,

    /// Enter the difficult mode
    #[clap(short = 'D', long)]
    pub difficult: bool,

    /// Display stats after each round
    #[clap(short = 't', long)]
    pub stats: bool,

    /// (random mode only) Set the day of the first round
    #[clap(short, long)]
    pub day: Option<usize>,

    /// (random mode only) Set the seed for shuffling target words
    #[clap(short, long)]
    pub seed: Option<u64>,

    /// Set the path to the file containing valid target words instead of using the default set
    #[clap(short, long)]
    pub final_set: Option<String>,

    /// Set the path to the file containing acceptable words instead of using the default set
    #[clap(short, long)]
    pub acceptable_set: Option<String>,

    /// (random mode only) Set the path to the file where the game state is saved
    #[clap(short = 'S', long)]
    pub state: Option<String>,

    /// Provide sharable guessing process after each game
    #[clap(long)]
    pub share: bool,

    /// Enable asking for possible answers and guess recommendations while guessing
    #[clap(long)]
    pub enable_solver: bool,

    /// Set path to the config file
    #[clap(short, long)]
    pub config: Option<String>,
}

/// Just a wrapper of clap::Parser::parse, but the caller doesn't need to use clap::Parser
pub fn parse_args() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This test is recommended by the
    /// [clap documentation](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html#testing)
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
