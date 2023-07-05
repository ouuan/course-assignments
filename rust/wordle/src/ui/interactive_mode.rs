//! A user-friendly interactive CLI UI

use super::*;
use anyhow::Context;
use colored::{Color, Colorize};
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;

pub enum ReadResult {
    Line(String),
    Eof,
}

pub struct InteractiveMode {
    stdin_rx: Receiver<ReadResult>,
}

impl InteractiveMode {
    pub fn new() -> Self {
        // https://stackoverflow.com/a/55201400
        let (tx, rx) = mpsc::channel::<ReadResult>();
        thread::spawn(move || loop {
            let mut input = String::new();
            match io::stdin().read_line(&mut input).unwrap() {
                0 => tx.send(ReadResult::Eof).ok(),
                _ => tx.send(ReadResult::Line(String::from(input.trim()))).ok(),
            };
        });
        InteractiveMode { stdin_rx: rx }
    }

    fn get_one_line(&self, prompt: &str, context: &'static str) -> Result<String> {
        if atty::is(atty::Stream::Stdin) {
            // drop existing inputs before asking for input, only when input is a tty
            self.stdin_rx.try_iter().count();
        }
        print!("{}", prompt);
        io::stdout().flush().context(context)?;
        let line = match self.stdin_rx.recv().context(context)? {
            ReadResult::Line(line) => line,
            ReadResult::Eof => std::process::exit(0),
        };
        if atty::isnt(atty::Stream::Stdin) {
            // simulate the input when stdin is not a tty
            println!("{}", line);
        }
        Ok(line)
    }
}

fn supports_truecolor() -> bool {
    // check the COLORTERM env according to the docs of the colored crate
    // https://crates.io/crates/colored#truecolors
    match std::env::var("COLORTERM") {
        Ok(result) if (result == "truecolor" || result == "24bit") => true,
        _ => false,
    }
}

fn state_to_color(state: &LetterState) -> Color {
    if supports_truecolor() {
        match state {
            // the colors are copied from the official Wordle in dark theme
            LetterState::Unknown => Color::TrueColor {
                r: 129,
                g: 131,
                b: 132,
            },
            LetterState::TooMany => Color::TrueColor {
                r: 58,
                g: 58,
                b: 60,
            },
            LetterState::IncorrectPos => Color::TrueColor {
                r: 181,
                g: 159,
                b: 59,
            },
            LetterState::Correct => Color::TrueColor {
                r: 83,
                g: 141,
                b: 78,
            },
        }
    } else {
        match state {
            LetterState::Unknown => Color::Black,
            LetterState::TooMany => Color::Red,
            LetterState::IncorrectPos => Color::Yellow,
            LetterState::Correct => Color::Green,
        }
    }
}

fn white() -> Color {
    if supports_truecolor() {
        Color::TrueColor {
            r: 255,
            g: 255,
            b: 255,
        }
    } else {
        Color::White
    }
}

impl UserInterface for InteractiveMode {
    fn welcome(&self, args: &Args) {
        println!("Welcome to {}!\n", "WORDLE".bold());
        if let Some(day) = args.day {
            println!("Today is Wordle {}\n", format!("#{}", day).bold());
        }
    }

    fn ask_for_target_word(&self) -> Result<String> {
        self.get_one_line(
            "Please enter the answer for the puzzle: ",
            "failed to read the target word",
        )
    }

    fn get_guess(&self, after_invalid: bool) -> Result<String> {
        let prompt = if after_invalid {
            "Please guess again: "
        } else {
            "Guess: "
        };
        self.get_one_line(prompt, "failed to read the guess")
    }

    fn get_guess_or_solver(&self, after_invalid: bool) -> Result<GuessOrSolver> {
        let prompt = if after_invalid {
            "Enter a word to guess, enter \"!\" to show all remaining possible answers, or enter \"?\" to get guess recommendation: "
        } else {
            "Guess/\"!\"/\"?\": "
        };
        let res = self.get_one_line(prompt, "failed to get guess or solver")?;
        Ok(if res == "!" {
            GuessOrSolver::PossibleAnswers
        } else if res == "?" {
            GuessOrSolver::WithRecommendation
        } else {
            GuessOrSolver::Guess(res)
        })
    }

    fn show_possible_answers(&self, possible_answers: Vec<String>) {
        let len = possible_answers.len();
        if len == 1 {
            println!(
                "\nThe only remaining possible answer is {}.",
                possible_answers[0].bold()
            );
        } else {
            print!("\nThere are {} remaining possible answers:", len);
            if len > 5 {
                println!("\n-------------------------------------------------------------");
            }
            for chunk in possible_answers.chunks(10) {
                print!(" ");
                for word in chunk {
                    print!("{} ", word.bold());
                }
                println!("");
            }
            if len > 5 {
                println!("-------------------------------------------------------------");
            }
        }
        println!("")
    }

    fn show_solver_result(&self, solver_result: SolverResult) {
        let len = solver_result.possible_answers.len();
        if len == 1 {
            self.show_possible_answers(solver_result.possible_answers);
        } else {
            println!("\nThere are {} remaining possible answers.", len);
            println!("Among them, the recommended guesses are:");
            for item in solver_result.best_possibly_answer_guesses.iter().take(5) {
                println!(" * {} (score: {:.2})", item.word.bold(), item.score);
            }
            if solver_result.best_guesses[0].word
                != solver_result.best_possibly_answer_guesses[0].word
            {
                println!("However, if words that are impossible to be the answer are considered, the recommended guesses will be:");
                for item in solver_result.best_guesses.iter().take(5) {
                    println!(" * {} (score: {:.2})", item.word.bold(), item.score);
                }
            }
            println!("");
        }
    }

    fn show_game(&self, game: &Game) {
        // previous guess results
        println!("  ┌{}┐", "─".repeat(crate::LETTER_NUM_IN_WORD));
        for guess in &game.guesses {
            print!("  │");
            for (letter, state) in guess.word.iter().zip(guess.result.iter()) {
                print!(
                    "{}",
                    String::from(*letter)
                        .color(white())
                        .on_color(state_to_color(state))
                );
            }
            println!("│");
        }
        for _ in game.guesses.len()..crate::MAX_GUESS_NUM {
            println!("  │{}│", ".".repeat(crate::LETTER_NUM_IN_WORD));
        }
        println!("  └{}┘", "─".repeat(crate::LETTER_NUM_IN_WORD));

        // letter states
        static LETTERS_ON_KEYBOARD: [[usize; 10]; 3] = [
            [16, 22, 4, 17, 19, 24, 20, 8, 14, 15],
            [26, 0, 18, 3, 5, 6, 7, 9, 10, 11],
            [26, 26, 25, 23, 2, 21, 1, 13, 12, 26],
        ];
        for line in LETTERS_ON_KEYBOARD {
            for letter in line {
                match letter {
                    26 => print!(" "),
                    c => print!(
                        "{}",
                        String::from(char::from_u32(b'A' as u32 + c as u32).unwrap())
                            .color(white())
                            .on_color(state_to_color(&game.letter_state[c]))
                    ),
                }
            }
            println!("");
        }
        println!("");
    }

    fn show_game_state(&self, state: &GameState) {
        match state {
            GameState::InProgress => println!("Game in progress..."),
            GameState::Failed { answer } => println!(
                "\nOops! You {}. The answer is {}.",
                "failed".red(),
                answer.bold()
            ),
            GameState::Success { guess_count } => println!(
                "Congratulations! You {} in {} guess{}.",
                "succeeded".green(),
                guess_count.to_string().bold(),
                if *guess_count > 1 { "es" } else { "" },
            ),
        }
    }

    fn show_share(&self, content: String) {
        println!("\n{}", content);
    }

    fn ask_if_next_round(&self) -> Result<bool> {
        let mut invalid_input = false;
        loop {
            let prompt = if invalid_input {
                "Enter y to continue, or enter n to exit. [Y/n] "
            } else {
                "Continue? [Y/n] "
            };
            let res = self.get_one_line(prompt, "failed to read continue or not")?;
            match res.to_ascii_lowercase().as_str() {
                "" | "y" | "yes" => break Ok(true),
                "n" | "no" => break Ok(false),
                _ => {
                    invalid_input = true;
                    continue;
                }
            }
        }
    }

    fn show_invalid_target(&self, error: Error) {
        println!("The answer is invalid: {}\n", error);
    }

    fn show_invalid_guess(&self, error: Error) {
        println!("The guess is invalid: {}\n", error);
    }

    fn show_stat(&self, stat: Stat) {
        println!("---------------------------");
        println!(" Success: {}", stat.success.to_string().bold());
        println!(" Failure: {}", stat.fail.to_string().bold());
        println!(
            " Avg guess on success: {}",
            format!("{:.2}", stat.avg).bold()
        );
        println!(
            " Most used word{}:",
            if stat.frequent.len() > 1 { "s" } else { "" },
        );
        for (word, count) in &stat.frequent {
            println!(
                " *   {} - {} time{}",
                word.bold(),
                count,
                if *count > 1 { "s" } else { "" }
            );
        }
        println!("---------------------------");
    }

    fn on_new_round(&self) {
        println!("");
    }
}
