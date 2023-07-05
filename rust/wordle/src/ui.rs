//! Different user interfaces.

use crate::args::Args;
use crate::game::{Game, GameState, LetterState, SolverResult};
use crate::stat::Stat;
use anyhow::{Error, Result};

pub enum GuessOrSolver {
    Guess(String),
    PossibleAnswers,
    WithRecommendation,
}

/// Implement this trait to define the behavior of a UI.
pub trait UserInterface {
    /// Display a welcome message.
    fn welcome(&self, args: &Args);

    /// Ask for a target word. The validity is not checked.
    fn ask_for_target_word(&self) -> Result<String>;

    /// Ask for a guess. The validity is not checked.
    fn get_guess(&self, after_invalid: bool) -> Result<String>;

    /// Ask the user to either guess or use the solver.
    fn get_guess_or_solver(&self, after_invalid: bool) -> Result<GuessOrSolver>;

    /// Show all possible answers based on current guesses.
    fn show_possible_answers(&self, possible_answers: Vec<String>);

    /// Show the solver result based on current guesses.
    fn show_solver_result(&self, solver_result: SolverResult);

    /// Show the game after each guess.
    fn show_game(&self, game: &Game);

    /// Show the game state when game ends.
    ///
    /// The *state* parameter will not be `InProgress`.
    fn show_game_state(&self, state: &GameState);

    /// Show Wordle-like sharable guess process.
    fn show_share(&self, content: String);

    /// Ask the user whether to play another round.
    fn ask_if_next_round(&self) -> Result<bool>;

    /// Show that the target word provided by the user is invalid.
    fn show_invalid_target(&self, error: Error);

    /// Show that the guess provided by the user is invalid.
    fn show_invalid_guess(&self, error: Error);

    /// Show game stats.
    fn show_stat(&self, stat: Stat);

    /// This function is called on each new round except for the first round.
    fn on_new_round(&self);
}

pub mod interactive_mode;
pub mod test_mode;

#[cfg(feature = "gui")]
pub mod gui;
