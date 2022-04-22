use crate::config::{generate_device_type, get_config_path, Config};
use crate::screens::bridge_connect::ConnectStatus::WaitingForButton;
use crate::screens::widgets::draw_bg;
use crate::screens::Screen;
use crate::{widgets, GlobalState};
use crossbeam_channel::{Receiver, Sender};
use hueclient::Bridge;

use std::io::Write;
use std::time::Duration;
use std::{fs, thread};
use tui::backend::Backend;

use tui::style::{Modifier, Style};
use tui::widgets::{Borders, ListItem};
use tui::Frame;

enum ConnectStatus {
    CannotFindBridge,
    WaitingForButton { waiting: bool },
    LoadingConfig { waiting: bool },
    Done { config: Config, bridge: Bridge },
    GenericError(anyhow::Error),
}

enum Message {
    SetStatus(ConnectStatus),
}

pub struct BridgeConnect {
    status: ConnectStatus,
    tx: Sender<Message>,
    rx: Receiver<Message>,
}

impl BridgeConnect {
    pub(crate) fn draw_error<B: Backend>(
        &self,
        _state: &mut GlobalState,
        f: &mut Frame<B>,
        err: &anyhow::Error,
    ) {
        let size = f.size();

        let menu_block = tui::widgets::Block::default()
            .title(" welcome to twitchbrite ")
            .borders(Borders::ALL);
        let menu = tui::widgets::List::new([ListItem::new(format!(
            "\n\n> shit's fucked, sorry! {}",
            err
        ))])
        .block(menu_block);
        f.render_widget(menu, widgets::center_rect(size, 72, 20));
    }

    pub(crate) fn draw_cannot_find_bridge<B: Backend>(
        &self,
        _state: &mut GlobalState,
        f: &mut Frame<B>,
    ) {
        let size = f.size();

        let menu_block = tui::widgets::Block::default()
            .title(" welcome to twitchbrite ")
            .borders(Borders::ALL);
        let menu = tui::widgets::List::new([ListItem::new("\n\n> can't find bridge, oops!")])
            .block(menu_block);
        f.render_widget(menu, widgets::center_rect(size, 72, 20));
    }
}

impl<B: Backend> Screen<B> for BridgeConnect {
    fn draw(&mut self, state: &mut GlobalState, f: &mut Frame<B>) {
        draw_bg(state, f);

        match &self.status {
            ConnectStatus::CannotFindBridge => self.draw_cannot_find_bridge(state, f),
            ConnectStatus::WaitingForButton { .. } => self.draw_waiting_for_button(state, f),
            ConnectStatus::LoadingConfig { .. } => self.draw_load_config(state, f),
            ConnectStatus::Done { .. } => self.draw_tmp_done(state, f),
            ConnectStatus::GenericError(e) => self.draw_error(state, f, e),
        };
    }

    fn update(&mut self, _state: &mut GlobalState) {
        match &mut self.status {
            ConnectStatus::CannotFindBridge => {}
            ConnectStatus::WaitingForButton { mut waiting } => {
                if !waiting {
                    waiting = true;
                    let tx = self.tx.clone();
                    thread::spawn(move || {
                        match Self::generate_new_config() {
                            Ok(status) => tx.send(Message::SetStatus(status)),
                            Err(e) => tx.send(Message::SetStatus(ConnectStatus::GenericError(e))),
                        };
                    });
                }
            }
            ConnectStatus::LoadingConfig { mut waiting } => {
                if !waiting {
                    waiting = true;
                    let tx = self.tx.clone();
                    thread::spawn(move || {
                        match Self::load_config() {
                            Ok(status) => tx.send(Message::SetStatus(status)),
                            Err(e) => tx.send(Message::SetStatus(ConnectStatus::GenericError(e))),
                        };
                    });
                }
            }
            ConnectStatus::GenericError(_err) => {}
            ConnectStatus::Done { .. } => {}
        }

        if let Ok(msg) = self.rx.try_recv() {
            match msg {
                Message::SetStatus(status) => self.status = status,
            }
        }
    }
}

impl BridgeConnect {
    pub fn init() -> Self {
        let (tx, rx) = crossbeam_channel::bounded(5);

        Self {
            status: ConnectStatus::LoadingConfig { waiting: false },
            tx,
            rx,
        }
    }

    fn load_config() -> anyhow::Result<ConnectStatus> {
        let config_path = get_config_path()?;
        let result = fs::read(&config_path);

        if let Err(_) = &result {
            return Ok(WaitingForButton { waiting: false });
        }

        // deserialize existing config
        let config: Config = toml::from_slice(result.unwrap().as_slice())?; // TODO: better error, failed to parse config

        if let Some(bridge) = hueclient::Bridge::discover() {
            let bridge = bridge.with_user(&config.username);

            // just to check if the username works
            bridge.get_all_lights()?;
            Ok(ConnectStatus::Done { config, bridge })
        } else {
            Ok(ConnectStatus::CannotFindBridge)
        }
    }

    fn register_bridge(device_type: &str) -> Option<Bridge> {
        let unauth_bridge = Bridge::discover()?;

        let mut bridge_result = unauth_bridge.clone().register_user(&device_type);
        while bridge_result.is_err() {
            bridge_result = unauth_bridge.clone().register_user(&device_type);
            thread::sleep(Duration::from_millis(500));
        }

        Some(bridge_result.unwrap())
    }

    fn generate_new_config() -> anyhow::Result<ConnectStatus> {
        let device_type = generate_device_type();

        let bridge_result = Self::register_bridge(&device_type);
        if bridge_result.is_none() {
            return Ok(ConnectStatus::CannotFindBridge);
        }

        let bridge = bridge_result.unwrap();

        let config = Config {
            device_type,
            username: bridge.username.clone(),
        };

        let mut file = fs::File::create(&get_config_path()?)?;

        file.write(toml::to_vec(&config)?.as_slice())?;
        return Ok(ConnectStatus::Done { bridge, config });
    }

    fn draw_load_config<B: Backend>(&self, state: &mut GlobalState, f: &mut Frame<B>) {
        let size = f.size();

        let dots_thing = (state.ticks % 40) / 10;
        let mut dots = String::from("");
        for _i in 0..dots_thing {
            dots.push('.');
        }

        let menu_block = tui::widgets::Block::default()
            .title(" welcome to twitchbrite ")
            .borders(Borders::ALL);
        let menu =
            tui::widgets::List::new([ListItem::new(format!("\n\n> loading config{}", dots))
                .style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::ITALIC),
                )])
            .block(menu_block);
        f.render_widget(menu, widgets::center_rect(size, 72, 20));
    }

    fn draw_waiting_for_button<B: Backend>(&self, state: &mut GlobalState, f: &mut Frame<B>) {
        let size = f.size();

        let dots_thing = (state.ticks % 40) / 10;
        let mut dots = String::from("");
        for _i in 0..dots_thing {
            dots.push('.');
        }

        let menu_block = tui::widgets::Block::default()
            .title(" welcome to twitchbrite ")
            .borders(Borders::ALL);
        let menu = tui::widgets::List::new([ListItem::new(format!(
            "\n\n> Let's connect twitchbrite to your Philips Hue Bridge.\n\n> Press the button on your bridge, please{}",
            dots
        ))])
        .block(menu_block);
        f.render_widget(menu, widgets::center_rect(size, 72, 20));
    }

    fn draw_tmp_done<B: Backend>(&self, _state: &mut GlobalState, f: &mut Frame<B>) {
        let size = f.size();

        let menu_block = tui::widgets::Block::default()
            .title(" welcome to twitchbrite ")
            .borders(Borders::ALL);
        let menu = tui::widgets::List::new([ListItem::new("\n\n> done!")]).block(menu_block);
        f.render_widget(menu, widgets::center_rect(size, 72, 20));
    }
}
