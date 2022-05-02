mod activities;
mod config;
mod tasks;
pub mod widgets;

use crate::widgets::unicorn_vomit;

use crossterm::terminal::enable_raw_mode;
use std::time::Duration;
use std::{io, thread};
use tui::backend::{Backend, CrosstermBackend};

use crate::activities::bridge_connect::BridgeConnect;
use crate::activities::Activity;
use tui::Terminal;

// TODO: maybe add blocking for the (what should be) asynchronous parts like hue comms

pub struct GlobalState {
    ticks: u64,
    should_stop: bool,
    edge_animated: bool,
}

pub struct TwitchBrite<B: Backend> {
    terminal: Terminal<B>,
    activity: Box<dyn Activity<B>>,
    state: GlobalState,
    // history_stack: Vec<Box<dyn Activity<B>>>,
}

impl<B: Backend> TwitchBrite<B> {
    pub fn with_backend(backend: B) -> anyhow::Result<()> {
        let mut terminal = Terminal::new(backend)?;

        enable_raw_mode()?; // TODO: this depends on crossterm - if the rest of the code is backend-agnostic, shouldn't this be, too?
        terminal.clear()?;

        let mut app = Self {
            terminal,
            activity: Box::new(BridgeConnect::init()),
            state: GlobalState {
                should_stop: false,
                ticks: 0,
                edge_animated: true,
            },
            // history_stack: vec![],
        };

        loop {
            app.update()?;
            app.draw()?;

            if app.state.should_stop {
                break;
            }

            thread::sleep(Duration::from_millis(16))
        }

        return Ok(());
    }

    fn update(&mut self) -> anyhow::Result<()> {
        self.state.ticks += 1;

        // new screens can use event::poll() and event::read() for input
        self.activity.update(&mut self.state);

        Ok(())
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        self.terminal.draw(|f| {
            self.activity.render(f);
        })?;

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdout = io::stdout();
    TwitchBrite::with_backend(CrosstermBackend::new(stdout))
}
