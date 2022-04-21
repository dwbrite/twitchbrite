mod config;

// dependencies
use crate::ui;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::terminal::enable_raw_mode;
use hueclient::Bridge;
use std::thread;
use std::time::Duration;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Borders;
use tui::Terminal;

// re-exports
pub use self::config::Config;

pub struct LightInterface {
    bridge: Bridge,
    config: Config,
}

pub struct TwitchBrite<B: Backend> {
    should_stop: bool,
    ticks: u64,
    terminal: Terminal<B>,
    // TODO: add list of screens and current screen, requires a Trait with update() and draw()
}

impl<B: Backend> TwitchBrite<B> {
    pub fn with_backend(backend: B) -> anyhow::Result<()> {
        let mut terminal = Terminal::new(backend)?;

        enable_raw_mode()?; // TODO: this depends on crossterm - if the rest of the code is backend-agnostic, shouldn't this be, too?
        terminal.clear()?;

        let mut app = Self {
            should_stop: false,
            ticks: 0,
            terminal,
        };

        loop {
            app.update()?;
            app.draw()?;

            if app.should_stop {
                break;
            }

            thread::sleep(Duration::from_millis(16))
        }

        return Ok(());
    }

    fn update(&mut self) -> anyhow::Result<()> {
        self.ticks += 1;

        while event::poll(Duration::from_millis(0))? {
            match event::read().unwrap() {
                Event::Key(e) => {
                    // (temporary) kill program on esc
                    if e.code == KeyCode::Esc {
                        self.should_stop = true;
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
            let size = f.size();
            let block = tui::widgets::Block::default().style(Style::default().bg(Color::Black));
            let menu = tui::widgets::Block::default()
                .borders(Borders::ALL)
                .title(" welcome to twitchbrite ")
                .style(Style::default().bg(Color::Black));

            let bg = crate::ui::bg::Background {
                state: self.ticks as f32 * 0.016,
            };
            f.render_widget(bg, size);
            f.render_widget(block, Rect::new(2, 1, size.width - 4, size.height - 2));
            f.render_widget(menu, ui::center_rect(size, 60, 16));
        })?;

        Ok(())
    }
}