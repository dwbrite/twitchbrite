use crate::activities::Activity;
use crate::config::{BridgeConfig, ValidatedBridge};
use crate::widgets::center_rect;
use crossbeam_channel::{Receiver, Sender};
use hueclient::{Bridge, UnauthBridge};

use tui::backend::Backend;
use tui::buffer::Buffer;

use crate::activities::bridge_connect::State::Waiting;
use crate::tasks::Task;
use crate::widgets::log_block::LogVariant::{TaskComplete, TaskFailed};
use crate::widgets::log_block::{Log, LogEvent, LogItem};
use crate::widgets::rainbow_border::RainbowBorderWidget;
use tui::layout::Rect;
use tui::widgets::{Block, Borders, Widget};
use tui::Frame;

pub struct DiscoverBridgeTask {
    log_tx: Sender<LogEvent>,
}

pub struct RegisterClientTask {
    log_tx: Sender<LogEvent>,
    device_type: String,
    unauth_bridge: Option<UnauthBridge>,
}

fn on_complete_default(r: anyhow::Result<State>, p: Sender<State>) {
    match r {
        Ok(state) => p.send(state).unwrap(),
        Err(e) => p.send(State::Failed(e)).unwrap(),
    };
}

impl Task for DiscoverBridgeTask {
    type Result = State;
    type OnCompleteParams = Sender<State>;

    fn run_task(self) -> anyhow::Result<State> {
        let (log, id) = LogItem::task_waiting("Discovering Philips Hue bridge...");
        self.log_tx.send(LogEvent::PushItem(log)).unwrap();

        match Bridge::discover() {
            None => {
                self.log_tx
                    .send(LogEvent::SetVariant(id, TaskFailed))
                    .unwrap();
                self.log_tx
                    .send(LogEvent::PushItem(
                        LogItem::info(
                            "Failed to discover bridge. Enter the bridge's IP address manually.",
                        )
                        .0,
                    ))
                    .unwrap();
                Ok(State::ManualEntry {
                    attempts: 0,
                    current_entry: "".to_string(),
                })
            }

            Some(unauth_bridge) => {
                self.log_tx
                    .send(LogEvent::SetVariant(id, TaskComplete))
                    .unwrap();

                Ok(State::RegisteringClient(Some(RegisterClientTask {
                    log_tx: self.log_tx,
                    device_type: BridgeConfig::generate_device_type(),
                    unauth_bridge: Some(unauth_bridge),
                })))
            }
        }
    }

    fn on_complete(r: anyhow::Result<Self::Result>, p: Self::OnCompleteParams) {
        on_complete_default(r, p)
    }
}

impl Task for RegisterClientTask {
    type Result = State;
    type OnCompleteParams = Sender<State>;

    fn run_task(mut self) -> anyhow::Result<Self::Result> {
        let unauth_bridge = self.unauth_bridge.take().unwrap();

        let (log, id) = LogItem::task_waiting(
            "Registering client. Press the button on your bridge to continue.",
        );
        self.log_tx.send(LogEvent::PushItem(log)).unwrap();

        loop {
            match unauth_bridge.clone().register_user(&self.device_type) {
                Ok(bridge) => {
                    self.log_tx
                        .send(LogEvent::SetVariant(id, TaskComplete))
                        .unwrap();
                    let bridge = ValidatedBridge::from_bridge(bridge, self.device_type)?;
                    return Ok(State::Complete(bridge));
                }
                Err(_) => {}
            }
        }
    }

    fn on_complete(r: anyhow::Result<Self::Result>, p: Self::OnCompleteParams) {
        on_complete_default(r, p)
    }
}

pub enum State {
    DiscoveringBridge(Option<DiscoverBridgeTask>),
    RegisteringClient(Option<RegisterClientTask>),

    ManualEntry {
        attempts: u32,
        current_entry: String,
    },

    Waiting(Box<State>),
    Failed(anyhow::Error),
    Complete(ValidatedBridge),
}

impl State {
    fn update(mut self, state_tx: Sender<State>) -> Self {
        match self {
            State::DiscoveringBridge(ref mut task) => {
                let task = task.take().unwrap();
                task.spawn(state_tx.clone());
                Waiting(Box::new(self))
            }

            State::RegisteringClient(ref mut task) => {
                let task = task.take().unwrap();
                task.spawn(state_tx);
                Waiting(Box::new(self))
            }

            State::ManualEntry { .. } => self, // TODO: should this be a different activity?

            _ => self,
        }
    }
}

pub struct BridgeConnect {
    state: Option<State>,
    log: Log,
    state_ch: (Sender<State>, Receiver<State>),
}

pub struct BridgeConnectWidget {
    border_animated: bool,
    ticks: u64,
}

impl Widget for BridgeConnectWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        RainbowBorderWidget {
            border_animated: self.border_animated,
            ticks: self.ticks,
        }
        .render(area, buf);
    }
}

impl<B: Backend> Activity<B> for BridgeConnect {
    fn render(&mut self, ticks: u64, f: &mut Frame<B>) {
        f.render_widget(
            BridgeConnectWidget {
                border_animated: false,
                ticks,
            },
            f.size(),
        );

        if let Some(state) = &self.state {
            match state {
                // Manual IP entry needs special rendering
                State::ManualEntry { attempts, .. } => {
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" oog! : {} ", attempts));
                    f.render_widget(block, center_rect(f.size(), 72, 20));
                }

                // print the log in any other case.
                _ => {
                    f.render_widget(self.log.clone(), center_rect(f.size(), 72, 20));
                }
            }
        }
    }

    fn update(&mut self, _ticks: u64) {
        let (state_tx, state_rx) = self.state_ch.clone();

        if let Ok(state) = state_rx.try_recv() {
            self.state = Some(state.update(state_tx));
        }

        self.log.update();
    }
}

impl BridgeConnect {
    pub fn init() -> Self {
        let mut log = Log::default();
        log.set_title(String::from(" welcome to twitchbrite "));

        let state_ch = crossbeam_channel::unbounded();
        state_ch
            .0
            .send(State::DiscoveringBridge(Some(DiscoverBridgeTask {
                log_tx: log.sender(),
            })))
            .unwrap();

        Self {
            state: None,
            log,
            state_ch,
        }
    }
}
