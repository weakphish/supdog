// crates/sup/src/commands/search.rs
use anyhow::Result;
use sup_core::db::Database;
use sup_core::queries::search;
use crate::output::print_node_tree;

pub fn run(db: &mut Database, query: String, json: bool) -> Result<()> {
    let results = search::search_nodes(db, &query)?;
    if json {
        let out = serde_json::json!({
            "query": query,
            "results": results
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        if results.is_empty() {
            println!("no results for \"{}\"", query);
        } else {
            println!("{} result(s):", results.len());
            print_node_tree(&results, 0);
        }
    }
    Ok(())
}
