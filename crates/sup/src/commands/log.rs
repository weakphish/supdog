// crates/sup/src/commands/log.rs
use anyhow::Result;
use chrono::Local;
use sup_core::db::Database;
use sup_core::models::NodeType;
use sup_core::queries::{daily_notes, nodes, tags};

pub fn run(db: &mut Database, content: String, type_str: String, tag: Option<String>, json: bool) -> Result<()> {
    let today = Local::now().date_naive();
    let note = daily_notes::get_or_create(db, today)?;

    let roots = nodes::get_roots_for_day(db, &note.id)?;
    let position = roots.len() as i64;

    let node_type = NodeType::from_str(&type_str).unwrap_or(NodeType::Bullet);
    let mut node = nodes::create(db, nodes::CreateNode {
        parent_id: None,
        daily_note_id: Some(note.id),
        content,
        node_type,
        position,
        status: None,
        priority: None,
        due_date: None,
    })?;

    if let Some(tag_str) = tag {
        let t = tags::get_or_create(db, &tag_str)?;
        tags::add_tag_to_node(db, &node.id, &t.id)?;
        node.tags = vec![t.name];
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&node)?);
    } else {
        println!("logged: {} {}", node.node_type.icon(), node.content);
    }
    Ok(())
}
