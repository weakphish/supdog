use rusqlite::{params, Connection};
use uuid::Uuid;
use std::collections::HashMap;
use crate::models::{Block, BlockType};
use crate::db::DbState;

pub fn row_to_block(row: &rusqlite::Row) -> rusqlite::Result<Block> {
    Ok(Block {
        id: row.get("id")?,
        content: row.get("content")?,
        block_type: serde_json::from_value(
            serde_json::Value::String(row.get::<_, String>("block_type")?)
        ).unwrap_or(BlockType::Bullet),
        parent_id: row.get("parent_id")?,
        daily_note_id: row.get("daily_note_id")?,
        position: row.get("position")?,
        status: row.get::<_, Option<String>>("status")?
            .and_then(|s| serde_json::from_value(serde_json::Value::String(s)).ok()),
        priority: row.get::<_, Option<String>>("priority")?
            .and_then(|s| serde_json::from_value(serde_json::Value::String(s)).ok()),
        due_date: row.get("due_date")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        tags: vec![],
        children: vec![],
    })
}

pub fn create_block_impl(conn: &Connection, daily_note_id: &str, parent_id: Option<&str>, content: &str, block_type: &str, position: i64) -> Result<Block, String> {
    let id = Uuid::new_v4().to_string();
    let status = if block_type == "task" { Some("todo".to_string()) } else { None };
    conn.execute(
        "INSERT INTO blocks (id, daily_note_id, parent_id, content, block_type, position, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, daily_note_id, parent_id, content, block_type, position, status],
    ).map_err(|e| e.to_string())?;

    conn.query_row("SELECT * FROM blocks WHERE id = ?1", params![id], |row| row_to_block(row))
        .map_err(|e| e.to_string())
}

pub fn get_blocks_for_date_impl(conn: &Connection, date: &str) -> Result<Vec<Block>, String> {
    let mut stmt = conn.prepare(
        "SELECT b.* FROM blocks b JOIN daily_notes dn ON b.daily_note_id = dn.id WHERE dn.date = ?1 ORDER BY b.position"
    ).map_err(|e| e.to_string())?;
    let all_blocks: Vec<Block> = stmt.query_map(params![date], |row| row_to_block(row))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut tag_stmt = conn.prepare(
        "SELECT bt.block_id, t.name FROM block_tags bt JOIN tags t ON bt.tag_id = t.id JOIN blocks b ON bt.block_id = b.id JOIN daily_notes dn ON b.daily_note_id = dn.id WHERE dn.date = ?1"
    ).map_err(|e| e.to_string())?;
    let mut tags_map: HashMap<String, Vec<String>> = HashMap::new();
    tag_stmt.query_map(params![date], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .for_each(|(bid, tname)| { tags_map.entry(bid).or_default().push(tname); });

    let mut block_map: HashMap<String, Block> = HashMap::new();
    let mut children_map: HashMap<Option<String>, Vec<String>> = HashMap::new();
    for mut block in all_blocks {
        block.tags = tags_map.remove(&block.id).unwrap_or_default();
        children_map.entry(block.parent_id.clone()).or_default().push(block.id.clone());
        block_map.insert(block.id.clone(), block);
    }

    fn build_tree(id: &str, block_map: &mut HashMap<String, Block>, children_map: &HashMap<Option<String>, Vec<String>>) -> Block {
        let child_ids = children_map.get(&Some(id.to_string())).cloned().unwrap_or_default();
        let children: Vec<Block> = child_ids.iter().map(|cid| build_tree(cid, block_map, children_map)).collect();
        let mut block = block_map.remove(id).unwrap();
        block.children = children;
        block
    }

    let root_ids = children_map.get(&None).cloned().unwrap_or_default();
    let roots: Vec<Block> = root_ids.iter().map(|id| build_tree(id, &mut block_map, &children_map)).collect();
    Ok(roots)
}

pub fn update_block_impl(conn: &Connection, id: &str, content: Option<&str>, block_type: Option<&str>, status: Option<&str>, priority: Option<&str>, due_date: Option<&str>) -> Result<(), String> {
    let mut sets = vec!["updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')".to_string()];
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(v) = content { values.push(Box::new(v.to_string())); sets.push(format!("content = ?{}", values.len())); }
    if let Some(v) = block_type { values.push(Box::new(v.to_string())); sets.push(format!("block_type = ?{}", values.len())); }
    if let Some(v) = status { values.push(Box::new(v.to_string())); sets.push(format!("status = ?{}", values.len())); }
    if let Some(v) = priority { values.push(Box::new(v.to_string())); sets.push(format!("priority = ?{}", values.len())); }
    if let Some(v) = due_date { values.push(Box::new(v.to_string())); sets.push(format!("due_date = ?{}", values.len())); }
    values.push(Box::new(id.to_string()));
    let sql = format!("UPDATE blocks SET {} WHERE id = ?{}", sets.join(", "), values.len());
    let rows = conn.execute(&sql, rusqlite::params_from_iter(values.iter().map(|v| v.as_ref())))
        .map_err(|e| e.to_string())?;
    if rows == 0 {
        return Err(format!("block not found: {}", id));
    }
    Ok(())
}

pub fn delete_block_impl(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM blocks WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn reorder_block_impl(conn: &Connection, id: &str, new_parent_id: Option<&str>, new_position: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE blocks SET parent_id = ?1, position = ?2 WHERE id = ?3",
        params![new_parent_id, new_position, id],
    ).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id FROM blocks WHERE parent_id IS ?1 AND id != ?2 ORDER BY position"
    ).map_err(|e| e.to_string())?;
    let sibling_ids: Vec<String> = stmt.query_map(params![new_parent_id, id], |r| r.get(0))
        .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    let mut pos = 0i64;
    for sid in &sibling_ids {
        if pos == new_position { pos += 1; }
        conn.execute("UPDATE blocks SET position = ?1 WHERE id = ?2", params![pos, sid])
            .map_err(|e| e.to_string())?;
        pos += 1;
    }
    Ok(())
}

pub fn reparent_block_impl(conn: &Connection, id: &str, new_parent_id: Option<&str>, position: i64) -> Result<(), String> {
    reorder_block_impl(conn, id, new_parent_id, position)
}

// Tauri command wrappers
#[tauri::command]
pub fn create_block(state: tauri::State<DbState>, daily_note_id: String, parent_id: Option<String>, content: String, block_type: String, position: i64) -> Result<Block, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    create_block_impl(&conn, &daily_note_id, parent_id.as_deref(), &content, &block_type, position)
}

#[tauri::command]
pub fn get_blocks_for_date(state: tauri::State<DbState>, date: String) -> Result<Vec<Block>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_blocks_for_date_impl(&conn, &date)
}

#[tauri::command]
pub fn update_block(state: tauri::State<DbState>, id: String, content: Option<String>, block_type: Option<String>, status: Option<String>, priority: Option<String>, due_date: Option<String>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    update_block_impl(&conn, &id, content.as_deref(), block_type.as_deref(), status.as_deref(), priority.as_deref(), due_date.as_deref())
}

#[tauri::command]
pub fn delete_block(state: tauri::State<DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    delete_block_impl(&conn, &id)
}

#[tauri::command]
pub fn reorder_block(state: tauri::State<DbState>, id: String, new_parent_id: Option<String>, new_position: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    reorder_block_impl(&conn, &id, new_parent_id.as_deref(), new_position)
}

#[tauri::command]
pub fn reparent_block(state: tauri::State<DbState>, id: String, new_parent_id: Option<String>, position: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    reparent_block_impl(&conn, &id, new_parent_id.as_deref(), position)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::commands::daily_notes::get_or_create_daily_note_impl;
    use tempfile::TempDir;

    fn test_conn() -> (TempDir, Connection) {
        let tmp = TempDir::new().unwrap();
        let conn = db::init_db(tmp.path().to_path_buf()).unwrap();
        (tmp, conn)
    }

    #[test]
    fn test_create_and_get_blocks() {
        let (_tmp, conn) = test_conn();
        let note = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let block = create_block_impl(&conn, &note.id, None, "hello world", "bullet", 0).unwrap();
        assert_eq!(block.content, "hello world");
        let blocks = get_blocks_for_date_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].content, "hello world");
    }

    #[test]
    fn test_nested_blocks() {
        let (_tmp, conn) = test_conn();
        let note = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let parent = create_block_impl(&conn, &note.id, None, "parent", "bullet", 0).unwrap();
        let child = create_block_impl(&conn, &note.id, Some(&parent.id), "child", "bullet", 0).unwrap();
        let _ = child;
        let blocks = get_blocks_for_date_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].children.len(), 1);
        assert_eq!(blocks[0].children[0].content, "child");
    }

    #[test]
    fn test_update_block() {
        let (_tmp, conn) = test_conn();
        let note = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let block = create_block_impl(&conn, &note.id, None, "original", "bullet", 0).unwrap();
        update_block_impl(&conn, &block.id, Some("updated"), None, None, None, None).unwrap();
        let blocks = get_blocks_for_date_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(blocks[0].content, "updated");
    }

    #[test]
    fn test_delete_block() {
        let (_tmp, conn) = test_conn();
        let note = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let block = create_block_impl(&conn, &note.id, None, "to delete", "bullet", 0).unwrap();
        delete_block_impl(&conn, &block.id).unwrap();
        let blocks = get_blocks_for_date_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(blocks.len(), 0);
    }

    #[test]
    fn test_reorder_blocks() {
        let (_tmp, conn) = test_conn();
        let note = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let a = create_block_impl(&conn, &note.id, None, "a", "bullet", 0).unwrap();
        let b = create_block_impl(&conn, &note.id, None, "b", "bullet", 1).unwrap();
        let c = create_block_impl(&conn, &note.id, None, "c", "bullet", 2).unwrap();
        let _ = (a, b);
        reorder_block_impl(&conn, &c.id, None, 0).unwrap();
        let blocks = get_blocks_for_date_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(blocks[0].content, "c");
        assert_eq!(blocks[1].content, "a");
        assert_eq!(blocks[2].content, "b");
    }
}
