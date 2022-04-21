use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Widget;

pub struct Background {
    pub state: f32,
}

impl Widget for Background {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let fx = x as f32 / area.right() as f32;
                let fy = y as f32 / area.bottom() as f32;

                let t = self.state;
                let base = fx + fy + t;

                // base is on an infinite scale

                let mut fr = (base) % 2.0;
                let mut fg = (base + 0.6667) % 2.0;
                let mut fb = (base + 1.3333) % 2.0; // cannot be larger than 2.0

                fn transform(col: &mut f32) {
                    if *col >= 1.0 {
                        *col -= (*col % 1.0) * 2.0;
                    }
                }

                transform(&mut fr);
                transform(&mut fg);
                transform(&mut fb);

                // get the total on a scale from 0.0 to 1.0
                // when > 1.0, subtract ft % 1.0 * 2

                let r = (fr * 255.0) as u8;
                let g = (fg * 255.0) as u8;
                let b = (fb * 255.0) as u8;

                let style = Style::default().bg(Color::Rgb(r, g, b));

                buf.get_mut(x, y).set_style(style);
            }
        }
    }
}
