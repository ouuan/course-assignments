//! Parse the arguments.

mod cli;
mod json;

use anyhow::{anyhow, Result};

/// The args for the game.
///
/// `word` can be `Some` only when `random` is `false`.
///
/// `day` and `seed` can be `Some` only when `random` is `true`.
#[readonly::make]
#[derive(Debug)]
pub struct Args {
    /// The word specified to be used as the answer in the only game.
    pub word: Option<String>,

    /// Whether in the random mode or not.
    pub random: bool,

    /// Whether in the difficult mode or not.
    pub difficult: bool,

    /// Whether to show game stats after each round.
    pub stat: bool,

    /// The day to start with in the random mode.
    pub day: Option<usize>,

    /// The seed used for random shuffling in the random mode.
    pub seed: Option<u64>,

    /// The path to the file containing valid target words
    pub final_set: Option<String>,

    /// The path to the file containing acceptable words
    pub acceptable_set: Option<String>,

    /// The path to store the states
    pub state: Option<String>,

    /// Provide sharable guessing process after each game
    pub share: bool,

    /// Enable asking for possible answers and guess recommendations while guessing
    pub enable_solver: bool,
}

/// Get args by parsing command line options and possibly the config file.
pub fn get_args() -> Result<Args> {
    let cli_args = cli::parse_args();

    let config = match &cli_args.config {
        Some(config_file) => json::parse_config_file(config_file)?,
        None => json::Config {
            word: None,
            random: None,
            difficult: None,
            stats: None,
            day: None,
            seed: None,
            final_set: None,
            acceptable_set: None,
            state: None,
            share: None,
            enable_solver: None,
        },
    };

    let args = Args {
        word: cli_args.word.or(config.word),
        random: cli_args.random || config.random.unwrap_or(false),
        difficult: cli_args.difficult || config.difficult.unwrap_or(false),
        stat: cli_args.stats || config.stats.unwrap_or(false),
        day: cli_args.day.or(config.day),
        seed: cli_args.seed.or(config.seed),
        final_set: cli_args.final_set.or(config.final_set),
        acceptable_set: cli_args.acceptable_set.or(config.acceptable_set),
        state: cli_args.state.or(config.state),
        share: cli_args.share || config.share.unwrap_or(false),
        enable_solver: cli_args.enable_solver || config.enable_solver.unwrap_or(false),
    };

    if args.random {
        if args.word.is_some() {
            return Err(anyhow!("cannot specify word in random mode"));
        }
    } else {
        if args.day.is_some() {
            return Err(anyhow!("cannot specify day without being in random mode"));
        }
        if args.seed.is_some() {
            return Err(anyhow!("cannot specify seed without being in random mode"));
        }
        if args.state.is_some() {
            return Err(anyhow!("cannot store state without being in random mode"));
        }
    }

    Ok(args)
}
