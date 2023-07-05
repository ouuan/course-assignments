//! The UI used in tests
//!
//! Many non-basic UI behaviors are not implemented.

use super::*;
use std::io;

pub struct TestMode();

fn get_one_line() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(String::from(input.trim()))
}

fn state_to_char(state: &LetterState) -> char {
    match state {
        LetterState::Unknown => 'X',
        LetterState::TooMany => 'R',
        LetterState::IncorrectPos => 'Y',
        LetterState::Correct => 'G',
    }
}

impl UserInterface for TestMode {
    fn welcome(&self, _args: &Args) {}

    fn ask_for_target_word(&self) -> Result<String> {
        get_one_line()
    }

    fn get_guess(&self, _after_invalid: bool) -> Result<String> {
        get_one_line()
    }

    fn get_guess_or_solver(&self, _after_invalid: bool) -> Result<GuessOrSolver> {
        let input = get_one_line()?;
        Ok(GuessOrSolver::Guess(input))
    }

    fn show_possible_answers(&self, _possible_answers: Vec<String>) {}

    fn show_solver_result(&self, _solver_result: SolverResult) {}

    fn show_game(&self, game: &Game) {
        let last_result = game
            .guesses
            .last()
            .expect("UserInterface::show_game should not be called with no guess in the game")
            .result;
        let letter_state = game.letter_state;
        println!(
            "{} {}",
            last_result.iter().map(state_to_char).collect::<String>(),
            letter_state.iter().map(state_to_char).collect::<String>()
        )
    }

    fn show_game_state(&self, state: &GameState) {
        match state {
            GameState::Failed { answer } => println!("FAILED {}", answer),
            GameState::Success { guess_count } => println!("CORRECT {}", guess_count),
            GameState::InProgress => panic!(
                "UserInterface::show_game_state should not be called with GameState::InProgress"
            ),
        }
    }

    fn show_share(&self, _content: String) {}

    fn ask_if_next_round(&self) -> Result<bool> {
        let mut input = String::new();
        Ok(match io::stdin().read_line(&mut input) {
            Ok(0) => false,
            Ok(_) => input.trim() == "Y",
            Err(_) => false,
        })
    }

    fn show_invalid_target(&self, _error: Error) {
        println!("INVALID");
    }

    fn show_invalid_guess(&self, _error: Error) {
        println!("INVALID");
    }

    fn show_stat(&self, stat: Stat) {
        println!("{} {} {:.2}", stat.success, stat.fail, stat.avg);
        println!(
            "{}",
            stat.frequent
                .iter()
                .map(|(word, count)| format!("{} {}", word, count))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }

    fn on_new_round(&self) {}
}
