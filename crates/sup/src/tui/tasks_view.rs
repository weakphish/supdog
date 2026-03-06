// crates/sup/src/tui/tasks_view.rs
use anyhow::Result;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState},
};
use sup_core::{
    db::Database,
    models::{Node, Priority, TaskStatus},
    queries::{nodes, tags},
};

pub struct TasksState {
    pub tasks: Vec<Node>,
    pub list_state: ListState,
}

impl TasksState {
    pub fn new(db: &mut Database) -> Result<Self> {
        let mut s = Self {
            tasks: vec![],
            list_state: ListState::default(),
        };
        s.reload(db)?;
        Ok(s)
    }

    pub fn reload(&mut self, db: &mut Database) -> Result<()> {
        let raw_tasks = nodes::get_all_tasks(db, None)?;
        let mut enriched = vec![];
        for mut task in raw_tasks {
            // attach children
            let children = nodes::get_children(db, &task.id)?;
            task.children = nodes::build_tree(db, children)?;
            // attach tags
            let task_tags = tags::get_tags_for_node(db, &task.id)?;
            task.tags = task_tags.into_iter().map(|t| t.name).collect();
            enriched.push(task);
        }
        self.tasks = enriched;

        // keep selection in bounds
        let len = self.tasks.len();
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
        if i + 1 < self.tasks.len() {
            self.list_state.select(Some(i + 1));
        }
    }

    pub fn selected_task(&self) -> Option<&Node> {
        self.list_state.selected().and_then(|i| self.tasks.get(i))
    }

    /// Cycle the status of the selected task: todo -> in_progress -> done -> cancelled -> todo
    pub fn cycle_status(&mut self, db: &mut Database) -> Result<()> {
        if self.tasks.is_empty() { return Ok(()); }
        let sel = self.list_state.selected().unwrap_or(0);
        if sel >= self.tasks.len() { return Ok(()); }
        let task = &self.tasks[sel];
        let new_status = task.status.as_ref()
            .map(|s| s.next())
            .unwrap_or(TaskStatus::Todo);
        nodes::update(db, &task.id, nodes::UpdateNode {
            status: Some(new_status),
            content: None,
            priority: None,
            due_date: None,
            position: None,
            parent_id: None,
        })?;
        self.reload(db)?;
        Ok(())
    }
}

pub fn render(state: &mut TasksState, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = state.tasks.iter().map(|task| {
        let status_icon = task.status.as_ref()
            .map(|s| s.icon())
            .unwrap_or("☐");

        let priority_badge = match &task.priority {
            Some(Priority::High) => " [H]",
            Some(Priority::Med)  => " [M]",
            Some(Priority::Low)  => " [L]",
            None => "",
        };

        let tag_str = if task.tags.is_empty() {
            String::new()
        } else {
            format!("  #{}", task.tags.join(" #"))
        };

        let due_str = task.due_date
            .map(|d| format!("  due:{}", d.format("%m/%d")))
            .unwrap_or_default();

        let status_style = match &task.status {
            Some(TaskStatus::Done) => Style::default().fg(Color::DarkGray),
            Some(TaskStatus::Cancelled) => Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT),
            Some(TaskStatus::InProgress) => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        };

        ListItem::new(Line::from(vec![
            Span::raw(format!("{} ", status_icon)),
            Span::styled(task.content.clone(), status_style),
            Span::raw(priority_badge),
            Span::styled(due_str, Style::default().fg(Color::DarkGray)),
            Span::styled(tag_str, Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();

    let list = List::new(items)
        .block(Block::bordered().title(" Tasks "))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut state.list_state);
}
