use anyhow::Result;
use chrono::NaiveDate;
use rusqlite::params;
use uuid::Uuid;
use crate::db::Database;
use crate::models::DailyNote;

pub fn get_or_create(db: &mut Database, date: NaiveDate) -> Result<DailyNote> {
    if let Some(note) = get_by_date(db, date)? {
        return Ok(note);
    }
    let id = Uuid::new_v4().to_string();
    let date_str = date.format("%Y-%m-%d").to_string();
    db.conn.execute(
        "INSERT INTO daily_notes (id, date) VALUES (?1, ?2)",
        params![id, date_str],
    )?;
    Ok(DailyNote { id, date })
}

pub fn get_by_date(db: &mut Database, date: NaiveDate) -> Result<Option<DailyNote>> {
    let date_str = date.format("%Y-%m-%d").to_string();
    let mut stmt = db.conn.prepare("SELECT id, date FROM daily_notes WHERE date = ?1")?;
    let mut rows = stmt.query(params![date_str])?;
    if let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let date_str: String = row.get(1)?;
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")?;
        Ok(Some(DailyNote { id, date }))
    } else {
        Ok(None)
    }
}

pub fn list_all(db: &mut Database) -> Result<Vec<DailyNote>> {
    let mut stmt = db.conn.prepare("SELECT id, date FROM daily_notes ORDER BY date DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut notes = vec![];
    for row in rows {
        let (id, date_str) = row?;
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")?;
        notes.push(DailyNote { id, date });
    }
    Ok(notes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use chrono::NaiveDate;

    fn test_db() -> Database { Database::open_in_memory().unwrap() }

    #[test]
    fn test_get_or_create_today() {
        let mut db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = get_or_create(&mut db, date).unwrap();
        assert_eq!(note.date, date);
        // idempotent
        let note2 = get_or_create(&mut db, date).unwrap();
        assert_eq!(note.id, note2.id);
    }

    #[test]
    fn test_get_by_date_none_then_some() {
        let mut db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        assert!(get_by_date(&mut db, date).unwrap().is_none());
        get_or_create(&mut db, date).unwrap();
        assert!(get_by_date(&mut db, date).unwrap().is_some());
    }

    #[test]
    fn test_list_all_ordered_by_date_desc() {
        let mut db = test_db();
        let d1 = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        get_or_create(&mut db, d1).unwrap();
        get_or_create(&mut db, d2).unwrap();
        let all = list_all(&mut db).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].date, d2); // newest first
    }
}
