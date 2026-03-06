mod cli;
mod config;
mod commands;
mod output;
mod tui;

use clap::Parser;
use cli::args::Cli;
use anyhow::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_path = config::db_path();
    let db = sup_core::db::Database::open(db_path.to_str().unwrap())?;
    commands::dispatch(db, cli)
}
