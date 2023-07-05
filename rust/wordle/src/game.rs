//! The core logic of a guessing round. It does not contain any inputs or outputs.

use crate::valid_word::ValidWord;
use crate::{LETTER_NUM_IN_WORD, MAX_GUESS_NUM};
use anyhow::{anyhow, Result};
use ordinal::Ordinal;
use rayon::prelude::*;
use std::collections::HashSet;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum LetterState {
    // the order must not change, because they are being compared
    Unknown = -1,
    TooMany = 0,
    IncorrectPos = 1,
    Correct = 2,
}

impl Default for LetterState {
    fn default() -> Self {
        Self::Unknown
    }
}

pub type GuessResult = [LetterState; LETTER_NUM_IN_WORD];

pub fn result_to_number(result: &GuessResult) -> usize {
    let mut number = 0;
    for r in result {
        number = number * 3 + *r as usize;
    }
    number
}

/// the word and the result for each letter in a guess
#[derive(Debug, PartialEq, Clone)]
pub struct Guess {
    pub word: ValidWord,
    pub result: GuessResult,
}

#[derive(Debug, PartialEq)]
pub enum GameState {
    InProgress,
    Success { guess_count: usize },
    Failed { answer: String },
}

/// The core struct which represents a game.
#[readonly::make]
#[derive(Clone, Debug)]
pub struct Game {
    difficult: bool,
    target: ValidWord,
    pub guesses: Vec<Guess>,
    pub letter_state: [LetterState; 26],
}

#[readonly::make]
#[derive(Clone, Debug)]
pub struct SolverResultItem {
    pub word: String,
    pub score: f64,
}

#[derive(Debug)]
pub struct SolverResult {
    pub best_possibly_answer_guesses: Vec<SolverResultItem>,
    pub best_guesses: Vec<SolverResultItem>,
    pub possible_answers: Vec<String>,
}

impl Game {
    pub fn new(target: &str, target_word_set: &HashSet<String>, difficult: bool) -> Result<Self> {
        Ok(Self {
            difficult,
            target: ValidWord::new(target, "answer", target_word_set, "target")?,
            guesses: Vec::new(),
            letter_state: [LetterState::Unknown; 26],
        })
    }

    /// Apply a guess. Returns an error if the guess is invalid.
    /// Returns the game state (success/failed/in progress) after the guess if the guess is valid.
    /// When the guess is valid, it records the guess in `self.guesses` and update the letter states.
    pub fn guess(
        &mut self,
        word: &str,
        acceptable_word_set: &HashSet<String>,
    ) -> Result<GameState> {
        let word = ValidWord::new(word, "guess", acceptable_word_set, "acceptable")?;

        let result = Self::guess_result(
            &word,
            &self.target,
            if self.difficult {
                self.guesses.last()
            } else {
                None
            },
        )?;

        let correct = result.iter().all(|s| *s == LetterState::Correct);

        // update letter states
        for i in 0..LETTER_NUM_IN_WORD {
            let state = &mut self.letter_state[(word[i] as u32 - 'A' as u32) as usize];
            if result[i] > *state {
                *state = result[i];
            }
        }

        self.guesses.push(Guess { word, result });

        Ok(if correct {
            GameState::Success {
                guess_count: self.guesses.len(),
            }
        } else {
            if self.guesses.len() >= MAX_GUESS_NUM {
                GameState::Failed {
                    answer: self.target.to_string(),
                }
            } else {
                GameState::InProgress
            }
        })
    }

    fn valid_in_difficult_mode(word: &ValidWord, last_guess: Option<&Guess>) -> Result<()> {
        if let Some(last_guess) = last_guess {
            for i in 0..LETTER_NUM_IN_WORD {
                if last_guess.result[i] == LetterState::Correct && word[i] != last_guess.word[i] {
                    return Err(anyhow!(
                        "{} letter must be {} in difficult mode",
                        Ordinal(i + 1),
                        last_guess.word[i]
                    ));
                }
            }

            let mut used = [false; LETTER_NUM_IN_WORD];
            for i in 0..LETTER_NUM_IN_WORD {
                if last_guess.result[i] != LetterState::TooMany {
                    let mut ok = false;
                    for j in 0..LETTER_NUM_IN_WORD {
                        if word[j] == last_guess.word[i] && !used[j] {
                            ok = true;
                            used[j] = true;
                            break;
                        }
                    }
                    if !ok {
                        return Err(anyhow!(
                            "letter {} must be used in difficult mode",
                            last_guess.word[i]
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Used in solver for better performance
    fn guess_result(
        word: &ValidWord,
        target: &ValidWord,
        last_guess_if_difficult: Option<&Guess>,
    ) -> Result<GuessResult> {
        Self::valid_in_difficult_mode(word, last_guess_if_difficult)?;

        let mut result = [LetterState::TooMany; LETTER_NUM_IN_WORD];

        // get Correct results
        for i in 0..LETTER_NUM_IN_WORD {
            if word[i] == target[i] {
                result[i] = LetterState::Correct;
            }
        }

        // get IncorrectPos results
        for i in 0..LETTER_NUM_IN_WORD {
            if target[i] != word[i] {
                for j in 0..LETTER_NUM_IN_WORD {
                    if target[i] == word[j] && result[j] == LetterState::TooMany {
                        result[j] = LetterState::IncorrectPos;
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    /// returns the tuple (answer, guesses), can be used in `crate::stat::State`
    pub fn into_game_info(self) -> (String, Vec<String>) {
        (
            self.target.to_string(),
            self.guesses
                .into_iter()
                .map(|guess| guess.word.to_string())
                .collect(),
        )
    }

    pub fn possible_answers(&self, word_set: &HashSet<String>) -> Vec<String> {
        let mut possible_answers = Vec::<String>::new();
        for word in word_set {
            let mut tmp_game = Self::new(word, word_set, false).unwrap();
            let mut possible = true;
            for guess in &self.guesses {
                tmp_game.guess(&guess.word.to_string(), word_set).unwrap();
                if tmp_game.guesses.last() != Some(guess) {
                    possible = false;
                    break;
                }
            }
            if possible {
                possible_answers.push(word.clone());
            }
        }
        possible_answers.sort();
        possible_answers
    }

    /// Get the solve result for `guess`.
    /// The guess is not checked to be valid.
    fn get_solve_result(
        &self,
        guess: &String,
        possible_set: &HashSet<String>,
    ) -> Option<SolverResultItem> {
        let valid_guess = unsafe { ValidWord::new_unchecked(&guess) };

        if self.difficult {
            if Self::valid_in_difficult_mode(&valid_guess, self.guesses.last()).is_err() {
                return None;
            }
        }

        // get the count of each color pattern
        let mut pattern_count = [0; 3usize.pow(LETTER_NUM_IN_WORD as u32)];
        for target in possible_set {
            let result = Self::guess_result(
                &valid_guess,
                unsafe { &ValidWord::new_unchecked(target) },
                None,
            )
            .unwrap();
            pattern_count[result_to_number(&result)] += 1;
        }

        let mut score = match possible_set.get(guess) {
            Some(_) => 0.01, // add 0.01 to the score if the guess is possibly the answer
            None => 0.0,
        };
        // calculate the entropy based on the pattern counts
        for count in pattern_count {
            if count == 0 {
                continue;
            }
            let p = count as f64 / possible_set.len() as f64;
            score -= p * p.log2();
        }

        Some(SolverResultItem {
            word: guess.clone(),
            score,
        })
    }

    /// Solve the puzzle based on current guesses and the acceptable word set.
    ///
    /// Set `parallel` to false if it's called with a Mutex held.
    /// See: <https://github.com/rayon-rs/rayon/issues/592>
    pub fn solve(&self, word_set: &HashSet<String>, parallel: bool) -> SolverResult {
        let possible_answers = self.possible_answers(word_set);

        let possible_set = possible_answers.iter().cloned().collect::<HashSet<_>>();

        let filter_map_fn = |guess| self.get_solve_result(guess, &possible_set);

        let mut best_guesses: Vec<_> = if parallel {
            word_set.par_iter().filter_map(filter_map_fn).collect()
        } else {
            word_set.iter().filter_map(filter_map_fn).collect()
        };

        best_guesses.sort_by(|lhs, rhs| rhs.score.total_cmp(&lhs.score));
        let best_possibly_answer_guesses = best_guesses
            .iter()
            .filter(|&item| possible_set.get(&item.word).is_some())
            .cloned()
            .collect::<Vec<_>>();

        // If only one chance remains, consider possible answers only
        if self.guesses.len() >= MAX_GUESS_NUM - 1 {
            best_guesses = best_possibly_answer_guesses.clone();
        }

        SolverResult {
            best_guesses,
            best_possibly_answer_guesses,
            possible_answers,
        }
    }

    /// Add a `Guess` in the game. It is unsafe because the content of the guess is not checked.
    /// Used in the interactive solver.
    pub unsafe fn add_guess(&mut self, guess: Guess) {
        self.guesses.push(guess);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn construct_word_set() -> HashSet<String> {
        HashSet::from_iter(
            ["TWICE", "FIGHT", "TEETH", "WORLD", "TICKY", "THINK"]
                .iter()
                .map(|&s| String::from(s)),
        )
    }

    #[test]
    fn successful_game() {
        let mut game = Game::new("tWiCe", &construct_word_set(), false)
            .expect("valid target word should succuessfully construct a new game");
        let mut state = [LetterState::Unknown; 26];

        assert_eq!(
            game.guess("fight", &construct_word_set())
                .expect("valid guess should be accepted"),
            GameState::InProgress
        );
        assert_eq!(
            game.guesses,
            vec![Guess {
                word: ValidWord::new("FIGHT", "test", &construct_word_set(), "test").unwrap(),
                result: [
                    LetterState::TooMany,
                    LetterState::IncorrectPos,
                    LetterState::TooMany,
                    LetterState::TooMany,
                    LetterState::IncorrectPos
                ]
            }]
        );
        state[(b'f' - b'a') as usize] = LetterState::TooMany;
        state[(b'g' - b'a') as usize] = LetterState::TooMany;
        state[(b'h' - b'a') as usize] = LetterState::TooMany;
        state[(b'i' - b'a') as usize] = LetterState::IncorrectPos;
        state[(b't' - b'a') as usize] = LetterState::IncorrectPos;
        assert_eq!(game.letter_state, state);

        game.guess("invalid", &construct_word_set())
            .expect_err("invalid guess should not be accepted");
        game.guess("apple", &construct_word_set())
            .expect_err("guesses outside of acceptable set should not be accepted");

        assert_eq!(
            game.guess("TEETH", &construct_word_set())
                .expect("valid guess should be accepted"),
            GameState::InProgress
        );
        assert_eq!(
            game.guesses,
            vec![
                Guess {
                    word: ValidWord::new("FIGHT", "test", &construct_word_set(), "test").unwrap(),
                    result: [
                        LetterState::TooMany,
                        LetterState::IncorrectPos,
                        LetterState::TooMany,
                        LetterState::TooMany,
                        LetterState::IncorrectPos
                    ]
                },
                Guess {
                    word: ValidWord::new("teeth", "test", &construct_word_set(), "test").unwrap(),
                    result: [
                        LetterState::Correct,
                        LetterState::IncorrectPos,
                        LetterState::TooMany,
                        LetterState::TooMany,
                        LetterState::TooMany,
                    ]
                }
            ]
        );
        state[(b'e' - b'a') as usize] = LetterState::IncorrectPos;
        state[(b't' - b'a') as usize] = LetterState::Correct;
        assert_eq!(game.letter_state, state);

        assert_eq!(
            game.guess("twice", &construct_word_set())
                .expect("valid guess should be accepted"),
            GameState::Success { guess_count: 3 },
        );
        assert_eq!(
            *game.guesses.last().expect("guesses should not be empty"),
            Guess {
                word: ValidWord::new("twice", "test", &construct_word_set(), "test").unwrap(),
                result: [
                    LetterState::Correct,
                    LetterState::Correct,
                    LetterState::Correct,
                    LetterState::Correct,
                    LetterState::Correct,
                ]
            }
        );
        state[(b'w' - b'a') as usize] = LetterState::Correct;
        state[(b'i' - b'a') as usize] = LetterState::Correct;
        state[(b'c' - b'a') as usize] = LetterState::Correct;
        state[(b'e' - b'a') as usize] = LetterState::Correct;
        assert_eq!(game.letter_state, state);
    }

    #[test]
    fn invalid_target() {
        Game::new("invalid", &construct_word_set(), false)
            .expect_err("invalid target should fail to construct a new game");
        Game::new("apple", &construct_word_set(), false)
            .expect_err("target outside of the set should fail to construct a new game");
    }

    #[test]
    fn failed_game() {
        let mut game = Game::new("tWiCe", &construct_word_set(), false)
            .expect("valid target word should succuessfully construct a new game");

        for i in 0..MAX_GUESS_NUM {
            assert_eq!(
                game.guess("fight", &construct_word_set())
                    .expect("valid guess should be accepted"),
                if i == MAX_GUESS_NUM - 1 {
                    GameState::Failed {
                        answer: String::from("TWICE"),
                    }
                } else {
                    GameState::InProgress
                }
            );
        }

        assert_eq!(
            game.guesses,
            vec![
                Guess {
                    word: ValidWord::new("FIGHT", "test", &construct_word_set(), "test").unwrap(),
                    result: [
                        LetterState::TooMany,
                        LetterState::IncorrectPos,
                        LetterState::TooMany,
                        LetterState::TooMany,
                        LetterState::IncorrectPos
                    ]
                };
                MAX_GUESS_NUM
            ]
        );

        let mut state = [LetterState::Unknown; 26];
        state[(b'f' - b'a') as usize] = LetterState::TooMany;
        state[(b'g' - b'a') as usize] = LetterState::TooMany;
        state[(b'h' - b'a') as usize] = LetterState::TooMany;
        state[(b'i' - b'a') as usize] = LetterState::IncorrectPos;
        state[(b't' - b'a') as usize] = LetterState::IncorrectPos;
        assert_eq!(game.letter_state, state);
    }

    #[test]
    fn difficult_mode() {
        let mut game = Game::new("tWiCe", &construct_word_set(), false)
            .expect("valid target word should succuessfully construct a new game");
        game.guess("fight", &construct_word_set())
            .expect("valid guess should be accepted");
        game.guess("world", &construct_word_set())
            .expect("I and T not used, but not in difficult mode");
        game.guess("think", &construct_word_set())
            .expect("valid guess should be accepted");
        game.guess("ticky", &construct_word_set())
            .expect("incorrect position for I, but not in difficult mode");
        game.guess("twice", &construct_word_set())
            .expect("valid guess should be accepted");

        let mut game = Game::new("tWiCe", &construct_word_set(), true)
            .expect("valid target word should succuessfully construct a new game");
        game.guess("fight", &construct_word_set())
            .expect("valid guess should be accepted");
        game.guess("world", &construct_word_set())
            .expect_err("I and T not used, should be rejected in difficult mode");
        game.guess("think", &construct_word_set())
            .expect("valid guess should be accepted");
        game.guess("ticky", &construct_word_set())
            .expect_err("incorrect position for I in difficult mode");
        game.guess("twice", &construct_word_set())
            .expect("valid guess should be accepted");
    }
}
