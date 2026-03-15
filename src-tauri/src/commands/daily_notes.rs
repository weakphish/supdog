use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use crate::models::DailyNote;
use crate::db::DbState;

pub fn get_or_create_daily_note_impl(conn: &Connection, date: &str) -> Result<DailyNote, String> {
    let existing: Option<DailyNote> = conn
        .query_row(
            "SELECT id, date FROM daily_notes WHERE date = ?1",
            params![date],
            |row| Ok(DailyNote { id: row.get(0)?, date: row.get(1)? }),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(note) = existing {
        return Ok(note);
    }

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO daily_notes (id, date) VALUES (?1, ?2)",
        params![id, date],
    )
    .map_err(|e| e.to_string())?;

    Ok(DailyNote { id, date: date.to_string() })
}

#[tauri::command]
pub fn get_or_create_daily_note(state: tauri::State<DbState>, date: String) -> Result<DailyNote, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_or_create_daily_note_impl(&conn, &date)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::TempDir;

    fn test_conn() -> (TempDir, Connection) {
        let tmp = TempDir::new().unwrap();
        let conn = db::init_db(tmp.path().to_path_buf()).unwrap();
        (tmp, conn)
    }

    #[test]
    fn test_get_or_create_daily_note() {
        let (_tmp, conn) = test_conn();
        let note = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(note.date, "2026-03-15");
        let note2 = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(note.id, note2.id);
    }
}
