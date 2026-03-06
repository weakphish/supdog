// crates/sup/src/tui/search_overlay.rs
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};
use sup_core::{db::Database, models::Node, queries::search};

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Navigate,
    Link,
}

pub struct SearchOverlay {
    pub query: String,
    pub results: Vec<Node>,
    pub list_state: ListState,
    pub active: bool,
    pub mode: SearchMode,
}

impl SearchOverlay {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: vec![],
            list_state: ListState::default(),
            active: false,
            mode: SearchMode::Navigate,
        }
    }

    pub fn open(&mut self, mode: SearchMode) {
        self.active = true;
        self.mode = mode;
        self.query.clear();
        self.results.clear();
        self.list_state.select(None);
    }

    pub fn close(&mut self) {
        self.active = false;
        self.query.clear();
        self.results.clear();
    }

    pub fn push_char(&mut self, c: char, db: &mut Database) {
        self.query.push(c);
        self.run_search(db);
    }

    pub fn pop_char(&mut self, db: &mut Database) {
        self.query.pop();
        if self.query.is_empty() {
            self.results.clear();
            self.list_state.select(None);
        } else {
            self.run_search(db);
        }
    }

    fn run_search(&mut self, db: &mut Database) {
        self.results = search::search_nodes(db, &self.query).unwrap_or_default();
        if !self.results.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    pub fn move_up(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i > 0 {
            self.list_state.select(Some(i - 1));
        }
    }

    pub fn move_down(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i + 1 < self.results.len() {
            self.list_state.select(Some(i + 1));
        }
    }

    pub fn selected_node(&self) -> Option<&Node> {
        self.list_state.selected().and_then(|i| self.results.get(i))
    }
}

pub fn render(state: &mut SearchOverlay, frame: &mut Frame) {
    let area = centered_rect(75, 60, frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let title = match state.mode {
        SearchMode::Navigate => " Search (Esc to close) ",
        SearchMode::Link => " Link to node (Enter to link, Esc to cancel) ",
    };

    let input = Paragraph::new(format!("{}_", state.query))
        .block(Block::bordered().title(title))
        .style(Style::default().fg(Color::White));
    frame.render_widget(input, chunks[0]);

    let items: Vec<ListItem> = state
        .results
        .iter()
        .map(|n| ListItem::new(n.content.clone()))
        .collect();

    let list = List::new(items)
        .block(Block::bordered())
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, chunks[1], &mut state.list_state);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}
