// crates/sup/src/commands/tasks.rs
use anyhow::Result;
use chrono::{Local, NaiveDate};
use sup_core::db::Database;
use sup_core::models::{NodeType, Priority, TaskStatus};
use sup_core::queries::{daily_notes, nodes, tags};
use crate::cli::args::TaskCommand;
use crate::output::print_node_tree;

pub fn list(db: &mut Database, tag: Option<String>, status: Option<String>, json: bool) -> Result<()> {
    let status_filter = status.as_deref().and_then(|s| TaskStatus::from_str(s).ok());
    let mut task_list = nodes::get_all_tasks(db, status_filter.as_ref())?;

    // filter by tag if provided
    if let Some(tag_str) = &tag {
        let node_ids = tags::get_nodes_for_tag_prefix(db, tag_str)?;
        task_list.retain(|t| node_ids.contains(&t.id));
    }

    // attach children + tags
    let mut result = vec![];
    for mut task in task_list {
        let children = nodes::get_children(db, &task.id)?;
        task.children = nodes::build_tree(db, children)?;
        result.push(task);
    }
    crate::output::attach_tags_to_tree(db, &mut result)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if result.is_empty() {
            println!("no tasks");
        } else {
            print_node_tree(&result, 0);
        }
    }
    Ok(())
}

pub fn dispatch_task(db: &mut Database, cmd: TaskCommand, json: bool) -> Result<()> {
    match cmd {
        TaskCommand::Add { title, priority, due, tag } => {
            let today = Local::now().date_naive();
            let note = daily_notes::get_or_create(db, today)?;
            let roots = nodes::get_roots_for_day(db, &note.id)?;
            let position = roots.len() as i64;

            let due_date = due.as_deref()
                .map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d"))
                .transpose()?;

            let mut node = nodes::create(db, nodes::CreateNode {
                parent_id: None,
                daily_note_id: Some(note.id),
                content: title,
                node_type: NodeType::Task,
                position,
                status: Some(TaskStatus::Todo),
                priority: priority.as_deref().and_then(|p| Priority::from_str(p).ok()),
                due_date,
            })?;

            if let Some(tag_str) = tag {
                let t = tags::get_or_create(db, &tag_str)?;
                tags::add_tag_to_node(db, &node.id, &t.id)?;
                node.tags = vec![t.name];
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&node)?);
            } else {
                println!("task created [{}]: {}", node.id, node.content);
            }
        }
        TaskCommand::Done { id } => {
            nodes::update(db, &id, nodes::UpdateNode {
                content: None,
                status: Some(TaskStatus::Done),
                priority: None,
                due_date: None,
                position: None,
                parent_id: None,
            })?;
            if json {
                println!("{{\"status\":\"done\",\"id\":\"{}\"}}", id);
            } else {
                println!("✓ done");
            }
        }
        TaskCommand::Edit { id, status, priority, due } => {
            nodes::update(db, &id, nodes::UpdateNode {
                content: None,
                status: status.as_deref().and_then(|s| TaskStatus::from_str(s).ok()),
                priority: priority.as_deref().and_then(|p| Priority::from_str(p).ok()),
                due_date: due.as_deref()
                    .map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
                position: None,
                parent_id: None,
            })?;
            if json {
                println!("{{\"status\":\"updated\",\"id\":\"{}\"}}", id);
            } else {
                println!("updated");
            }
        }
    }
    Ok(())
}
