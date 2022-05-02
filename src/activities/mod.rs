pub mod bridge_connect;

use crate::GlobalState;
use tui::backend::Backend;

use tui::widgets::Widget;
use tui::Frame;

pub trait Activity<B: Backend> {
    fn render(&mut self, f: &mut Frame<B>);
    fn update(&mut self, state: &mut GlobalState);
}
