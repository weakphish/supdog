// crates/sup/src/tui/app.rs
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Paragraph};
use ratatui::text::Line;
use sup_core::db::Database;
use crate::tui::events::AppEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Journal,
    Tasks,
    Split,
}

pub struct App {
    pub db: Database,
    pub view: View,
    pub should_quit: bool,
}

pub enum Message {
    Quit,
    SwitchView(View),
    Noop,
}

impl App {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            view: View::Journal,
            should_quit: false,
        }
    }

    pub fn handle_event(&self, event: AppEvent) -> Message {
        match event {
            AppEvent::Key(k) => match k.code {
                KeyCode::Char('q') => Message::Quit,
                KeyCode::Char('1') => Message::SwitchView(View::Journal),
                KeyCode::Char('2') => Message::SwitchView(View::Tasks),
                KeyCode::Char('3') => Message::SwitchView(View::Split),
                _ => Message::Noop,
            },
            AppEvent::Tick => Message::Noop,
        }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::Quit => self.should_quit = true,
            Message::SwitchView(v) => self.view = v,
            Message::Noop => {}
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let view_label = match self.view {
            View::Journal => "Journal [1]  Tasks [2]  Split [3]  Quit [q]",
            View::Tasks   => "Journal [1]  Tasks [2]  Split [3]  Quit [q]",
            View::Split   => "Journal [1]  Tasks [2]  Split [3]  Quit [q]",
        };

        // Status bar at the bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let view_name = match self.view {
            View::Journal => "sup — Journal",
            View::Tasks => "sup — Tasks",
            View::Split => "sup — Split",
        };

        let placeholder = Paragraph::new(Line::from(format!(
            "{} (TUI views coming in next tasks)",
            view_name
        )))
        .block(Block::bordered());

        let status = Paragraph::new(Line::from(view_label))
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        frame.render_widget(placeholder, chunks[0]);
        frame.render_widget(status, chunks[1]);
    }
}
