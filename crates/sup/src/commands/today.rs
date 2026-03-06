use anyhow::Result;
use chrono::NaiveDate;
use sup_core::db::Database;

pub fn run(_db: &mut Database, date: NaiveDate, _json: bool) -> Result<()> {
    println!("today: {}", date);
    Ok(())
}
