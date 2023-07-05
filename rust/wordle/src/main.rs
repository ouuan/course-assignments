use anyhow::Result;
use wordle::ui::{interactive_mode::InteractiveMode, test_mode::TestMode};

fn main() -> Result<()> {
    let args = wordle::args::get_args()?;
    if atty::is(atty::Stream::Stdout) {
        wordle::run(&args, &InteractiveMode::new())
    } else {
        wordle::run(&args, &TestMode())
    }
}
