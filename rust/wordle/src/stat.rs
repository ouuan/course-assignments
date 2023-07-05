use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::HashMap;

const WORD_NUM_IN_FREQUENT: usize = 5;

#[derive(Serialize, Deserialize)]
struct GameInFile {
    answer: Option<String>,
    guesses: Option<Vec<String>>,
}

#[derive(Clone)]
struct Game {
    answer: String,
    guesses: Vec<String>,
}

/// used at runtime
pub struct State {
    games: Vec<Game>,
    word_count: HashMap<String, usize>,
    success: usize,
    successful_guess_count: usize,
}

/// used when reading from / writing to the state file
#[derive(Serialize, Deserialize)]
pub struct StateFile {
    total_rounds: Option<usize>,
    games: Option<Vec<GameInFile>>,
}

impl State {
    pub fn new() -> Self {
        State {
            games: Vec::new(),
            word_count: HashMap::new(),
            success: 0,
            successful_guess_count: 0,
        }
    }

    pub fn push(&mut self, answer: String, guesses: Vec<String>) {
        for guess in &guesses {
            *self.word_count.entry(guess.clone()).or_insert(0) += 1;
        }
        if guesses.last() == Some(&answer) {
            self.success += 1;
            self.successful_guess_count += guesses.len();
        }
        self.games.push(Game { answer, guesses });
    }
}

impl From<StateFile> for State {
    fn from(state: StateFile) -> Self {
        let mut word_count = HashMap::new();
        let mut success = 0;
        let mut successful_guess_count = 0;
        let mut valid_games = Vec::new();

        // ignore total_rounds and only use valid games
        if let Some(games) = state.games {
            for game in games {
                if let Some(guesses) = game.guesses {
                    if let Some(answer) = game.answer {
                        for guess in &guesses {
                            *word_count.entry(guess.clone()).or_insert(0) += 1;
                        }
                        if guesses.last() == Some(&answer) {
                            success += 1;
                            successful_guess_count += guesses.len();
                        }
                        valid_games.push(Game { guesses, answer });
                    }
                }
            }
        }

        State {
            games: valid_games,
            word_count,
            success,
            successful_guess_count,
        }
    }
}

impl From<&State> for StateFile {
    fn from(state: &State) -> Self {
        StateFile {
            total_rounds: Some(state.games.len()),
            games: Some(
                state
                    .games
                    .iter()
                    .map(|game| GameInFile {
                        answer: Some(game.answer.clone()),
                        guesses: Some(game.guesses.clone()),
                    })
                    .collect(),
            ),
        }
    }
}

/// used when displaying the stats (-t, --stats)
#[readonly::make]
pub struct Stat {
    pub success: usize,
    pub fail: usize,
    pub avg: f64,
    pub frequent: Vec<(String, usize)>,
}

impl From<&State> for Stat {
    fn from(state: &State) -> Self {
        let mut frequent = state
            .word_count
            .iter()
            .map(|(word, count)| (word.clone(), *count))
            .collect::<Vec<_>>();
        frequent.sort_by_key(|(word, count)| (Reverse(*count), word.clone()));
        frequent.truncate(WORD_NUM_IN_FREQUENT);

        Stat {
            success: state.success,
            fail: state.games.len() - state.success,
            avg: if state.success > 0 {
                state.successful_guess_count as f64 / state.success as f64
            } else {
                0.0
            },
            frequent,
        }
    }
}
