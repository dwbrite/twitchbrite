use crossbeam_channel::{Receiver, Sender};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::VecDeque;
use tui::buffer::Buffer;
use tui::layout::{Corner, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Widget};

#[derive(Debug, Copy, Clone)]
pub enum LogVariant {
    Info,
    Error,
    TaskWaiting,
    TaskFailed,
    TaskComplete,
}

#[derive(Debug, Clone)]
pub struct LogItem {
    pub message: String,
    pub variant: LogVariant,
    id: String, // ehh
}

impl LogItem {
    pub fn new<T: Into<String>>(message: T, variant: LogVariant) -> (Self, String) {
        let mut rng = thread_rng();
        let id: String = (0..6).map(|_| rng.sample(Alphanumeric) as char).collect(); // somewhat unlikely to clash :)))

        (
            LogItem {
                message: message.into(),
                variant,
                id: id.clone(),
            },
            id,
        )
    }

    pub fn set_variant(&mut self, variant: LogVariant) {
        self.variant = variant;
    }

    pub fn info<T: Into<String>>(message: T) -> (Self, String) {
        Self::new(message, LogVariant::Info)
    }
    pub fn error<T: Into<String>>(message: T) -> (Self, String) {
        Self::new(message, LogVariant::Error)
    }
    pub fn task_waiting<T: Into<String>>(message: T) -> (Self, String) {
        Self::new(message, LogVariant::TaskWaiting)
    }
    pub fn task_failed<T: Into<String>>(message: T) -> (Self, String) {
        Self::new(message, LogVariant::TaskFailed)
    }
    pub fn task_complete<T: Into<String>>(message: T) -> (Self, String) {
        Self::new(message, LogVariant::TaskComplete)
    }
}

#[derive(Debug, Clone)]
pub enum LogEvent {
    PushItem(LogItem),
    PopItem,
    SetVariant(String, LogVariant), // use LogItem id
}

#[derive(Debug, Clone)]
pub struct Margin {
    top: u16,
    bottom: u16,
    left: u16,
    right: u16,
}

#[derive(Debug, Clone)]
pub struct LogHistory {
    content: VecDeque<LogItem>,
    ticks: u64,
}

#[derive(Debug, Clone)]
pub struct Log {
    title: String,
    margin: Margin,
    history: LogHistory,
    rx: Receiver<LogEvent>,
    tx: Sender<LogEvent>,
}

impl Log {
    pub fn default() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        Log {
            title: "".to_string(),
            margin: Margin {
                top: 1,
                bottom: 1,
                left: 2,
                right: 2,
            },
            history: LogHistory {
                content: Default::default(),
                ticks: 0,
            },
            rx,
            tx,
        }
    }

    pub fn new(title: String, margin: Margin) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        Log {
            title,
            margin,
            history: LogHistory {
                content: Default::default(),
                ticks: 0,
            },
            rx,
            tx,
        }
    }

    pub fn sender(&self) -> Sender<LogEvent> {
        self.tx.clone()
    }

    pub fn update(&mut self) {
        self.history.ticks += 1;

        while let Ok(event) = self.rx.try_recv() {
            match event {
                LogEvent::PushItem(item) => {
                    self.history.content.push_front(item);
                }
                LogEvent::PopItem => {
                    self.history.content.pop_front();
                }
                LogEvent::SetVariant(id, variant) => {
                    for item in &mut self.history.content {
                        if item.id == id {
                            item.variant = variant;
                        }
                    }
                }
            }
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_margin(&mut self, margin: Margin) {
        self.margin = margin;
    }

    fn calculate_inner(&self, area: Rect) -> Rect {
        Rect {
            x: area.x + self.margin.left,
            y: area.y + self.margin.top,
            width: area.width - (self.margin.left + self.margin.right),
            height: area.height - (self.margin.top + self.margin.bottom),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogWidget {
    title: String,
    margin: Margin,
    history: LogHistory,
}

impl From<&mut LogHistory> for Vec<ListItem<'_>> {
    fn from(history: &mut LogHistory) -> Self {
        history
            .content
            .iter()
            .map(|item| -> ListItem {
                // ~8 fps animation
                let anim_idx = ((history.ticks % 32) / 8) as usize;
                let waiting_anim = ["[|]", "[/]", "[—]", "[\\]"];
                let waiting = Span::styled(
                    waiting_anim[anim_idx],
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
                let done = Span::styled(
                    "[=]",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                );
                let failed = Span::styled(
                    "[×]",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                );
                let info = Span::styled("[i]", Style::default().add_modifier(Modifier::BOLD));
                let error = Span::styled(
                    "[!]",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                );

                match item.variant {
                    LogVariant::Info => ListItem::new(Spans::from(vec![
                        info,
                        Span::raw(format!(" {}", item.message)),
                    ])),
                    LogVariant::Error => ListItem::new(Spans::from(vec![
                        error,
                        Span::styled(
                            format!(" {}", item.message.clone()),
                            Style::default().fg(Color::Red),
                        ),
                    ])),
                    LogVariant::TaskWaiting => ListItem::new(Spans::from(vec![
                        waiting,
                        Span::raw(format!(" {}", item.message)),
                    ])),
                    LogVariant::TaskFailed => ListItem::new(Spans::from(vec![
                        failed,
                        Span::raw(format!(" {}", item.message)),
                    ])),
                    LogVariant::TaskComplete => ListItem::new(Spans::from(vec![
                        done,
                        Span::raw(format!(" {}", item.message)),
                    ])),
                }
            })
            .collect()
    }
}

impl Widget for Log {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.clone());
        outer_block.render(area, buf);

        // TODO: break up (newline + tab?) log items based on width vs inner_area
        let log_list = List::new(&mut self.history).start_corner(Corner::BottomLeft);
        Widget::render(log_list, self.calculate_inner(area), buf);
    }
}
