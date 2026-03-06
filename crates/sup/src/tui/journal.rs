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
use crate::tui::editor::InlineEditor;
use sup_core::queries::nodes::{CreateNode, UpdateNode};

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

    pub fn selected_flat_node(&self) -> Option<&FlatNode> {
        self.list_state.selected().and_then(|i| self.flat.get(i))
    }

    /// Add a new node below the current selection (or at end if nothing selected)
    pub fn add_node_below(&mut self, db: &mut Database) -> Result<InlineEditor> {
        let daily_note = daily_notes::get_or_create(db, self.date)?;
        let (parent_id, position) = if let Some(flat) = self.selected_flat_node() {
            let pos = flat.node.position + 1;
            (flat.node.parent_id.clone(), pos)
        } else {
            let roots = nodes::get_roots_for_day(db, &daily_note.id)?;
            (None, roots.len() as i64)
        };

        let node = nodes::create(db, CreateNode {
            parent_id,
            daily_note_id: Some(daily_note.id),
            content: String::new(),
            node_type: NodeType::Bullet,
            position,
            status: None,
            priority: None,
            due_date: None,
        })?;

        let node_id = node.id.clone();
        self.reload(db)?;

        // select the new node
        if let Some(idx) = self.flat.iter().position(|f| f.node.id == node_id) {
            self.list_state.select(Some(idx));
        }

        Ok(InlineEditor::edit_existing(node_id, ""))
    }

    /// Delete the currently selected node
    pub fn delete_selected(&mut self, db: &mut Database) -> Result<()> {
        if let Some(flat) = self.selected_flat_node() {
            let id = flat.node.id.clone();
            nodes::delete(db, &id)?;
            self.reload(db)?;
        }
        Ok(())
    }

    /// Indent: make selected node a child of the node above it
    pub fn indent_selected(&mut self, db: &mut Database) -> Result<()> {
        let sel = self.list_state.selected().unwrap_or(0);
        if sel == 0 { return Ok(()); }
        let node_id = self.flat[sel].node.id.clone();
        let new_parent_id = self.flat[sel - 1].node.id.clone();
        nodes::update(db, &node_id, UpdateNode {
            content: None,
            status: None,
            priority: None,
            due_date: None,
            position: None,
            parent_id: Some(Some(new_parent_id)),
        })?;
        self.reload(db)?;
        Ok(())
    }

    /// Unindent: promote selected node to sibling of its parent
    pub fn unindent_selected(&mut self, db: &mut Database) -> Result<()> {
        if self.flat.is_empty() { return Ok(()); }
        let sel = self.list_state.selected().unwrap_or(0);
        let node = self.flat[sel].node.clone();
        if node.parent_id.is_none() { return Ok(()); } // already root

        // find parent's parent_id
        let parent_parent_id = if let Some(parent_id) = &node.parent_id {
            nodes::get_by_id(db, parent_id)?
                .and_then(|p| p.parent_id)
        } else {
            None
        };

        nodes::update(db, &node.id, UpdateNode {
            content: None,
            status: None,
            priority: None,
            due_date: None,
            position: None,
            parent_id: Some(parent_parent_id),
        })?;
        self.reload(db)?;
        Ok(())
    }

    /// Commit an editor result: update or finalize the node
    pub fn commit_edit(&mut self, db: &mut Database, node_id: &str, content: String) -> Result<()> {
        if content.trim().is_empty() {
            // empty content = delete the node
            nodes::delete(db, node_id)?;
        } else {
            nodes::update(db, node_id, UpdateNode {
                content: Some(content),
                status: None,
                priority: None,
                due_date: None,
                position: None,
                parent_id: None,
            })?;
        }
        self.reload(db)?;
        Ok(())
    }

    /// Start editing the currently selected node
    pub fn start_edit_selected(&self) -> Option<InlineEditor> {
        self.selected_flat_node().map(|flat| {
            InlineEditor::edit_existing(flat.node.id.clone(), &flat.node.content)
        })
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

pub fn render(state: &mut JournalState, editor: Option<&InlineEditor>, frame: &mut Frame, area: Rect) {
    let selected_idx = state.list_state.selected();

    let items: Vec<ListItem> = state.flat.iter().enumerate().map(|(i, flat)| {
        let indent = "  ".repeat(flat.depth);
        let icon = match &flat.node.node_type {
            NodeType::Task => flat.node.status.as_ref().map(|s| s.icon()).unwrap_or("☐"),
            t => t.icon(),
        };

        // If this item is being edited, show the editor content
        let display_content = if Some(i) == selected_idx {
            if let Some(ed) = editor {
                format!("{} |", ed.content) // show cursor marker
            } else {
                flat.node.content.clone()
            }
        } else {
            flat.node.content.clone()
        };

        let content_style = if editor.is_some() && Some(i) == selected_idx {
            Style::default().fg(Color::Yellow) // editing = yellow
        } else {
            match flat.node.node_type {
                NodeType::H1 | NodeType::H2 => Style::default().add_modifier(Modifier::BOLD),
                NodeType::Quote => Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                NodeType::Code => Style::default().fg(Color::Cyan),
                _ => Style::default(),
            }
        };

        let tag_str = if flat.node.tags.is_empty() || editor.is_some() {
            String::new()
        } else {
            format!("  #{}", flat.node.tags.join(" #"))
        };

        ListItem::new(Line::from(vec![
            Span::raw(format!("{}{} ", indent, icon)),
            Span::styled(display_content, content_style),
            Span::styled(tag_str, Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();

    let title = format!(" {} ", state.date.format("%A, %B %d %Y"));
    let list = List::new(items)
        .block(Block::bordered().title(title))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
    frame.render_stateful_widget(list, area, &mut state.list_state);
}
