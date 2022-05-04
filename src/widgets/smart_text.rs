use crate::Msg;
use itertools::Itertools;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::tui::buffer::Buffer;
use tuirealm::tui::layout::Rect;
use tuirealm::tui::text::{Span, Spans};
use tuirealm::tui::widgets::Widget;
use tuirealm::{
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
};
use unicode_segmentation::UnicodeSegmentation;

fn split_span_at(span: Span, idx: usize) -> (Span, Span) {
    let graphemes = span.content.graphemes(true);
    let a = graphemes.clone().take(idx).collect::<String>();
    let b = graphemes.clone().skip(idx).collect::<String>();

    (Span::styled(a, span.style), Span::styled(b, span.style))
}

fn split_span_every(span: Span, n: usize) -> Vec<Span> {
    let mut lines = vec![];
    for chunk in &span.content.graphemes(true).chunks(n) {
        lines.push(chunk.collect::<String>())
    }
    lines
        .iter()
        .map(|s| Span::styled(s.clone(), span.style.clone()))
        .collect()
}

/// Smartline represents a single *wrapped* line of text
struct WrappingText<'a> {
    lines: Vec<Line<'a>>, // one line that may be split into multiple lines
}

impl<'a> WrappingText<'a> {
    fn from_spans<T: Into<Spans<'a>>>(spans: T, width: usize) -> Self {
        let mut wrapping_text = WrappingText { lines: vec![] };
        let mut line = vec![];
        let mut x = 0;
        for span in spans.into().0 {
            // if text doesn't overflow overflow
            if x + span.width() <= width as usize {
                // append span to line
                x += span.width();
                line.push(span)
            } else {
                // if text does overflow
                // append as much as possible to line
                let remainder = width - x;
                let (a, b) = split_span_at(span.clone(), remainder);
                line.push(a);
                wrapping_text.lines.push(Line(line.clone()));
                line = vec![];
                x = 0;

                // split the remaining spans into full lines
                // the last newline will be a partial line
                let newlines = split_span_every(b, width);
                for (idx, newspan) in newlines.iter().enumerate() {
                    if idx < newlines.len() - 1 {
                        wrapping_text.lines.push(Line(vec![newspan.clone()]))
                    } else {
                        // append span to line
                        x += span.width();
                        line.push(newspan.clone())
                    }
                }
            }
        }
        wrapping_text.lines.push(Line(line));

        wrapping_text
    }
}

/// Line represents single a line of text
#[derive(Debug, Clone)]
pub struct Line<'a>(Vec<Span<'a>>);

impl<'a> Widget for Line<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_spans(area.x, area.y, &Spans(self.0), area.width);
    }
}

pub struct SmartTextComponent<'a> {
    props: Props,
    reversed: bool,
    input_text: Vec<Spans<'a>>,
}

impl<'a> SmartTextComponent<'a> {
    pub fn new() -> Self {
        Self {
            props: Default::default(),
            reversed: false,
            input_text: vec![],
        }
    }

    pub fn append_line<T: Into<Spans<'a>>>(&mut self, content: T) {
        self.input_text.push(content.into());
    }

    pub fn append_span<T: Into<Span<'a>>>(&mut self, content: T) {
        if let Some(spans) = self.input_text.last_mut() {
            spans.0.push(content.into());
        }
    }

    fn output_text(&self, area: Rect) -> Vec<WrappingText<'a>> {
        let mut output_text = vec![];

        for line in &self.input_text {
            let wrapping_text = WrappingText::from_spans(line.clone(), area.width as usize);
            output_text.push(wrapping_text);
        }

        output_text
    }
}

impl<'a> MockComponent for SmartTextComponent<'a> {
    fn view(&mut self, f: &mut Frame, area: Rect) {
        let mut lines_used = 0;
        for wrapping_text in self.output_text(area) {
            if self.reversed {
                lines_used += wrapping_text.lines.len();
                for (idx, line) in wrapping_text.lines.iter().enumerate() {
                    f.render_widget(
                        line.clone(),
                        Rect::new(area.x, area.y - (lines_used + idx) as u16, area.width, 1),
                    );
                }
            } else {
                for line in wrapping_text.lines {
                    f.render_widget(
                        line,
                        Rect::new(area.x, area.y + lines_used as u16, area.width, 1),
                    );
                    lines_used += 1;

                    if lines_used > area.height as usize {
                        return;
                    }
                }
            }
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }
    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            _ => CmdResult::None,
        }
    }
}

impl<'a> Component<Msg, NoUserEvent> for SmartTextComponent<'a> {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}
