// crates/sup/src/tui/journal.rs
use anyhow::Result;
use chrono::{Local, NaiveDate};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState},
};
use sup_core::{
    db::Database,
    models::{Node, NodeType},
    queries::{daily_notes, nodes, tags},
};

pub struct FlatNode {
    pub node: Node,
    pub depth: usize,
}

pub struct JournalState {
    pub date: NaiveDate,
    pub flat: Vec<FlatNode>,
    pub list_state: ListState,
}

impl JournalState {
    pub fn new(db: &mut Database) -> Result<Self> {
        let date = Local::now().date_naive();
        let mut s = Self {
            date,
            flat: vec![],
            list_state: ListState::default(),
        };
        s.reload(db)?;
        Ok(s)
    }

    pub fn reload(&mut self, db: &mut Database) -> Result<()> {
        let note = daily_notes::get_or_create(db, self.date)?;
        let roots = nodes::get_roots_for_day(db, &note.id)?;
        let tree = nodes::build_tree(db, roots)?;
        let mut tree_with_tags = tree;
        attach_tags(db, &mut tree_with_tags)?;
        self.flat = flatten_tree(&tree_with_tags, 0);
        let len = self.flat.len();
        if len == 0 {
            self.list_state.select(None);
        } else {
            let sel = self.list_state.selected().unwrap_or(0).min(len - 1);
            self.list_state.select(Some(sel));
        }
        Ok(())
    }

    pub fn move_up(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i > 0 {
            self.list_state.select(Some(i - 1));
        }
    }

    pub fn move_down(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i + 1 < self.flat.len() {
            self.list_state.select(Some(i + 1));
        }
    }

    pub fn prev_day(&mut self, db: &mut Database) -> Result<()> {
        if let Some(d) = self.date.pred_opt() {
            self.date = d;
            self.list_state.select(None);
            self.reload(db)?;
        }
        Ok(())
    }

    pub fn next_day(&mut self, db: &mut Database) -> Result<()> {
        let today = Local::now().date_naive();
        if self.date < today {
            if let Some(d) = self.date.succ_opt() {
                self.date = d;
                self.list_state.select(None);
                self.reload(db)?;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn selected_flat_node(&self) -> Option<&FlatNode> {
        self.list_state.selected().and_then(|i| self.flat.get(i))
    }
}

fn attach_tags(db: &mut Database, nodes: &mut Vec<Node>) -> Result<()> {
    for node in nodes.iter_mut() {
        let node_tags = tags::get_tags_for_node(db, &node.id)?;
        node.tags = node_tags.into_iter().map(|t| t.name).collect();
        attach_tags(db, &mut node.children)?;
    }
    Ok(())
}

fn flatten_tree(tree: &[Node], depth: usize) -> Vec<FlatNode> {
    let mut result = vec![];
    for node in tree {
        result.push(FlatNode {
            node: node.clone(),
            depth,
        });
        result.extend(flatten_tree(&node.children, depth + 1));
    }
    result
}

pub fn render(state: &mut JournalState, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = state.flat.iter().map(|flat| {
        let indent = "  ".repeat(flat.depth);

        let icon = match &flat.node.node_type {
            NodeType::Task => flat.node.status.as_ref()
                .map(|s| s.icon())
                .unwrap_or("☐"),
            t => t.icon(),
        };

        let content_style = match flat.node.node_type {
            NodeType::H1 => Style::default().add_modifier(Modifier::BOLD),
            NodeType::H2 => Style::default().add_modifier(Modifier::BOLD),
            NodeType::Quote => Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            NodeType::Code => Style::default().fg(Color::Cyan),
            _ => Style::default(),
        };

        let tag_str = if flat.node.tags.is_empty() {
            String::new()
        } else {
            format!("  #{}", flat.node.tags.join(" #"))
        };

        ListItem::new(Line::from(vec![
            Span::raw(format!("{}{} ", indent, icon)),
            Span::styled(flat.node.content.clone(), content_style),
            Span::styled(tag_str, Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();

    let title = format!(" {} ", state.date.format("%A, %B %d %Y"));
    let list = List::new(items)
        .block(Block::bordered().title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        );

    frame.render_stateful_widget(list, area, &mut state.list_state);
}
