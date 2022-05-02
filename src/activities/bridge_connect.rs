use crate::activities::Activity;
use crate::config::Config;
use crate::widgets::{center_rect, draw_bg};
use crate::GlobalState;
use crossbeam_channel::{Receiver, Sender};
use hueclient::Bridge;

use std::thread;
use tui::backend::Backend;
use tui::buffer::Buffer;

use crate::widgets::log_block::{Log, LogEvent};
use crate::widgets::rainbow_border::RainbowBorderWidget;
use tui::layout::Rect;
use tui::widgets::{Block, Borders, Widget};
use tui::Frame;

/*pub mod tasks {
    use crate::activities::bridge_connect::State;
    use crate::config::Config;
    use crate::widgets::log_block::LogVariant::{Info, TaskComplete, TaskFailed};
    use crate::widgets::log_block::{LogEvent, LogItem};
    use crossbeam_channel::Sender;
    use hueclient::{Bridge, UnauthBridge};
    use std::fs;

    pub trait Task {
        fn run_task(&mut self, tx: Sender<LogEvent>) -> anyhow::Result<State>;
    }

    pub struct ReadConfigTask;
    pub struct DiscoverBridgeTask;
    pub struct RegisterClientTask {
        device_type: String,
        unauth_bridge: Option<UnauthBridge>,
    }
    pub struct ValidateConfigTask {
        config: Config,
    }

    impl Task for DiscoverBridgeTask {
        fn run_task(&mut self, tx: Sender<LogEvent>) -> anyhow::Result<State> {
            let (log, id) = LogItem::task_waiting("Discovering Philips Hue bridge...");
            tx.send(LogEvent::PushItem(log)).unwrap();

            match Bridge::discover() {
                None => {
                    tx.send(LogEvent::SetVariant(id, TaskFailed)).unwrap();
                    tx.send(LogEvent::PushItem(
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
                    tx.send(LogEvent::SetVariant(id, TaskComplete)).unwrap();

                    Ok(State::RegisteringClient(Some(RegisterClientTask {
                        device_type: generate_device_type(),
                        unauth_bridge: Some(unauth_bridge),
                    })))
                }
            }
        }
    }

    impl Task for ValidateConfigTask {
        fn run_task(&mut self, tx: Sender<LogEvent>) -> anyhow::Result<State> {
            let bridge = Bridge::for_ip(self.config.bridge_ip)
                .with_user(self.config.bridge_username.clone());

            let (log, id) = LogItem::task_waiting("Validating bridge config... ");
            tx.send(LogEvent::PushItem(log)).unwrap();

            match bridge.get_all_lights() {
                Ok(lights) => {
                    tx.send(LogEvent::SetVariant(id, TaskComplete)).unwrap();
                    tx.send(LogEvent::PushItem(
                        LogItem::new(format!("Found {} lights", lights.len()), Info).0,
                    ))
                    .unwrap();
                }
                Err(e) => {
                    tx.send(LogEvent::SetVariant(id, TaskFailed)).unwrap();
                    tx.send(LogEvent::PushItem(
                        LogItem::error(
                            format!("{}\n{}\n{}\n{}",
                                    "Couldn't communicate with bridge.",
                                    "Make sure your bridge is plugged in and try again.",
                                    "If you've manually entered the bridge's IP, its DHCP lease may have expired.",
                                    "For now you'll need to delete or edit config.toml and try again.")).0,
                    )).unwrap();
                    return Err(anyhow::Error::from(e));
                }
            }

            // save config
            todo!();
        }
    }

    impl Task for RegisterClientTask {
        fn run_task(&mut self, tx: Sender<LogEvent>) -> anyhow::Result<State> {
            // let unauth_bridge = self.unauth_bridge.take().unwrap();
            //
            // let (log, id) = LogItem::task_waiting(
            //     "Registering client. Press the button on your bridge to continue.",
            // );
            // tx.send(LogEvent::PushItem(log)).unwrap();
            //
            // loop {
            //     match unauth_bridge.clone().register_user(&self.device_type) {
            //         Ok(bridge) => {
            //             tx.send(LogEvent::SetVariant(id, TaskComplete)).unwrap();
            //             return Ok(State::ValidatingConfig(Some(ValidateConfigTask {
            //                 config: Config {
            //                     device_type: self.device_type.clone(),
            //                     bridge_ip: bridge.ip,
            //                     bridge_username: bridge.username,
            //                 },
            //             })));
            //         }
            //         Err(_) => {}
            //     }
            // }
        }
    }
}*/

pub enum State {
    DiscoveringBridge(Option<()>),
    RegisteringClient(Option<()>),

    ValidatingConfig(Option<()>),

    ManualEntry {
        attempts: u32,
        current_entry: String,
    },

    Waiting(Box<State>),
    Failed(anyhow::Error),
    Complete(Config, Bridge),
}

impl State {
    fn update(mut self) -> Self {
        match self {
            // State::ReadingConfig(ref mut task) => {
            //     let task = task.take().unwrap();
            //     self.spawn_task(task, state_tx, log_tx)
            // }
            // State::DiscoveringBridge(ref mut task) => {
            //     let task = task.take().unwrap();
            //     self.spawn_task(task, state_tx, log_tx)
            // }
            // State::RegisteringClient(ref mut task) => {
            //     let task = task.take().unwrap();
            //     self.spawn_task(task, state_tx, log_tx)
            // }
            // State::ValidatingConfig(ref mut task) => {
            //     let task = task.take().unwrap();
            //     self.spawn_task(task, state_tx, log_tx)
            // }
            //
            // State::ManualEntry { .. } => self, // TODO:

            // no logic to run while waiting, complete, or failed
            // well, maybe there's some display logic to run. we'll see.
            _ => self,
        }
    }
}

pub struct BridgeConnect {
    state: State,
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
    fn render(&mut self, f: &mut Frame<B>) {
        f.render_widget(
            BridgeConnectWidget {
                border_animated: false,
                ticks: 0,
            },
            f.size(),
        );

        f.render_widget(self.log.clone(), center_rect(f.size(), 80, 24));
    }

    // fn draw(&mut self, state: &mut GlobalState, f: &mut Frame<B>) {
    //     draw_bg(state, f);
    //
    //     match self.state {
    //         // Manual IP entry needs special rendering
    //         State::ManualEntry { attempts, .. } => {
    //             let block = Block::default()
    //                 .borders(Borders::ALL)
    //                 .title(format!(" oog! : {} ", attempts));
    //
    //             f.render_widget(block, center_rect(f.size(), 72, 20));
    //         }
    //
    //         // print the log in any other case.
    //         _ => {
    //             f.render_stateful_widget(
    //                 self.log.widget(),
    //                 center_rect(f.size(), 72, 20),
    //                 &mut self.log.history,
    //             );
    //         }
    //     }
    // }

    fn update(&mut self, _state: &mut GlobalState) {
        // let log_tx = self.log.sender();
        let (state_tx, state_rx) = self.state_ch.clone();
        take_mut::take(&mut self.state, move |state| state.update());

        if let Ok(state) = state_rx.try_recv() {
            self.state = state
        }

        self.log.update();
    }
}

impl BridgeConnect {
    pub fn init() -> Self {
        let mut log = Log::default();
        log.set_title(String::from(" welcome to twitchbrite "));

        Self {
            state: State::DiscoveringBridge(None),
            log,
            state_ch: crossbeam_channel::unbounded(),
        }
    }
}
