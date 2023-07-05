use anyhow::{anyhow, Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::{self, Write};
use std::sync::Mutex;
use wordle::{
    game::{self, GameState, Guess, GuessResult, LetterState},
    valid_word::ValidWord,
    word_set,
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Solve in difficult mode
    #[clap(short = 'D', long)]
    pub difficult: bool,

    /// Benchmark the speed and avg guess count over the whole final set with the given starting word
    #[clap(short, long)]
    pub benchmark_word: Option<String>,

    /// Benchmark all words in the acceptable set as the starting word
    #[clap(long)]
    pub benchmark_all: bool,

    /// Set the path to the file containing valid target words instead of using the default set
    #[clap(short, long)]
    pub final_set: Option<String>,

    /// Set the path to the file containing acceptable words instead of using the default set
    #[clap(short, long)]
    pub acceptable_set: Option<String>,
}

struct BenchmarkInfo {
    starting_word: String,
    success_count: usize,
    fail_count: usize,
    avg_guess: f64,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let acceptable_set = word_set::get_acceptable_set(&args.acceptable_set)?;
    let (final_set, final_list) =
        word_set::get_final_set(&args.final_set, &acceptable_set, false, None)?;

    if let Some(starting_word) = &args.benchmark_word {
        if args.benchmark_all {
            return Err(anyhow!(
                "cannot specify benchmark all and benchmark word at the same time"
            ));
        }

        let starting_word = starting_word.to_ascii_uppercase();

        ValidWord::new(
            &starting_word,
            "starting word",
            &acceptable_set,
            "acceptable",
        )?;

        benchmark(
            &starting_word,
            &final_set,
            &final_list,
            &acceptable_set,
            args.difficult,
            true,
        );

        return Ok(());
    }

    if args.benchmark_all {
        let mut finished = 0;

        let mut result = acceptable_set
            .iter()
            .map(|word| {
                let info = benchmark(
                    &word,
                    &final_set,
                    &final_list,
                    &acceptable_set,
                    args.difficult,
                    false,
                );
                finished += 1;
                eprintln!(
                    "{}: {}/{} @ {:.3} ({:.2}%)",
                    info.starting_word,
                    info.success_count,
                    info.fail_count,
                    info.avg_guess,
                    finished as f64 / acceptable_set.len() as f64 * 100.0
                );
                info
            })
            .collect::<Vec<_>>();

        result.sort_by(|lhs, rhs| match lhs.fail_count.cmp(&rhs.fail_count) {
            Ordering::Equal => lhs.avg_guess.total_cmp(&rhs.avg_guess),
            not_equal => not_equal,
        });

        for info in result {
            println!(
                "{}: {}/{} @ {:.3}",
                info.starting_word, info.success_count, info.fail_count, info.avg_guess
            );
        }

        return Ok(());
    }

    let mut game = game::Game::new(
        final_set
            .iter()
            .next()
            .expect("final set should not be empty"),
        &final_set,
        args.difficult,
    )?;

    loop {
        let word = loop {
            print!("Enter your guess, or leave it empty to let the solver guess: ");
            io::stdout().flush().context("failed to flush stdout")?;
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .context("failed to read guess")?;
            let input = String::from(input.trim());
            if input == "" {
                let guess = game.solve(&acceptable_set, true).best_guesses[0]
                    .word
                    .clone();
                println!("Guess from solver: {}", guess);
                break guess;
            }
            match ValidWord::new(&input, "guess", &acceptable_set, "acceptable") {
                Ok(_) => break input,
                Err(error) => println!("Guess is invalid: {}", error),
            }
        };

        let result: GuessResult = loop {
            print!("Enter the color result ([R/Y/G]): ");
            io::stdout().flush().context("failed to flush stdout")?;
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .context("failed to read color result")?;
            let input = input.trim().to_ascii_uppercase();
            if input.len() != wordle::LETTER_NUM_IN_WORD
                || input.chars().any(|c| c != 'R' && c != 'Y' && c != 'G')
            {
                println!("Invalid result. Should look like \"RYGRY\" where R means too many, Y means incorrect position, and G means correct.");
                continue;
            }
            break input
                .chars()
                .map(|c| match c {
                    'R' => LetterState::TooMany,
                    'Y' => LetterState::IncorrectPos,
                    'G' => LetterState::Correct,
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
        };

        unsafe {
            game.add_guess(Guess {
                word: ValidWord::new(&word, "", &acceptable_set, "").unwrap(),
                result,
            });
        }

        let possible_answers = game.possible_answers(&acceptable_set);

        match possible_answers.len() {
            0 => {
                return Err(anyhow!(
                    "zero possible answer remains, maybe caused by incorrect word set or incorrect input"
                ))
            }
            1 => {
                println!("The answer is {}", possible_answers[0]);
                break;
            }
            _ => (),
        }
    }

    Ok(())
}

fn benchmark(
    starting_word: &str,
    final_set: &HashSet<String>,
    final_list: &Vec<String>,
    acceptable_set: &HashSet<String>,
    difficult: bool,
    verbose: bool,
) -> BenchmarkInfo {
    let count_sum = Mutex::new(0);
    let success_count = Mutex::new(0);
    let fail_count = Mutex::new(0);

    const EMPTY_STRING_MUTEX: Mutex<String> = Mutex::new(String::new());
    let first_guess = [EMPTY_STRING_MUTEX; 3usize.pow(wordle::LETTER_NUM_IN_WORD as u32)];
    let second_guess = [EMPTY_STRING_MUTEX; 3usize.pow(wordle::LETTER_NUM_IN_WORD as u32 * 2)];

    let output = final_list
        .par_iter()
        .map(|answer| {
            let mut output = String::new();
            output += answer;
            output += ": ";
            let mut game = game::Game::new(answer, &final_set, difficult).unwrap();
            let mut state = game.guess(&starting_word, &acceptable_set).unwrap();
            output += &starting_word;
            loop {
                match state {
                    GameState::Success { guess_count } => {
                        *count_sum.lock().unwrap() += guess_count;
                        *success_count.lock().unwrap() += 1;
                        break;
                    }
                    GameState::Failed { .. } => {
                        *fail_count.lock().unwrap() += 1;
                        break;
                    }
                    GameState::InProgress => {}
                }
                let solve = || {
                    game.solve(&acceptable_set, false).best_guesses[0]
                        .word
                        .clone()
                };
                let guess = if game.guesses.len() == 1 {
                    let mut guess = first_guess[game::result_to_number(&game.guesses[0].result)]
                        .lock()
                        .unwrap();
                    if guess.is_empty() {
                        *guess = solve();
                    }
                    guess.clone()
                } else if game.guesses.len() == 2 {
                    let mut guess = second_guess[game::result_to_number(&game.guesses[0].result)
                        * first_guess.len()
                        + game::result_to_number(&game.guesses[1].result)]
                    .lock()
                    .unwrap();
                    if guess.is_empty() {
                        *guess = solve();
                    }
                    guess.clone()
                } else {
                    solve().clone()
                };
                state = game.guess(&guess, &acceptable_set).unwrap();
                output += " ";
                output += &guess;
            }

            if verbose {
                let count_sum = *count_sum.lock().unwrap();
                let success_count = *success_count.lock().unwrap();
                let fail_count = *fail_count.lock().unwrap();
                eprintln!(
                    "({:.2}%  avg: {:.2})",
                    (success_count + fail_count) as f64 / final_list.len() as f64 * 100.0,
                    count_sum as f64 / success_count as f64
                );
            }
            output
        })
        .collect::<Vec<_>>();

    if verbose {
        for line in output {
            println!("{}", line);
        }
    }

    let count_sum = *count_sum.lock().unwrap();
    let success_count = *success_count.lock().unwrap();
    let fail_count = *fail_count.lock().unwrap();
    let avg_guess = count_sum as f64 / success_count as f64;

    if verbose {
        println!("Success: {}", success_count);
        println!("Fail: {}", fail_count);
        println!("Avg guess: {:.3}", avg_guess);
    }

    BenchmarkInfo {
        starting_word: String::from(starting_word),
        success_count,
        fail_count,
        avg_guess,
    }
}
