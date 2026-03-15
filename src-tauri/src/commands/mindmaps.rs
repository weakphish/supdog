use rusqlite::{params, Connection};
use uuid::Uuid;
use crate::models::{MindMap, MindMapNode, BlockType};
use crate::db::DbState;

#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeWithBlock {
    pub node: MindMapNode,
    pub block: crate::models::Block,
}

pub fn create_mind_map_impl(conn: &Connection, name: &str) -> Result<MindMap, String> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO mind_maps (id, name) VALUES (?1, ?2)",
        params![id, name],
    ).map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, name, created_at, updated_at FROM mind_maps WHERE id = ?1",
        params![id],
        |row| Ok(MindMap {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
        }),
    ).map_err(|e| e.to_string())
}

pub fn get_mind_maps_impl(conn: &Connection) -> Result<Vec<MindMap>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, created_at, updated_at FROM mind_maps ORDER BY updated_at DESC"
    ).map_err(|e| e.to_string())?;

    let maps = stmt.query_map([], |row| Ok(MindMap {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
    })).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(maps)
}

pub fn delete_mind_map_impl(conn: &Connection, id: &str) -> Result<(), String> {
    let rows = conn.execute("DELETE FROM mind_maps WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    if rows == 0 {
        return Err(format!("mind map not found: {}", id));
    }
    Ok(())
}

pub fn add_mind_map_node_impl(conn: &Connection, mind_map_id: &str, content: &str, x: f64, y: f64) -> Result<MindMapNode, String> {
    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;

    let block_id = Uuid::new_v4().to_string();
    if let Err(e) = conn.execute(
        "INSERT INTO blocks (id, content, block_type, position) VALUES (?1, ?2, ?3, ?4)",
        params![block_id, content, "bullet", 0i64],
    ) {
        conn.execute("ROLLBACK", []).ok();
        return Err(e.to_string());
    }

    let node_id = Uuid::new_v4().to_string();
    if let Err(e) = conn.execute(
        "INSERT INTO mind_map_nodes (id, mind_map_id, block_id, x, y) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![node_id, mind_map_id, block_id, x, y],
    ) {
        conn.execute("ROLLBACK", []).ok();
        return Err(e.to_string());
    }

    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;

    Ok(MindMapNode {
        id: node_id,
        mind_map_id: mind_map_id.to_string(),
        block_id,
        x,
        y,
    })
}

pub fn update_node_position_impl(conn: &Connection, node_id: &str, x: f64, y: f64) -> Result<(), String> {
    let rows = conn.execute(
        "UPDATE mind_map_nodes SET x = ?1, y = ?2 WHERE id = ?3",
        params![x, y, node_id],
    ).map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("node not found: {}", node_id));
    }
    Ok(())
}

pub fn get_mind_map_nodes_impl(conn: &Connection, mind_map_id: &str) -> Result<Vec<MindMapNode>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, mind_map_id, block_id, x, y FROM mind_map_nodes WHERE mind_map_id = ?1"
    ).map_err(|e| e.to_string())?;

    let nodes = stmt.query_map(params![mind_map_id], |row| Ok(MindMapNode {
        id: row.get(0)?,
        mind_map_id: row.get(1)?,
        block_id: row.get(2)?,
        x: row.get(3)?,
        y: row.get(4)?,
    })).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(nodes)
}

pub fn get_mind_map_nodes_with_blocks_impl(conn: &Connection, mind_map_id: &str) -> Result<Vec<NodeWithBlock>, String> {
    let mut stmt = conn.prepare(
        "SELECT mmn.id, mmn.mind_map_id, mmn.block_id, mmn.x, mmn.y, b.id, b.content, b.block_type, b.parent_id, b.daily_note_id, b.position, b.status, b.priority, b.due_date, b.created_at, b.updated_at FROM mind_map_nodes mmn JOIN blocks b ON mmn.block_id = b.id WHERE mmn.mind_map_id = ?1"
    ).map_err(|e| e.to_string())?;

    let results = stmt.query_map(params![mind_map_id], |row| {
        let node = MindMapNode {
            id: row.get(0)?,
            mind_map_id: row.get(1)?,
            block_id: row.get(2)?,
            x: row.get(3)?,
            y: row.get(4)?,
        };
        let block = crate::models::Block {
            id: row.get(5)?,
            content: row.get(6)?,
            block_type: serde_json::from_value(
                serde_json::Value::String(row.get::<_, String>(7)?)
            ).unwrap_or(BlockType::Bullet),
            parent_id: row.get(8)?,
            daily_note_id: row.get(9)?,
            position: row.get(10)?,
            status: row.get::<_, Option<String>>(11)?
                .and_then(|s| serde_json::from_value(serde_json::Value::String(s)).ok()),
            priority: row.get::<_, Option<String>>(12)?
                .and_then(|s| serde_json::from_value(serde_json::Value::String(s)).ok()),
            due_date: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
            tags: vec![],
            children: vec![],
        };
        Ok(NodeWithBlock { node, block })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(results)
}

pub fn send_nodes_to_journal_impl(conn: &Connection, block_ids: &[String], date: &str) -> Result<(), String> {
    let daily_note = crate::commands::daily_notes::get_or_create_daily_note_impl(conn, date)?;

    // Find the max position in that daily note so we append
    let max_pos: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) FROM blocks WHERE daily_note_id = ?1",
        params![daily_note.id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;

    let result = (|| {
        for (i, block_id) in block_ids.iter().enumerate() {
            let new_pos = max_pos + 1 + i as i64;
            conn.execute(
                "UPDATE blocks SET daily_note_id = ?1, position = ?2, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?3",
                params![daily_note.id, new_pos, block_id],
            ).map_err(|e| e.to_string())?;

            conn.execute(
                "DELETE FROM mind_map_nodes WHERE block_id = ?1",
                params![block_id],
            ).map_err(|e| e.to_string())?;
        }
        Ok(())
    })();

    if let Err(e) = result {
        conn.execute("ROLLBACK", []).ok();
        return Err(e);
    }

    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;

    Ok(())
}

// Tauri command wrappers

#[tauri::command]
pub fn create_mind_map(state: tauri::State<DbState>, name: String) -> Result<MindMap, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    create_mind_map_impl(&conn, &name)
}

#[tauri::command]
pub fn get_mind_maps(state: tauri::State<DbState>) -> Result<Vec<MindMap>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_mind_maps_impl(&conn)
}

#[tauri::command]
pub fn delete_mind_map(state: tauri::State<DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    delete_mind_map_impl(&conn, &id)
}

#[tauri::command]
pub fn add_mind_map_node(state: tauri::State<DbState>, mind_map_id: String, content: String, x: f64, y: f64) -> Result<MindMapNode, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    add_mind_map_node_impl(&conn, &mind_map_id, &content, x, y)
}

#[tauri::command]
pub fn update_node_position(state: tauri::State<DbState>, node_id: String, x: f64, y: f64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    update_node_position_impl(&conn, &node_id, x, y)
}

#[tauri::command]
pub fn get_mind_map_nodes(state: tauri::State<DbState>, mind_map_id: String) -> Result<Vec<MindMapNode>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_mind_map_nodes_impl(&conn, &mind_map_id)
}

#[tauri::command]
pub fn get_mind_map_nodes_with_blocks(state: tauri::State<DbState>, mind_map_id: String) -> Result<Vec<NodeWithBlock>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_mind_map_nodes_with_blocks_impl(&conn, &mind_map_id)
}

#[tauri::command]
pub fn send_nodes_to_journal(state: tauri::State<DbState>, block_ids: Vec<String>, date: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    send_nodes_to_journal_impl(&conn, &block_ids, &date)
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
    fn test_create_mind_map() {
        let (_tmp, conn) = test_conn();
        let mm = create_mind_map_impl(&conn, "brainstorm").unwrap();
        assert_eq!(mm.name, "brainstorm");
    }

    #[test]
    fn test_add_node_to_mind_map() {
        let (_tmp, conn) = test_conn();
        let mm = create_mind_map_impl(&conn, "brainstorm").unwrap();
        let node = add_mind_map_node_impl(&conn, &mm.id, "idea", 100.0, 200.0).unwrap();
        assert_eq!(node.x, 100.0);
        assert_eq!(node.y, 200.0);
        let nodes = get_mind_map_nodes_impl(&conn, &mm.id).unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_send_to_journal() {
        let (_tmp, conn) = test_conn();
        let mm = create_mind_map_impl(&conn, "brainstorm").unwrap();
        let node = add_mind_map_node_impl(&conn, &mm.id, "idea to journal", 0.0, 0.0).unwrap();

        send_nodes_to_journal_impl(&conn, &[node.block_id.clone()], "2026-03-15").unwrap();

        let nodes = get_mind_map_nodes_impl(&conn, &mm.id).unwrap();
        assert_eq!(nodes.len(), 0);

        let journal_blocks = crate::commands::blocks::get_blocks_for_date_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(journal_blocks.len(), 1);
        assert_eq!(journal_blocks[0].content, "idea to journal");
    }
}
