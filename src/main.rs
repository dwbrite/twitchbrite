mod twitchbrite;
mod ui;

use anyhow::Result;
use std::io;
use tui::backend::CrosstermBackend;

// re-exports
use twitchbrite::*;

fn main() -> anyhow::Result<()> {
    let stdout = io::stdout();
    TwitchBrite::with_backend(CrosstermBackend::new(stdout))
}
