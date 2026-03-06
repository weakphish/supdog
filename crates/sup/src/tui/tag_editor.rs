// crates/sup/src/tui/tag_editor.rs
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};
use sup_core::{
    db::Database,
    models::Tag,
    queries::tags,
};

pub struct TagEditor {
    pub node_id: String,
    pub current_tags: Vec<Tag>,
    pub input: String,
    pub active: bool,
    pub list_state: ListState,
}

impl TagEditor {
    pub fn new() -> Self {
        Self {
            node_id: String::new(),
            current_tags: vec![],
            input: String::new(),
            active: false,
            list_state: ListState::default(),
        }
    }

    pub fn open(&mut self, db: &mut Database, node_id: String) -> Result<()> {
        self.node_id = node_id.clone();
        self.current_tags = tags::get_tags_for_node(db, &node_id)?;
        self.input.clear();
        self.active = true;
        if !self.current_tags.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
        Ok(())
    }

    pub fn close(&mut self) {
        self.active = false;
        self.input.clear();
    }

    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn pop_char(&mut self) {
        self.input.pop();
    }

    /// Add the tag currently in the input field
    pub fn add_tag(&mut self, db: &mut Database) -> Result<()> {
        let tag_name = self.input.trim().to_string();
        if tag_name.is_empty() { return Ok(()); }
        let tag = tags::get_or_create(db, &tag_name)?;
        tags::add_tag_to_node(db, &self.node_id, &tag.id)?;
        // refresh
        self.current_tags = tags::get_tags_for_node(db, &self.node_id)?;
        self.input.clear();
        Ok(())
    }

    /// Remove the currently selected tag
    pub fn remove_selected_tag(&mut self, db: &mut Database) -> Result<()> {
        if self.current_tags.is_empty() { return Ok(()); }
        let sel = self.list_state.selected().unwrap_or(0);
        if sel >= self.current_tags.len() { return Ok(()); }
        let tag_id = self.current_tags[sel].id.clone();
        tags::remove_tag_from_node(db, &self.node_id, &tag_id)?;
        // refresh
        self.current_tags = tags::get_tags_for_node(db, &self.node_id)?;
        // keep selection in bounds
        let len = self.current_tags.len();
        if len == 0 {
            self.list_state.select(None);
        } else {
            let new_sel = sel.min(len - 1);
            self.list_state.select(Some(new_sel));
        }
        Ok(())
    }

    pub fn move_up(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i > 0 { self.list_state.select(Some(i - 1)); }
    }

    pub fn move_down(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i + 1 < self.current_tags.len() { self.list_state.select(Some(i + 1)); }
    }
}

pub fn render(state: &mut TagEditor, frame: &mut Frame) {
    let area = centered_rect(60, 50, frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),     // tag list
            Constraint::Length(3),  // input field
            Constraint::Length(1),  // hint
        ])
        .split(area);

    // Current tags list
    let items: Vec<ListItem> = state.current_tags.iter()
        .map(|t| ListItem::new(format!("#{}", t.name)))
        .collect();
    let list = List::new(items)
        .block(Block::bordered().title(" Tags (j/k select, d remove) "))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
    frame.render_stateful_widget(list, chunks[0], &mut state.list_state);

    // Input field for adding new tag
    let input_display = format!("#{}_", state.input);
    let input = Paragraph::new(input_display)
        .block(Block::bordered().title(" Add tag (Enter to add) "))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input, chunks[1]);

    // Hint
    let hint = Paragraph::new("Esc to close")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, chunks[2]);
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
