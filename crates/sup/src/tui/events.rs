// crates/sup/src/tui/events.rs
use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub fn next_event() -> anyhow::Result<AppEvent> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(k) = event::read()? {
            return Ok(AppEvent::Key(k));
        }
    }
    Ok(AppEvent::Tick)
}
