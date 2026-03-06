use anyhow::Result;
use sup_core::db::Database;

pub fn run(_db: &mut Database, query: String, _json: bool) -> Result<()> {
    println!("search: {}", query);
    Ok(())
}
