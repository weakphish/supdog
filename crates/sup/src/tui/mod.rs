// crates/sup/src/tui/mod.rs
pub mod app;
pub mod events;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use sup_core::db::Database;

pub fn run(db: Database) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::new(db);

    let result = run_loop(&mut terminal, &mut app);

    // Always clean up, even on error
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut app::App,
) -> Result<()> {
    loop {
        terminal.draw(|f| app.render(f))?;
        let event = events::next_event()?;
        let msg = app.handle_event(event);
        app.update(msg);
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
