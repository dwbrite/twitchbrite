use crate::unicorn_vomit;
use crate::widgets::{center_rect, static_unicorn_vomit, BLACK};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::Widget;

pub struct RainbowBorderWidget {
    pub(crate) edge_animated: bool,
    pub(crate) ticks: u64,
}

impl Widget for RainbowBorderWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let outer_rect = area;
        let inner_rect = center_rect(outer_rect, outer_rect.width - 4, outer_rect.height - 2);

        // set foreground to unicorn_vomit
        let static_vomit_fg = static_unicorn_vomit::Foreground;
        static_vomit_fg.render(outer_rect, buf);

        // set background to unicorn vomit
        if self.edge_animated {
            let state = self.ticks as f32 * 0.016;
            let bg = unicorn_vomit::Background { state };
            bg.render(outer_rect, buf);
        } else {
            let bg = static_unicorn_vomit::Background;
            bg.render(outer_rect, buf);
        }

        let block = tui::widgets::Block::default().style(Style::default().bg(BLACK));
        block.render(inner_rect, buf);
    }
}
