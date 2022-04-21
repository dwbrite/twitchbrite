pub mod widgets;

use tui::backend::Backend;
use tui::Frame;

pub trait Screen<B: Backend> {
    fn draw(f: &mut Frame<B>);
    fn update();
}
