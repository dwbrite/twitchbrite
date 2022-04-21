use crate::config::{generate_device_type, get_config_path, Config};
use crate::screens::bridge_connect::ConnectStatus::WaitingForButton;
use crate::screens::widgets::draw_bg;
use crate::screens::Screen;
use crate::{unicorn_vomit, widgets, GlobalState};
use hueclient::{Bridge, HueError};
use std::fs;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Borders, ListItem};
use tui::Frame;

enum ConnectStatus {
    CannotFindBridge,
    WaitingForButton { device_type: String },
    LoadingConfig,
    Done,
    GenericError(anyhow::Error),
}

pub struct BridgeConnect {
    status: ConnectStatus,
    bridge: Option<Bridge>,
    config: Option<Config>,
}

impl<B: Backend> Screen<B> for BridgeConnect {
    fn draw(&mut self, state: &mut GlobalState, f: &mut Frame<B>) {
        draw_bg(state, f);

        match &self.status {
            ConnectStatus::CannotFindBridge => {}
            ConnectStatus::WaitingForButton { .. } => {}
            ConnectStatus::LoadingConfig => {
                self.draw_load_config(state, f);
            }
            ConnectStatus::Done => {}
            ConnectStatus::GenericError(e) => {}
        }
    }

    fn update(&mut self, state: &mut GlobalState) {
        match &mut self.status {
            ConnectStatus::CannotFindBridge => {}
            ConnectStatus::WaitingForButton { device_type } => {}
            ConnectStatus::LoadingConfig => match self.load_config() {
                Ok(s) => {
                    self.status = s;
                }
                Err(e) => {
                    self.status = ConnectStatus::GenericError(e);
                }
            },
            ConnectStatus::GenericError(err) => {}
            ConnectStatus::Done => {}
        }
    }
}

impl BridgeConnect {
    pub fn init() -> Self {
        Self {
            status: ConnectStatus::LoadingConfig,
            bridge: None,
            config: None,
        }
    }

    fn load_config(&mut self) -> anyhow::Result<ConnectStatus> {
        let config_path = get_config_path()?;
        let result = fs::read(&config_path);

        if let Err(_) = &result {
            return Ok(WaitingForButton {
                device_type: generate_device_type(),
            });
        }

        // deserialize existing config
        let config: Config = toml::from_slice(result.unwrap().as_slice())?; // TODO: better error, failed to parse config

        if let Some(bridge) = hueclient::Bridge::discover() {
            self.bridge = Some(bridge.with_user(&config.username));
            self.config = Some(config);

            // just to check if the username works
            let _ = self.bridge.as_ref().unwrap().get_all_lights()?;
            Ok(ConnectStatus::Done)
        } else {
            Ok(ConnectStatus::CannotFindBridge)
        }
    }

    fn draw_load_config<B: Backend>(&self, state: &mut GlobalState, f: &mut Frame<B>) {
        let size = f.size();

        let dots_thing = (state.ticks % 40) / 10;
        let mut dots = String::from("");
        for i in 0..dots_thing {
            dots.push('.');
        }

        let menu_block = tui::widgets::Block::default()
            .title(" welcome to twitchbrite ")
            .borders(Borders::ALL);
        let menu = tui::widgets::List::new([ListItem::new(format!("> loading config{}", dots))
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::ITALIC),
            )])
        .block(menu_block);
        f.render_widget(menu, widgets::center_rect(size, 72, 20));
    }
}
