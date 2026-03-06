use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sup", about = "Terminal knowledge base and engineering journal")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output as JSON (machine-readable)
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Append a node to today's journal
    Log {
        content: String,
        #[arg(long, default_value = "bullet")]
        r#type: String,
        #[arg(long)]
        tag: Option<String>,
    },
    /// Show today's journal
    Today,
    /// Show a specific day's journal (format: YYYY-MM-DD)
    Day { date: String },
    /// Task subcommands
    Task {
        #[command(subcommand)]
        cmd: TaskCommand,
    },
    /// List tasks
    Tasks {
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    /// Search nodes by content
    Search { query: String },
    /// Launch the TUI
    Tui,
}

#[derive(Subcommand)]
pub enum TaskCommand {
    /// Add a new task
    Add {
        title: String,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        due: Option<String>,
        #[arg(long)]
        tag: Option<String>,
    },
    /// Mark a task as done
    Done { id: String },
    /// Edit a task's properties
    Edit {
        id: String,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        due: Option<String>,
    },
}
