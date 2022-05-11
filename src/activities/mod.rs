pub mod bridge_connect;

use tui::backend::Backend;

use tui::Frame;

pub trait Activity<B: Backend> {
    fn render(&mut self, ticks: u64, f: &mut Frame<B>);
    fn update(&mut self, ticks: u64);
}
