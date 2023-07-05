//! A GUI powered by egui
//!
//! It requires the "gui" feature flag.

mod app;

use super::*;
use crate::args::Args;
use crate::game::Guess;
use crate::LETTER_NUM_IN_WORD;
use anyhow::{anyhow, Context};
use app::App;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use std::ops::DerefMut;
use std::sync::mpsc::Sender;
use std::thread;
use std::{
    cell::RefCell,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
};

/// shared data between the GUI thread and the logic thread
#[derive(Default)]
struct Data {
    pub guesses: Vec<Guess>,
    pub guess_input: [Option<char>; LETTER_NUM_IN_WORD],
    pub letter_state: [LetterState; 26],
    pub reading_guess: bool,
    pub solver_available: bool,
}

/// The type and content of a modal dialog
enum Modal {
    Message { title: String, content: String },
    TargetWordInput,
    AskNextRound,
    Share(String),
}

/// The result of a modal dialog
enum ModalResult {
    Finished,
    Input(String),
    YesNo(bool),
    GuessOrSolver(GuessOrSolver),
}

/// The GUI struct used in the logic thread that implements the UserInterface trait
struct Gui {
    data: Arc<Mutex<Data>>,
    modal_request_tx: Sender<Modal>,
    modal_result_rx: Receiver<ModalResult>,
    /// used to generate random success message
    rng: RefCell<StdRng>,
}

/// Run the game in GUI based on the provided args.
pub fn run(args: Args) -> Result<()> {
    let data = Arc::new(Mutex::new(Data::default()));
    let gui_data = Arc::clone(&data);

    let (modal_request_tx, modal_request_rx) = mpsc::channel();
    let (modal_result_tx, modal_result_rx) = mpsc::channel();
    let (game_end_tx, game_end_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();

    let gui = Gui {
        data,
        modal_request_tx,
        modal_result_rx,
        rng: RefCell::new(StdRng::from_entropy()),
    };

    thread::spawn(move || {
        result_tx.send(crate::run(&args, &gui)).ok();
        game_end_tx.send(()).ok();
    });

    eframe::run_native(
        "Wordle",
        eframe::NativeOptions {
            initial_window_size: Some(eframe::egui::vec2(
                app::MIN_WINDOW_WIDTH,
                app::MIN_WINDOW_HEIGHT,
            )),
            min_window_size: Some(eframe::egui::vec2(
                app::MIN_WINDOW_WIDTH,
                app::MIN_WINDOW_HEIGHT,
            )),
            default_theme: eframe::Theme::Light,
            ..Default::default()
        },
        Box::new(|cc| {
            Box::new(App::new(
                cc,
                gui_data,
                modal_request_rx,
                modal_result_tx,
                game_end_rx,
            ))
        }),
    );

    result_rx.recv().unwrap()
}

const GUI_CLOSED: &str = "GUI closed before game ends";
const INCORRECT_MESSAGE: &str = "received incorrect message sent by GUI";

impl UserInterface for Gui {
    fn welcome(&self, _args: &Args) {}

    fn ask_for_target_word(&self) -> Result<String> {
        self.modal_request_tx
            .send(Modal::TargetWordInput)
            .context(GUI_CLOSED)?;
        match self.modal_result_rx.recv() {
            Ok(ModalResult::Input(word)) => Ok(word),
            Ok(_) => Err(anyhow!(INCORRECT_MESSAGE)),
            Err(_) => Err(anyhow!(GUI_CLOSED)),
        }
    }

    fn get_guess(&self, _after_invalid: bool) -> Result<String> {
        self.data.lock().unwrap().reading_guess = true;
        match self.modal_result_rx.recv() {
            Ok(ModalResult::Finished) => Ok(self
                .data
                .lock()
                .unwrap()
                .guess_input
                .iter()
                .filter_map(|c| *c)
                .collect()),
            Ok(_) => Err(anyhow!(INCORRECT_MESSAGE)),
            Err(_) => Err(anyhow!(GUI_CLOSED)),
        }
    }

    fn get_guess_or_solver(&self, _after_invalid: bool) -> Result<GuessOrSolver> {
        let mut data = self.data.lock().unwrap();
        data.reading_guess = true;
        data.solver_available = true;
        drop(data);
        let result = match self.modal_result_rx.recv() {
            Ok(ModalResult::Finished) => Ok(GuessOrSolver::Guess(
                self.data
                    .lock()
                    .unwrap()
                    .guess_input
                    .iter()
                    .filter_map(|c| *c)
                    .collect(),
            )),
            Ok(ModalResult::GuessOrSolver(result)) => Ok(result),
            Ok(_) => Err(anyhow!(INCORRECT_MESSAGE)),
            Err(_) => Err(anyhow!(GUI_CLOSED)),
        };
        result
    }

    fn show_possible_answers(&self, possible_answers: Vec<String>) {
        let len = possible_answers.len();
        let content = if len == 1 {
            format!(
                "The only remaining possible answer is {}.",
                possible_answers[0]
            )
        } else {
            let mut content = format!("There are {} remaining possible answers:", len);
            for chunk in possible_answers.chunks(5) {
                content.push('\n');
                content += &chunk.join(" ");
            }
            content
        };
        self.modal_request_tx
            .send(Modal::Message {
                title: String::from("Possible Answers"),
                content,
            })
            .ok();
        self.modal_result_rx.recv().ok();
    }

    fn show_solver_result(&self, solver_result: SolverResult) {
        let len = solver_result.possible_answers.len();
        if len == 1 {
            self.show_possible_answers(solver_result.possible_answers);
        } else {
            let mut content = format!("There are {} remaining possible answers.\n", len);
            content += "Among them, the recommended guesses are:\n";
            for item in solver_result.best_possibly_answer_guesses.iter().take(5) {
                content += &format!(" - {} (score: {:.2})\n", item.word, item.score);
            }
            if solver_result.best_guesses[0].word
                != solver_result.best_possibly_answer_guesses[0].word
            {
                content += "However, if words that are impossible to be the answer are considered, the recommended guesses will be:";
                for item in solver_result.best_guesses.iter().take(5) {
                    content += &format!("\n - {} (score: {:.2})", item.word, item.score);
                }
            }
            self.modal_request_tx
                .send(Modal::Message {
                    title: String::from("Recommended Guesses"),
                    content,
                })
                .ok();
            self.modal_result_rx.recv().ok();
        }
    }

    fn show_game(&self, game: &Game) {
        let mut data = self.data.lock().unwrap();
        data.guess_input.fill(Default::default());
        data.guesses = game.guesses.clone();
        data.letter_state = game.letter_state;
    }

    fn show_game_state(&self, state: &GameState) {
        match state {
            GameState::InProgress => {
                panic!("UserInterface::show_game_state gets GameState::InProgress")
            }
            GameState::Failed { answer } => {
                self.modal_request_tx
                    .send(Modal::Message {
                        title: String::from("Failed"),
                        content: answer.clone(),
                    })
                    .ok();
            }
            GameState::Success { .. } => {
                let content = String::from(
                    *[
                        "Genius",
                        "Magnificent",
                        "Impressive",
                        "Splendid",
                        "Great",
                        "Phew",
                    ]
                    .choose(self.rng.borrow_mut().deref_mut())
                    .unwrap(),
                );
                self.modal_request_tx
                    .send(Modal::Message {
                        title: String::from("Succeeded"),
                        content,
                    })
                    .ok();
            }
        }
        self.modal_result_rx.recv().ok();
    }

    fn show_share(&self, content: String) {
        self.modal_request_tx.send(Modal::Share(content)).ok();
        self.modal_result_rx.recv().ok();
    }

    fn ask_if_next_round(&self) -> Result<bool> {
        self.modal_request_tx
            .send(Modal::AskNextRound)
            .context(GUI_CLOSED)?;
        match self.modal_result_rx.recv() {
            Ok(ModalResult::YesNo(yesno)) => Ok(yesno),
            Ok(_) => Err(anyhow!(INCORRECT_MESSAGE)),
            Err(_) => Err(anyhow!(GUI_CLOSED)),
        }
    }

    fn show_invalid_target(&self, error: Error) {
        self.modal_request_tx
            .send(Modal::Message {
                title: String::from("Invalid Answer"),
                content: format!("{}", error),
            })
            .ok();
        self.modal_result_rx.recv().ok();
    }

    fn show_invalid_guess(&self, error: Error) {
        self.modal_request_tx
            .send(Modal::Message {
                title: String::from("Invalid Guess"),
                content: format!("{}", error),
            })
            .ok();
        self.modal_result_rx.recv().ok();
    }

    fn show_stat(&self, stat: Stat) {
        let mut content = format!(
            "Success: {}\nFail: {}\nAvg guess on success: {:.2}\n\nMost used words:",
            stat.success, stat.fail, stat.avg
        );
        for (word, time) in &stat.frequent {
            content += &format!(
                "\n - {} ({} time{})",
                word,
                time,
                if *time > 1 { "s" } else { "" }
            );
        }
        self.modal_request_tx
            .send(Modal::Message {
                title: String::from("Game Stats"),
                content,
            })
            .ok();
        self.modal_result_rx.recv().ok();
    }

    fn on_new_round(&self) {
        let mut data = self.data.lock().unwrap();
        *data = Default::default();
    }
}
