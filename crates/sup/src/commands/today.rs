// crates/sup/src/commands/today.rs
use anyhow::Result;
use chrono::NaiveDate;
use sup_core::db::Database;
use sup_core::queries::{daily_notes, nodes, carryover};
use crate::output::print_node_tree;

pub fn run(db: &mut Database, date: NaiveDate, json: bool) -> Result<()> {
    let note = daily_notes::get_or_create(db, date)?;

    // run carryover only for today
    if date == chrono::Local::now().date_naive() {
        carryover::carry_over_tasks(db, &note.id, date)?;
    }

    // get roots and attach tags
    let roots = nodes::get_roots_for_day(db, &note.id)?;
    let mut tree = nodes::build_tree(db, roots)?;

    // attach tags to each node
    crate::output::attach_tags_to_tree(db, &mut tree)?;

    if json {
        let out = serde_json::json!({
            "date": date.to_string(),
            "nodes": tree
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("# {}", date.format("%A, %B %d %Y"));
        print_node_tree(&tree, 0);
    }
    Ok(())
}

