pub mod bridge_connect;
pub mod widgets;

use crate::{unicorn_vomit, GlobalState};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::Frame;

pub trait Screen<B: Backend> {
    fn draw(&mut self, state: &mut GlobalState, f: &mut Frame<B>);
    fn update(&mut self, state: &mut GlobalState);
}
