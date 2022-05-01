pub mod log_block;
pub mod rainbow_border;
pub mod static_unicorn_vomit;
pub mod unicorn_vomit;

use crate::GlobalState;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};

use tui::Frame;

/// haha, you fool, this is actually purple!
pub const BLACK: Color = Color::Rgb(14, 0, 20);

pub fn center_rect(parent_size: Rect, mut width: u16, mut height: u16) -> Rect {
    if width > parent_size.width {
        width = parent_size.width;
    }
    if height > parent_size.height {
        height = parent_size.height;
    }

    let x = parent_size.width / 2 - width / 2;
    let y = parent_size.height / 2 - height / 2;

    Rect::new(x, y, width, height)
}

pub fn draw_bg<B: Backend>(state: &mut GlobalState, f: &mut Frame<B>) {
    let outer_size = f.size();
    let inner_rect = center_rect(outer_size, 80, 24);
    let block = tui::widgets::Block::default().style(Style::default().bg(BLACK));

    // set foreground to unicorn_vomit
    let static_vomit_fg = static_unicorn_vomit::Foreground;
    f.render_widget(static_vomit_fg, inner_rect);

    // set background to unicorn vomit
    if state.edge_animated {
        let state = state.ticks as f32 * 0.016;
        let bg = unicorn_vomit::Background { state };
        f.render_widget(bg, inner_rect);
    } else {
        let bg = static_unicorn_vomit::Background;
        f.render_widget(bg, inner_rect);
    }

    // fill with black, leaving just the unicorn vomit
    f.render_widget(
        block,
        Rect::new(
            inner_rect.x + 2,
            inner_rect.y + 1,
            inner_rect.width - 4,
            inner_rect.height - 2,
        ),
    );
}
