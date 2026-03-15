use rusqlite::Connection;
use crate::models::{BlockType, SearchResult};
use crate::commands::blocks::row_to_block;
use crate::db::DbState;

/// FTS5 search with optional block_type, tag, and status filters.
/// `status_filter = "open"` maps to todo + in_progress per spec.
pub fn search_impl(conn: &Connection, query: &str, block_type_filter: Option<&str>, tag_filter: Option<&str>, status_filter: Option<&str>) -> Result<Vec<SearchResult>, String> {
    let mut sql = String::from(
        "SELECT b.*, p.content as parent_content, dn.date as daily_note_date
         FROM blocks_fts fts
         JOIN blocks b ON fts.block_id = b.id
         LEFT JOIN blocks p ON b.parent_id = p.id
         LEFT JOIN daily_notes dn ON b.daily_note_id = dn.id"
    );
    let mut conditions = vec!["blocks_fts MATCH ?1".to_string()];
    let mut param_values: Vec<String> = vec![query.to_string()];

    if let Some(bt) = block_type_filter {
        param_values.push(bt.to_string());
        conditions.push(format!("b.block_type = ?{}", param_values.len()));
    }

    if let Some(tag) = tag_filter {
        param_values.push(tag.to_string());
        sql.push_str(" JOIN block_tags bt ON b.id = bt.block_id JOIN tags t ON bt.tag_id = t.id");
        conditions.push(format!("t.name = ?{}", param_values.len()));
    }

    if let Some(status) = status_filter {
        if status == "open" {
            conditions.push("b.status IN ('todo', 'in_progress')".to_string());
        } else {
            param_values.push(status.to_string());
            conditions.push(format!("b.status = ?{}", param_values.len()));
        }
    }

    sql.push_str(&format!(" WHERE {} ORDER BY rank LIMIT 50", conditions.join(" AND ")));

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let results = stmt.query_map(rusqlite::params_from_iter(&param_values), |row| {
        let block = row_to_block(row)?;
        let parent_content: Option<String> = row.get("parent_content")?;
        let daily_note_date: Option<String> = row.get("daily_note_date")?;
        Ok(SearchResult { block, parent_content, daily_note_date })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    Ok(results)
}

#[tauri::command]
pub fn search(state: tauri::State<DbState>, query: String, block_type_filter: Option<String>, tag_filter: Option<String>, status_filter: Option<String>) -> Result<Vec<SearchResult>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    search_impl(&conn, &query, block_type_filter.as_deref(), tag_filter.as_deref(), status_filter.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::commands::{daily_notes, blocks};
    use crate::models::BlockType;
    use tempfile::TempDir;

    fn test_conn() -> (TempDir, Connection) {
        let tmp = TempDir::new().unwrap();
        let conn = db::init_db(tmp.path().to_path_buf()).unwrap();
        (tmp, conn)
    }

    #[test]
    fn test_fts_search() {
        let (_tmp, conn) = test_conn();
        let note = daily_notes::get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        blocks::create_block_impl(&conn, &note.id, None, "discussed migration strategy", "bullet", 0).unwrap();
        blocks::create_block_impl(&conn, &note.id, None, "unrelated note about lunch", "bullet", 1).unwrap();

        let results = search_impl(&conn, "migration", None, None, None).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].block.content.contains("migration"));
    }

    #[test]
    fn test_search_with_type_filter() {
        let (_tmp, conn) = test_conn();
        let note = daily_notes::get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        blocks::create_block_impl(&conn, &note.id, None, "migration note", "bullet", 0).unwrap();
        blocks::create_block_impl(&conn, &note.id, None, "migration task", "task", 1).unwrap();

        let results = search_impl(&conn, "migration", Some("task"), None, None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].block.block_type, BlockType::Task);
    }
}
