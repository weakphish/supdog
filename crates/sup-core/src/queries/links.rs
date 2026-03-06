use anyhow::Result;
use rusqlite::params;
use uuid::Uuid;
use chrono::Utc;
use crate::db::Database;

pub fn create_link(db: &mut Database, source_id: &str, target_id: &str) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    db.conn.execute(
        "INSERT OR IGNORE INTO node_links (id, source_id, target_id, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, source_id, target_id, now],
    )?;
    Ok(())
}
