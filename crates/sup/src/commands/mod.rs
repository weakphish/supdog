use anyhow::Result;
use sup_core::db::Database;
use crate::cli::args::{Cli, Command};

pub mod log;
pub mod today;
pub mod tasks;
pub mod search;

pub fn dispatch(mut db: Database, cli: Cli) -> Result<()> {
    match cli.command {
        Command::Log { content, r#type, tag } => {
            log::run(&mut db, content, r#type, tag, cli.json)
        }
        Command::Today => {
            let date = chrono::Local::now().date_naive();
            today::run(&mut db, date, cli.json)
        }
        Command::Day { date } => {
            let d = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")?;
            today::run(&mut db, d, cli.json)
        }
        Command::Tasks { tag, status } => {
            tasks::list(&mut db, tag, status, cli.json)
        }
        Command::Task { cmd } => {
            tasks::dispatch_task(&mut db, cmd, cli.json)
        }
        Command::Search { query } => {
            search::run(&mut db, query, cli.json)
        }
        Command::Tui => {
            println!("TUI not yet implemented");
            Ok(())
        }
    }
}
