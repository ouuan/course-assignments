use super::{Data, Modal, ModalResult};
use crate::{game::LetterState, ui::GuessOrSolver, LETTER_NUM_IN_WORD, MAX_GUESS_NUM};
use arboard::Clipboard;
use eframe::egui;
use std::{
    sync::{
        mpsc::{Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
    time::Duration,
};

pub struct App {
    data: Arc<Mutex<Data>>,
    modal_request_rx: Receiver<Modal>,
    modal_result_tx: Sender<ModalResult>,
    game_end_rx: Receiver<()>,
    modal_open: bool,
    modal_content: Option<Modal>,
    /// Used to send modal result when a modal is closed by egui::Window::open.
    /// It should be set to false if the modal result is already sent when the modal is closed.
    modal_was_opened: bool,
    input_buffer: String,
    pending_exit: bool,
}

impl App {
    pub(super) fn new(
        cc: &eframe::CreationContext,
        data: Arc<Mutex<Data>>,
        modal_request_rx: Receiver<Modal>,
        modal_result_tx: Sender<ModalResult>,
        game_end_rx: Receiver<()>,
    ) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Noto Sans Bold".into(),
            egui::FontData::from_static(include_bytes!("../../../assets/NotoSans-Bold.ttf")),
        );
        fonts.families.insert(
            egui::FontFamily::Name("bold".into()),
            vec!["Noto Sans Bold".into()],
        );
        cc.egui_ctx.set_fonts(fonts);

        Self {
            data,
            modal_request_rx,
            modal_result_tx,
            game_end_rx,
            modal_open: false,
            modal_content: None,
            modal_was_opened: false,
            input_buffer: String::new(),
            pending_exit: false,
        }
    }
}

fn state_to_color(state: &LetterState) -> egui::Color32 {
    match state {
        // the colors are copied from the official Wordle in light theme
        LetterState::Unknown => egui::Color32::from_rgb(211, 214, 218),
        LetterState::TooMany => egui::Color32::from_rgb(120, 124, 126),
        LetterState::IncorrectPos => egui::Color32::from_rgb(201, 180, 88),
        LetterState::Correct => egui::Color32::from_rgb(106, 170, 100),
    }
}

const TILE_SIZE: f32 = 46.5;
const TILE_BORDER: f32 = 1.5;
const TILE_GAP: f32 = 3.75;
const BUTTON_WIDTH: f32 = 32.25;
const BUTTON_HEIGHT: f32 = 43.5;
const BUTTON_GAP_X: f32 = 4.5;
const BUTTON_GAP_Y: f32 = 6.0;

pub const MIN_WINDOW_WIDTH: f32 = BUTTON_WIDTH * 10.0 + BUTTON_GAP_X * 11.0 + 10.0;
pub const MIN_WINDOW_HEIGHT: f32 = TILE_SIZE * MAX_GUESS_NUM as f32
    + TILE_GAP * MAX_GUESS_NUM as f32
    + BUTTON_HEIGHT * 3.0
    + BUTTON_GAP_Y * 2.0
    + 125.0;

fn letter_tile(ui: &mut egui::Ui, letter: char, state: &LetterState) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(TILE_SIZE, TILE_SIZE),
        egui::Sense {
            click: false,
            drag: false,
            focusable: false,
        },
    );

    if ui.is_rect_visible(rect) {
        ui.painter().rect(
            rect.shrink(TILE_BORDER * 0.5),
            0.0,
            state_to_color(state),
            egui::Stroke::new(TILE_BORDER, egui::Color32::from_rgb(211, 214, 218)),
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            letter,
            egui::FontId::new(31.0, egui::FontFamily::Name("bold".into())),
            egui::Color32::WHITE,
        );
    }
}

fn no_state_letter_tile(ui: &mut egui::Ui, letter: Option<char>) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(TILE_SIZE, TILE_SIZE),
        egui::Sense {
            click: false,
            drag: false,
            focusable: false,
        },
    );

    if ui.is_rect_visible(rect) {
        ui.painter().rect(
            rect.shrink(TILE_BORDER * 0.5),
            0.0,
            egui::Color32::WHITE,
            egui::Stroke::new(
                TILE_BORDER,
                if letter.is_some() {
                    egui::Color32::from_rgb(135, 138, 140)
                } else {
                    egui::Color32::from_rgb(211, 214, 218)
                },
            ),
        );
        if let Some(letter) = letter {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                letter,
                egui::FontId::new(31.0, egui::FontFamily::Name("bold".into())),
                egui::Color32::BLACK,
            );
        }
    }
}

fn letter_input_button(ui: &mut egui::Ui, letter: char, state: &LetterState) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(BUTTON_WIDTH, BUTTON_HEIGHT),
        egui::Sense::click(),
    );

    if ui.is_rect_visible(rect) {
        let rect = rect.expand(ui.style().interact(&response).expansion);
        ui.painter()
            .rect(rect, 3.0, state_to_color(state), egui::Stroke::none());
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            letter,
            egui::FontId::new(13.5, egui::FontFamily::Name("bold".into())),
            if *state == LetterState::Unknown {
                egui::Color32::BLACK
            } else {
                egui::Color32::WHITE
            },
        );
    }

    response
}

fn input_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(BUTTON_WIDTH * 1.5 + BUTTON_GAP_X * 0.5, BUTTON_HEIGHT),
        egui::Sense::click(),
    );

    if ui.is_rect_visible(rect) {
        let rect = rect.expand(ui.style().interact(&response).expansion);
        ui.painter().rect(
            rect,
            3.0,
            state_to_color(&LetterState::Unknown),
            egui::Stroke::none(),
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::new(12.0, egui::FontFamily::Name("bold".into())),
            egui::Color32::BLACK,
        );
    }

    response
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.pending_exit && !self.modal_open {
            frame.close();
        }

        match self.game_end_rx.try_recv() {
            Err(TryRecvError::Empty) => {}
            _ => self.pending_exit = true,
        }

        let mut data = self.data.lock().unwrap();

        // handle modal dialogs
        if !self.modal_open {
            if self.modal_was_opened {
                self.modal_was_opened = false;
                self.modal_result_tx
                    .send(ModalResult::Finished)
                    .unwrap_or_else(|_| frame.close());
            }
            match self.modal_request_rx.try_recv() {
                Ok(request) => {
                    self.modal_open = true;
                    self.modal_content = Some(request);
                    self.input_buffer.clear();
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => frame.close(),
            }
        } else {
            self.modal_was_opened = true;
            if let Some(content) = &self.modal_content {
                match content {
                    Modal::Message { title, content } => {
                        egui::Window::new(title)
                            .vscroll(true)
                            .open(&mut self.modal_open)
                            .show(ctx, |ui| {
                                ui.label(content);
                            });
                        if ctx.input().key_pressed(egui::Key::Escape) {
                            self.modal_open = false;
                        }
                    }
                    Modal::TargetWordInput => {
                        egui::Window::new("Enter the target word for the next round").show(
                            ctx,
                            |ui| {
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Min),
                                    |ui| {
                                        if (ui
                                            .text_edit_singleline(&mut self.input_buffer)
                                            .lost_focus()
                                            && ui.input().key_pressed(egui::Key::Enter))
                                            || ui.button("Submit").clicked()
                                        {
                                            self.modal_open = false;
                                            self.modal_was_opened = false;
                                            self.modal_result_tx
                                                .send(ModalResult::Input(String::from(
                                                    self.input_buffer.trim(),
                                                )))
                                                .unwrap_or_else(|_| frame.close());
                                        }
                                    },
                                );
                            },
                        );
                    }
                    Modal::AskNextRound => {
                        egui::Window::new("Play another round?").show(ctx, |ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                if ui.button("Continue (Y)").clicked()
                                    || ui.input().key_pressed(egui::Key::Y)
                                {
                                    self.modal_open = false;
                                    self.modal_was_opened = false;
                                    self.modal_result_tx
                                        .send(ModalResult::YesNo(true))
                                        .unwrap_or_else(|_| frame.close());
                                }
                                if ui.button("Exit (N)").clicked()
                                    || ui.input().key_pressed(egui::Key::N)
                                {
                                    self.modal_open = false;
                                    self.modal_was_opened = false;
                                    self.modal_result_tx
                                        .send(ModalResult::YesNo(false))
                                        .unwrap_or_else(|_| frame.close());
                                }
                            });
                        });
                    }
                    Modal::Share(content) => {
                        let mut copied = false;
                        egui::Window::new("Share")
                            .open(&mut self.modal_open)
                            .show(ctx, |ui| {
                                if ui.button("Copy share message (C)").clicked()
                                    || ui.input().key_pressed(egui::Key::C)
                                {
                                    Clipboard::new()
                                        .expect("failed to use clipboard")
                                        .set_text(content.clone())
                                        .expect("failed to write to cilpboard");
                                    copied = true;
                                }
                            });
                        if copied || ctx.input().key_pressed(egui::Key::Escape) {
                            self.modal_open = false;
                        }
                    }
                };
            }
        }

        // draw the UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!self.modal_open);

            {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(BUTTON_WIDTH * 10.0 + BUTTON_GAP_X * 9.0, 30.0),
                    egui::Sense {
                        click: false,
                        drag: false,
                        focusable: false,
                    },
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Wordle",
                    egui::FontId::new(21.0, egui::FontFamily::Name("bold".into())),
                    egui::Color32::BLACK,
                );
            }

            // guesses
            egui::Grid::new("guesses")
                .spacing(egui::vec2(TILE_GAP, TILE_GAP))
                .show(ui, |ui| {
                    for row in 0..MAX_GUESS_NUM {
                        ui.allocate_space(egui::vec2(
                            (BUTTON_WIDTH * 10.0 + BUTTON_GAP_X * 9.0
                                - TILE_SIZE * LETTER_NUM_IN_WORD as f32
                                - TILE_GAP * (LETTER_NUM_IN_WORD + 1) as f32)
                                * 0.5,
                            TILE_SIZE,
                        ));
                        for col in 0..LETTER_NUM_IN_WORD {
                            if row < data.guesses.len() {
                                letter_tile(
                                    ui,
                                    data.guesses[row].word[col],
                                    &data.guesses[row].result[col],
                                );
                            } else if row == data.guesses.len() {
                                no_state_letter_tile(ui, data.guess_input[col]);
                            } else {
                                no_state_letter_tile(ui, None);
                            }
                        }
                        ui.end_row();
                    }
                });

            ui.allocate_space(egui::vec2(1.0, 30.0));

            // keyboard
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.set_enabled(data.reading_guess);

                let add_letter = |ui: &mut egui::Ui, letter: char| {
                    if letter_input_button(
                        ui,
                        letter,
                        &data.letter_state[(letter as u32 - 'A' as u32) as usize],
                    )
                    .clicked()
                    {
                        ctx.input_mut()
                            .events
                            .push(egui::Event::Text(letter.to_string()));
                    }
                };

                ui.spacing_mut().item_spacing = egui::vec2(BUTTON_GAP_X, BUTTON_GAP_Y);
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    for letter in "QWERTYUIOP".chars() {
                        add_letter(ui, letter);
                    }
                });
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    ui.allocate_space(egui::vec2(
                        (BUTTON_WIDTH - BUTTON_GAP_X) * 0.5,
                        BUTTON_HEIGHT,
                    ));
                    for letter in "ASDFGHJKL".chars() {
                        add_letter(ui, letter);
                    }
                });
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    if input_button(ui, "ENTER").clicked()
                        || ui.input().key_pressed(egui::Key::Enter)
                    {
                        ctx.input_mut().events.push(egui::Event::Key {
                            key: egui::Key::Enter,
                            pressed: true,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                    for letter in "ZXCVBNM".chars() {
                        add_letter(ui, letter);
                    }
                    if input_button(ui, "DEL").clicked() {
                        ctx.input_mut().events.push(egui::Event::Key {
                            key: egui::Key::Backspace,
                            pressed: true,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                });
            });

            // ask for solver
            if data.solver_available {
                ui.allocate_space(egui::vec2(1.0, 10.0));
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    if ui.button("Show possible answers (F1)").clicked()
                        || ui.input().key_pressed(egui::Key::F1)
                    {
                        data.solver_available = false;
                        data.reading_guess = false;
                        self.modal_result_tx
                            .send(ModalResult::GuessOrSolver(GuessOrSolver::PossibleAnswers))
                            .unwrap_or_else(|_| frame.close());
                    }
                    if ui.button("Show recommended guesses (F2)").clicked()
                        || ui.input().key_pressed(egui::Key::F2)
                    {
                        data.solver_available = false;
                        data.reading_guess = false;
                        self.modal_result_tx
                            .send(ModalResult::GuessOrSolver(
                                GuessOrSolver::WithRecommendation,
                            ))
                            .unwrap_or_else(|_| frame.close());
                    }
                });
            }
        });

        // handle guess input
        // This should be placed after drawing the UI,
        // because the ENTER and DEL buttons modifies the input events.
        if !self.modal_open && data.reading_guess {
            for event in &ctx.input().events {
                match event {
                    egui::Event::Key {
                        key, pressed: true, ..
                    } => match key {
                        egui::Key::Backspace => {
                            for item in data.guess_input.iter_mut().rev() {
                                if item.is_some() {
                                    *item = None;
                                    break;
                                }
                            }
                        }
                        egui::Key::Enter => {
                            data.reading_guess = false;
                            data.solver_available = false;
                        }
                        _ => {}
                    },
                    egui::Event::Text(text) => {
                        for key in text.chars() {
                            if !key.is_ascii_alphabetic() {
                                continue;
                            }
                            for item in &mut data.guess_input {
                                if item.is_none() {
                                    *item = Some(key.to_ascii_uppercase());
                                    break;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            if !data.reading_guess {
                self.modal_result_tx
                    .send(ModalResult::Finished)
                    .unwrap_or_else(|_| frame.close());
            }
        }

        // prevent outdated UI
        ctx.request_repaint_after(Duration::from_millis(200));
    }
}
