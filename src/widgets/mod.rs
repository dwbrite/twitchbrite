pub mod log_block;
pub mod rainbow_border;
pub mod smart_text;
pub mod static_unicorn_vomit;
pub mod unicorn_vomit;

use tui::layout::Rect;
use tui::style::Color;

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
