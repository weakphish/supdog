use anyhow::Result;
use sup_core::db::Database;

pub fn run(_db: &mut Database, content: String, _type_str: String, _tag: Option<String>, _json: bool) -> Result<()> {
    println!("log: {}", content);
    Ok(())
}
