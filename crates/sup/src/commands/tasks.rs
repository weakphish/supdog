use anyhow::Result;
use sup_core::db::Database;
use crate::cli::args::TaskCommand;

pub fn list(_db: &mut Database, _tag: Option<String>, _status: Option<String>, _json: bool) -> Result<()> {
    println!("tasks");
    Ok(())
}

pub fn dispatch_task(_db: &mut Database, _cmd: TaskCommand, _json: bool) -> Result<()> {
    println!("task command");
    Ok(())
}
