mod config;
mod screens;

use crate::screens::widgets::unicorn_vomit;
use crate::screens::{widgets, Screen};

use crate::screens::bridge_connect::BridgeConnect;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use crossterm::terminal::enable_raw_mode;
use std::time::Duration;
use std::{io, thread};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Borders;
use tui::Terminal;

// TODO: maybe add blocking for the (what should be) asynchronous parts like hue comms

pub struct GlobalState {
    ticks: u64,
    should_stop: bool,
    edge_animated: bool,
}

pub struct TwitchBrite<B: Backend> {
    terminal: Terminal<B>,
    screen: Box<dyn Screen<B>>,
    state: GlobalState,
    history_stack: Vec<Box<dyn Screen<B>>>,
}

impl<B: Backend> TwitchBrite<B> {
    pub fn with_backend(backend: B) -> anyhow::Result<()> {
        let mut terminal = Terminal::new(backend)?;

        enable_raw_mode()?; // TODO: this depends on crossterm - if the rest of the code is backend-agnostic, shouldn't this be, too?
        terminal.clear()?;

        let mut app = Self {
            terminal,
            screen: Box::new(BridgeConnect::init()),
            state: GlobalState {
                should_stop: false,
                ticks: 0,
                edge_animated: true,
            },
            history_stack: vec![],
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

        while event::poll(Duration::from_millis(0))? {
            match event::read().unwrap() {
                Event::Key(e) => {
                    // (temporary) kill program on esc
                    if e.code == KeyCode::Esc {
                        self.state.should_stop = true;
                    }
                }
                Event::Mouse(_) => {} // we can use this for selecting things with the mouse, if warranted
                _ => {}               // ignore anything else
            }
        }

        Ok(())
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        self.terminal.draw(|f| {
            self.screen.draw(&mut self.state, f);
        })?;

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdout = io::stdout();
    TwitchBrite::with_backend(CrosstermBackend::new(stdout))
}
