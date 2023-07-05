pub mod args;
pub mod game;
pub mod json;
pub mod stat;
pub mod ui;
pub mod valid_word;
pub mod word_set;

use anyhow::{anyhow, Context, Result};
use game::LetterState;
use stat::*;
use std::fs;
use std::io::ErrorKind;

pub const LETTER_NUM_IN_WORD: usize = 5;
pub const MAX_GUESS_NUM: usize = 6;

/// The main logic of the whole game process
pub fn run(args: &args::Args, ui: &impl ui::UserInterface) -> Result<()> {
    let acceptable_set = word_set::get_acceptable_set(&args.acceptable_set)?;
    let (final_set, final_list) =
        word_set::get_final_set(&args.final_set, &acceptable_set, args.random, args.seed)?;

    // read or create the state
    let mut state = match &args.state {
        Some(path) => match fs::read_to_string(path) {
            Ok(content) => {
                json::parse_json::<StateFile>(&content, &format!("the state file [{}]", path))?
                    .into()
            }
            Err(err) => match err.kind() {
                ErrorKind::NotFound => State::new(),
                _ => return Err(err).context(format!("failed to read the state file [{}]", path)),
            },
        },
        None => State::new(),
    };

    ui.welcome(&args);

    let mut day = args.day.unwrap_or(1);

    // games
    loop {
        let target = if let Some(word) = &args.word {
            word.clone()
        } else if args.random {
            if day > final_list.len() {
                return Err(anyhow!(
                    "the day {} exceeds the size of the target word set",
                    day
                ));
            }
            final_list[day - 1].clone()
        } else {
            loop {
                let word = ui.ask_for_target_word()?;
                match valid_word::ValidWord::new(&word, "answer", &final_set, "valid answer") {
                    Ok(_) => break word,
                    Err(error) => {
                        ui.show_invalid_target(error);
                        continue;
                    }
                }
            }
        };

        let mut game = game::Game::new(&target, &final_set, args.difficult)?;
        let mut after_invalid = false;

        loop {
            let guess = if args.enable_solver {
                match ui.get_guess_or_solver(after_invalid)? {
                    ui::GuessOrSolver::PossibleAnswers => {
                        ui.show_possible_answers(game.possible_answers(&acceptable_set));
                        continue;
                    }
                    ui::GuessOrSolver::WithRecommendation => {
                        ui.show_solver_result(game.solve(&acceptable_set, true));
                        continue;
                    }
                    ui::GuessOrSolver::Guess(word) => word,
                }
            } else {
                ui.get_guess(after_invalid)?
            };
            let state = match game.guess(&guess, &acceptable_set) {
                Ok(state) => {
                    after_invalid = false;
                    state
                }
                Err(error) => {
                    ui.show_invalid_guess(error);
                    after_invalid = true;
                    continue;
                }
            };
            ui.show_game(&game);
            match state {
                game::GameState::InProgress => continue,
                _ => {
                    ui.show_game_state(&state);
                    if args.share {
                        let mut share = format!(
                            "Wordle {} {}/{}{}{}\n",
                            if args.random && args.final_set.is_none() {
                                match args.seed {
                                    Some(seed) => format!("{} ({})", day, seed),
                                    None => day.to_string(),
                                }
                            } else {
                                String::from("(custom)")
                            },
                            match state {
                                game::GameState::Success { guess_count } => guess_count.to_string(),
                                _ => String::from("X"),
                            },
                            crate::MAX_GUESS_NUM,
                            if args.difficult { "*" } else { "" },
                            if args.enable_solver { "?" } else { "" },
                        );
                        for guess in &game.guesses {
                            for state in guess.result {
                                share.push(match state {
                                    LetterState::Unknown | LetterState::TooMany => '⬜',
                                    LetterState::IncorrectPos => '🟨',
                                    LetterState::Correct => '🟩',
                                });
                            }
                            share.push('\n');
                        }
                        ui.show_share(share);
                    }
                    break;
                }
            }
        }

        let (answer, guesses) = game.into_game_info();
        state.push(answer, guesses);

        if let Some(path) = &args.state {
            let json = serde_json::to_string_pretty(&StateFile::from(&state))
                .context("failed to serialize the final state")?;
            fs::write(path, json).with_context(|| format!("failed to save state to [{}]", path))?;
        }

        if args.stat {
            ui.show_stat(Stat::from(&state));
        }

        if args.word.is_some() {
            break;
        }

        if !ui.ask_if_next_round()? {
            break;
        }

        day += 1;

        ui.on_new_round();
    }

    Ok(())
}
