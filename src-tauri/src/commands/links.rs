use rusqlite::{params, Connection};
use uuid::Uuid;
use crate::models::{Block, BlockLink};
use crate::commands::blocks::row_to_block;
use crate::db::DbState;

pub fn create_link_impl(conn: &Connection, source_id: &str, target_id: &str) -> Result<BlockLink, String> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO block_links (id, source_id, target_id) VALUES (?1, ?2, ?3)",
        params![id, source_id, target_id],
    ).map_err(|e| e.to_string())?;
    let link = conn.query_row(
        "SELECT id, source_id, target_id, created_at FROM block_links WHERE id = ?1",
        params![id],
        |r| Ok(BlockLink { id: r.get(0)?, source_id: r.get(1)?, target_id: r.get(2)?, created_at: r.get(3)? })
    ).map_err(|e| e.to_string())?;
    Ok(link)
}

pub fn delete_link_impl(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM block_links WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_backlinks_impl(conn: &Connection, block_id: &str) -> Result<Vec<Block>, String> {
    let mut stmt = conn.prepare(
        "SELECT b.* FROM blocks b JOIN block_links bl ON b.id = bl.source_id WHERE bl.target_id = ?1 ORDER BY b.created_at"
    ).map_err(|e| e.to_string())?;
    let blocks = stmt.query_map(params![block_id], |r| row_to_block(r))
        .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    Ok(blocks)
}

pub fn get_forward_links_impl(conn: &Connection, block_id: &str) -> Result<Vec<Block>, String> {
    let mut stmt = conn.prepare(
        "SELECT b.* FROM blocks b JOIN block_links bl ON b.id = bl.target_id WHERE bl.source_id = ?1 ORDER BY b.created_at"
    ).map_err(|e| e.to_string())?;
    let blocks = stmt.query_map(params![block_id], |r| row_to_block(r))
        .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    Ok(blocks)
}

#[tauri::command]
pub fn create_link(state: tauri::State<DbState>, source_id: String, target_id: String) -> Result<BlockLink, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    create_link_impl(&conn, &source_id, &target_id)
}

#[tauri::command]
pub fn delete_link(state: tauri::State<DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    delete_link_impl(&conn, &id)
}

#[tauri::command]
pub fn get_backlinks(state: tauri::State<DbState>, block_id: String) -> Result<Vec<Block>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_backlinks_impl(&conn, &block_id)
}

#[tauri::command]
pub fn get_forward_links(state: tauri::State<DbState>, block_id: String) -> Result<Vec<Block>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_forward_links_impl(&conn, &block_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::commands::{daily_notes, blocks};
    use tempfile::TempDir;

    fn test_conn() -> (TempDir, Connection) {
        let tmp = TempDir::new().unwrap();
        let conn = db::init_db(tmp.path().to_path_buf()).unwrap();
        (tmp, conn)
    }

    #[test]
    fn test_create_and_query_link() {
        let (_tmp, conn) = test_conn();
        let note = daily_notes::get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let source = blocks::create_block_impl(&conn, &note.id, None, "source", "bullet", 0).unwrap();
        let target = blocks::create_block_impl(&conn, &note.id, None, "target", "task", 1).unwrap();
        create_link_impl(&conn, &source.id, &target.id).unwrap();
        let backlinks = get_backlinks_impl(&conn, &target.id).unwrap();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].content, "source");
    }

    #[test]
    fn test_delete_link() {
        let (_tmp, conn) = test_conn();
        let note = daily_notes::get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let source = blocks::create_block_impl(&conn, &note.id, None, "source", "bullet", 0).unwrap();
        let target = blocks::create_block_impl(&conn, &note.id, None, "target", "task", 1).unwrap();
        let link = create_link_impl(&conn, &source.id, &target.id).unwrap();
        delete_link_impl(&conn, &link.id).unwrap();
        let backlinks = get_backlinks_impl(&conn, &target.id).unwrap();
        assert_eq!(backlinks.len(), 0);
    }
}
