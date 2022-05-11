mod activities;
mod config;
mod tasks;
pub mod widgets;

use crate::widgets::unicorn_vomit;

use crossbeam_channel::{Receiver, Sender, TryRecvError};
use crossterm::terminal::enable_raw_mode;
use std::time::Duration;
use std::{io, thread};
use tui::backend::{Backend, CrosstermBackend};

use crate::activities::bridge_connect::BridgeConnect;
use crate::activities::Activity;
use crate::Mode::Setup;
use tui::Terminal;

// TODO: maybe add blocking for the (what should be) asynchronous parts like hue comms

pub struct GlobalState {
    ticks: u64,
    should_stop: bool,
    edge_animated: bool,
}

pub enum Mode<B: Backend> {
    Setup {
        activities: Vec<Box<dyn Activity<B>>>,
    },
}

impl<B: Backend> Mode<B> {
    pub fn init_setup(app_tx: Sender<AppMsg>) -> Mode<B> {
        Setup {
            activities: vec![
                Box::new(BridgeConnect::init(app_tx.clone())),
                Box::new(BridgeConnect::init(app_tx.clone())),
                Box::new(BridgeConnect::init(app_tx.clone())),
            ],
        }
    }
}

pub enum AppMsg {
    Next,
}

pub struct TwitchBrite<B: Backend> {
    terminal: Terminal<B>,
    activity: Box<dyn Activity<B>>,
    state: GlobalState,
    channel: (Sender<AppMsg>, Receiver<AppMsg>),
    mode: Mode<B>,
}

impl<B: Backend> TwitchBrite<B> {
    pub fn with_backend(backend: B) -> anyhow::Result<()> {
        let mut terminal = Terminal::new(backend)?;

        enable_raw_mode()?; // TODO: this depends on crossterm - if the rest of the code is backend-agnostic, shouldn't this be, too?
        terminal.clear()?;

        let channel = crossbeam_channel::unbounded();

        let mut mode = Mode::init_setup(channel.0.clone());
        let curr_activity = match &mut mode {
            Mode::Setup { activities } => activities.remove(0),
        };

        let mut app = Self {
            terminal,
            activity: curr_activity,
            state: GlobalState {
                should_stop: false,
                ticks: 0,
                edge_animated: true,
            },
            channel,
            // history_stack: vec![],
            mode,
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
        self.activity.update(self.state.ticks);

        if let Ok(x) = self.channel.1.try_recv() {
            self.handle_message(x);
        }

        Ok(())
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        self.terminal.draw(|f| {
            self.activity.render(self.state.ticks, f);
        })?;

        Ok(())
    }

    fn handle_message(&mut self, msg: AppMsg) {
        match msg {
            AppMsg::Next => {
                match &mut self.mode {
                    Mode::Setup { activities } => {
                        if activities.is_empty() {
                            // TODO: go to the next mode
                        } else {
                            self.activity = activities.remove(0);
                        }
                    }
                }
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let stdout = io::stdout();
    TwitchBrite::with_backend(CrosstermBackend::new(stdout))
}
