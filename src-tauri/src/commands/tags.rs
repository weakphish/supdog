use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use crate::models::Tag;
use crate::db::DbState;

/// Creates a tag, auto-creating parent tags for paths like "project/migration".
pub fn create_tag_impl(conn: &Connection, name: &str) -> Result<Tag, String> {
    let parts: Vec<&str> = name.split('/').collect();
    let mut parent_id: Option<String> = None;

    for i in 0..parts.len() {
        let path = parts[..=i].join("/");
        let existing: Option<(String, Option<String>)> = conn.query_row(
            "SELECT id, parent_id FROM tags WHERE name = ?1", params![path],
            |r| Ok((r.get(0)?, r.get(1)?))
        ).optional().map_err(|e| e.to_string())?;

        if let Some((id, _)) = existing {
            parent_id = Some(id);
        } else {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO tags (id, name, parent_id) VALUES (?1, ?2, ?3)",
                params![id, path, parent_id],
            ).map_err(|e| e.to_string())?;
            parent_id = Some(id);
        }
    }

    conn.query_row("SELECT id, name, parent_id FROM tags WHERE name = ?1", params![name],
        |r| Ok(Tag { id: r.get(0)?, name: r.get(1)?, parent_id: r.get(2)? })
    ).map_err(|e| e.to_string())
}

pub fn get_all_tags_impl(conn: &Connection) -> Result<Vec<Tag>, String> {
    let mut stmt = conn.prepare("SELECT id, name, parent_id FROM tags ORDER BY name")
        .map_err(|e| e.to_string())?;
    let tags = stmt.query_map([], |r| Ok(Tag { id: r.get(0)?, name: r.get(1)?, parent_id: r.get(2)? }))
        .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    Ok(tags)
}

pub fn add_tag_to_block_impl(conn: &Connection, block_id: &str, tag_id: &str) -> Result<(), String> {
    conn.execute("INSERT OR IGNORE INTO block_tags (block_id, tag_id) VALUES (?1, ?2)",
        params![block_id, tag_id]).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_tag_from_block_impl(conn: &Connection, block_id: &str, tag_id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM block_tags WHERE block_id = ?1 AND tag_id = ?2",
        params![block_id, tag_id]).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_tags_for_block_impl(conn: &Connection, block_id: &str) -> Result<Vec<Tag>, String> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.parent_id FROM tags t JOIN block_tags bt ON t.id = bt.tag_id WHERE bt.block_id = ?1"
    ).map_err(|e| e.to_string())?;
    let tags = stmt.query_map(params![block_id], |r| Ok(Tag { id: r.get(0)?, name: r.get(1)?, parent_id: r.get(2)? }))
        .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    Ok(tags)
}

/// Returns blocks tagged with `tag_name` or any child tag.
/// Result separates open tasks (with children) from regular blocks (reverse-chronological).
pub fn get_blocks_by_tag_impl(conn: &Connection, tag_name: &str) -> Result<serde_json::Value, String> {
    let mut stmt = conn.prepare(
        "WITH RECURSIVE tag_tree AS (
            SELECT id FROM tags WHERE name = ?1
            UNION ALL
            SELECT t.id FROM tags t JOIN tag_tree tt ON t.parent_id = tt.id
        )
        SELECT DISTINCT b.* FROM blocks b
        JOIN block_tags bt ON b.id = bt.block_id
        JOIN tag_tree tt ON bt.tag_id = tt.id
        ORDER BY b.created_at DESC"
    ).map_err(|e| e.to_string())?;

    let blocks: Vec<crate::models::Block> = stmt.query_map(params![tag_name], |row| {
        crate::commands::blocks::row_to_block(row)
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // Separate open tasks from other blocks
    let tasks: Vec<_> = blocks.iter().filter(|b| {
        matches!(b.status.as_ref(), Some(s) if *s == crate::models::TaskStatus::Todo || *s == crate::models::TaskStatus::InProgress)
    }).cloned().collect();
    let other: Vec<_> = blocks.iter().filter(|b| {
        b.block_type != crate::models::BlockType::Task ||
        matches!(b.status.as_ref(), Some(s) if *s == crate::models::TaskStatus::Done || *s == crate::models::TaskStatus::Cancelled)
    }).cloned().collect();

    // Attach children to each task
    let mut tasks_with_children: Vec<crate::models::Block> = vec![];
    let mut child_stmt = conn.prepare(
        "SELECT * FROM blocks WHERE parent_id = ?1 ORDER BY position"
    ).map_err(|e| e.to_string())?;
    for mut task in tasks {
        task.children = child_stmt.query_map(params![task.id], |r| crate::commands::blocks::row_to_block(r))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        tasks_with_children.push(task);
    }
    Ok(serde_json::json!({ "tasks": tasks_with_children, "blocks": other }))
}

// Tauri command wrappers
#[tauri::command]
pub fn create_tag(state: tauri::State<DbState>, name: String) -> Result<Tag, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    create_tag_impl(&conn, &name)
}

#[tauri::command]
pub fn get_all_tags(state: tauri::State<DbState>) -> Result<Vec<Tag>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_all_tags_impl(&conn)
}

#[tauri::command]
pub fn add_tag_to_block(state: tauri::State<DbState>, block_id: String, tag_id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    add_tag_to_block_impl(&conn, &block_id, &tag_id)
}

#[tauri::command]
pub fn remove_tag_from_block(state: tauri::State<DbState>, block_id: String, tag_id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    remove_tag_from_block_impl(&conn, &block_id, &tag_id)
}

#[tauri::command]
pub fn get_tags_for_block(state: tauri::State<DbState>, block_id: String) -> Result<Vec<Tag>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_tags_for_block_impl(&conn, &block_id)
}

#[tauri::command]
pub fn get_blocks_by_tag(state: tauri::State<DbState>, tag_name: String) -> Result<serde_json::Value, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_blocks_by_tag_impl(&conn, &tag_name)
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
    fn test_create_tag_with_hierarchy() {
        let (_tmp, conn) = test_conn();
        let tag = create_tag_impl(&conn, "project/migration").unwrap();
        assert_eq!(tag.name, "project/migration");

        let parent: String = conn
            .query_row("SELECT name FROM tags WHERE id = ?1", params![tag.parent_id.unwrap()], |r| r.get(0))
            .unwrap();
        assert_eq!(parent, "project");
    }

    #[test]
    fn test_add_tag_to_block() {
        let (_tmp, conn) = test_conn();
        let note = crate::commands::daily_notes::get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let block = crate::commands::blocks::create_block_impl(&conn, &note.id, None, "test", "bullet", 0).unwrap();
        let tag = create_tag_impl(&conn, "work").unwrap();
        add_tag_to_block_impl(&conn, &block.id, &tag.id).unwrap();

        let tags = get_tags_for_block_impl(&conn, &block.id).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "work");
    }

    #[test]
    fn test_get_blocks_by_tag() {
        let (_tmp, conn) = test_conn();
        let note = crate::commands::daily_notes::get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let block = crate::commands::blocks::create_block_impl(&conn, &note.id, None, "tagged block", "bullet", 0).unwrap();
        let tag = create_tag_impl(&conn, "work").unwrap();
        add_tag_to_block_impl(&conn, &block.id, &tag.id).unwrap();

        let result = get_blocks_by_tag_impl(&conn, "work").unwrap();
        let blocks = result["blocks"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["content"].as_str().unwrap(), "tagged block");
    }
}
