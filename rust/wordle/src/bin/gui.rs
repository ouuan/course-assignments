#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::Result;

fn main() -> Result<()> {
    let args = wordle::args::get_args()?;
    wordle::ui::gui::run(args)
}
