// crates/sup/src/tui/app.rs
use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use sup_core::db::Database;
use crate::tui::events::AppEvent;
use crate::tui::journal::JournalState;

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
    pub journal: JournalState,
}

pub enum Message {
    Quit,
    SwitchView(View),
    JournalUp,
    JournalDown,
    JournalPrevDay,
    JournalNextDay,
    Noop,
}

impl App {
    pub fn new(mut db: Database) -> Result<Self> {
        let journal = JournalState::new(&mut db)?;
        Ok(Self {
            db,
            view: View::Journal,
            should_quit: false,
            journal,
        })
    }

    pub fn handle_event(&self, event: AppEvent) -> Message {
        match event {
            AppEvent::Key(k) => match k.code {
                KeyCode::Char('q') => Message::Quit,
                KeyCode::Char('1') => Message::SwitchView(View::Journal),
                KeyCode::Char('2') => Message::SwitchView(View::Tasks),
                KeyCode::Char('3') => Message::SwitchView(View::Split),
                KeyCode::Char('j') => Message::JournalDown,
                KeyCode::Char('k') => Message::JournalUp,
                KeyCode::Char('[') => Message::JournalPrevDay,
                KeyCode::Char(']') => Message::JournalNextDay,
                _ => Message::Noop,
            },
            AppEvent::Tick => Message::Noop,
        }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::Quit => self.should_quit = true,
            Message::SwitchView(v) => self.view = v,
            Message::JournalUp => self.journal.move_up(),
            Message::JournalDown => self.journal.move_down(),
            Message::JournalPrevDay => {
                let db = &mut self.db;
                let _ = self.journal.prev_day(db);
            }
            Message::JournalNextDay => {
                let db = &mut self.db;
                let _ = self.journal.next_day(db);
            }
            Message::Noop => {}
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        match &self.view {
            View::Journal => {
                crate::tui::journal::render(&mut self.journal, frame, chunks[0]);
            }
            View::Tasks => {
                let placeholder = ratatui::widgets::Paragraph::new("Tasks view (coming soon)")
                    .block(ratatui::widgets::Block::bordered().title("Tasks"));
                frame.render_widget(placeholder, chunks[0]);
            }
            View::Split => {
                let split = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(chunks[0]);
                crate::tui::journal::render(&mut self.journal, frame, split[0]);
                let placeholder = ratatui::widgets::Paragraph::new("Tasks view (coming soon)")
                    .block(ratatui::widgets::Block::bordered().title("Tasks"));
                frame.render_widget(placeholder, split[1]);
            }
        }

        let status = Paragraph::new(Line::from(
            "Journal [1]  Tasks [2]  Split [3]  j/k navigate  [/] days  q quit"
        ))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
        frame.render_widget(status, chunks[1]);
    }
}
