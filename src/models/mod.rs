pub mod bridge_connect;

use crate::GlobalState;
use tui::backend::Backend;

use tui::Frame;

pub trait Screen<B: Backend> {
    fn draw(&mut self, state: &mut GlobalState, f: &mut Frame<B>);
    fn update(&mut self, state: &mut GlobalState);
}

/// T is the type of message as a command
pub trait Model<T> {
    type View;
    type Message;

    fn view(&self) -> Self::View;
    fn update(&mut self, message: Self::Message) -> T;
}
