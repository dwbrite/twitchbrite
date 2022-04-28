use crate::widgets::unicorn_vomit::calculate_color;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::Widget;

pub struct Foreground;
pub struct Background;

impl Widget for Foreground {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let fx = x as f32 / area.right() as f32;
                let fy = y as f32 / area.bottom() as f32;

                let color = calculate_color(fx, fy, 0.29);
                let style = Style::default().fg(color);
                buf.get_mut(x, y).set_style(style);
            }
        }
    }
}

impl Widget for Background {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let fx = x as f32 / area.right() as f32;
                let fy = y as f32 / area.bottom() as f32;

                let color = calculate_color(fx, fy, 0.29);
                let style = Style::default().bg(color);
                buf.get_mut(x, y).set_style(style);
            }
        }
    }
}
