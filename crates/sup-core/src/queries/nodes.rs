use anyhow::Result;
use chrono::{NaiveDate, Utc};
use rusqlite::params;
use uuid::Uuid;
use crate::db::Database;
use crate::models::{Node, NodeType, Priority, TaskStatus};

pub struct CreateNode {
    pub parent_id: Option<String>,
    pub daily_note_id: Option<String>,
    pub content: String,
    pub node_type: NodeType,
    pub position: i64,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub due_date: Option<NaiveDate>,
}

pub struct UpdateNode {
    pub content: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub due_date: Option<Option<NaiveDate>>,
    pub position: Option<i64>,
    pub parent_id: Option<Option<String>>,
}

fn row_to_node(row: &rusqlite::Row) -> rusqlite::Result<Node> {
    let node_type_str: String = row.get(4)?;
    let status_str: Option<String> = row.get(6)?;
    let priority_str: Option<String> = row.get(7)?;
    let due_date_str: Option<String> = row.get(8)?;
    let created_str: String = row.get(9)?;
    let updated_str: String = row.get(10)?;

    Ok(Node {
        id: row.get(0)?,
        parent_id: row.get(1)?,
        daily_note_id: row.get(2)?,
        content: row.get(3)?,
        node_type: NodeType::from_str(&node_type_str).unwrap_or(NodeType::Bullet),
        position: row.get(5)?,
        status: status_str.as_deref().and_then(|s| TaskStatus::from_str(s).ok()),
        priority: priority_str.as_deref().and_then(|s| Priority::from_str(s).ok()),
        due_date: due_date_str.as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
        created_at: created_str.parse().unwrap_or_else(|_| Utc::now()),
        updated_at: updated_str.parse().unwrap_or_else(|_| Utc::now()),
        tags: vec![],
        children: vec![],
    })
}

pub fn create(db: &mut Database, req: CreateNode) -> Result<Node> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    db.conn.execute(
        "INSERT INTO nodes (id, parent_id, daily_note_id, content, node_type, position, \
         status, priority, due_date, created_at, updated_at) \
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![
            id,
            req.parent_id,
            req.daily_note_id,
            req.content,
            req.node_type.as_str(),
            req.position,
            req.status.as_ref().map(|s| s.as_str()),
            req.priority.as_ref().map(|p| p.as_str()),
            req.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
            now, now,
        ],
    )?;
    get_by_id(db, &id)?.ok_or_else(|| anyhow::anyhow!("node not found after insert"))
}

pub fn get_by_id(db: &mut Database, id: &str) -> Result<Option<Node>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, parent_id, daily_note_id, content, node_type, position, \
         status, priority, due_date, created_at, updated_at \
         FROM nodes WHERE id = ?1"
    )?;
    let mut rows = stmt.query(params![id])?;
    Ok(rows.next()?.map(|r| row_to_node(r).unwrap()))
}

pub fn get_children(db: &mut Database, parent_id: &str) -> Result<Vec<Node>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, parent_id, daily_note_id, content, node_type, position, \
         status, priority, due_date, created_at, updated_at \
         FROM nodes WHERE parent_id = ?1 ORDER BY position ASC"
    )?;
    let rows = stmt.query_map(params![parent_id], |r| row_to_node(r))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_roots_for_day(db: &mut Database, daily_note_id: &str) -> Result<Vec<Node>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, parent_id, daily_note_id, content, node_type, position, \
         status, priority, due_date, created_at, updated_at \
         FROM nodes WHERE daily_note_id = ?1 AND parent_id IS NULL ORDER BY position ASC"
    )?;
    let rows = stmt.query_map(params![daily_note_id], |r| row_to_node(r))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_all_tasks(db: &mut Database, status_filter: Option<&TaskStatus>) -> Result<Vec<Node>> {
    let rows: Vec<Node> = if let Some(s) = status_filter {
        let mut stmt = db.conn.prepare(
            "SELECT id, parent_id, daily_note_id, content, node_type, position, \
             status, priority, due_date, created_at, updated_at \
             FROM nodes WHERE node_type = 'task' AND status = ?1 ORDER BY created_at DESC"
        )?;
        let collected = stmt.query_map(params![s.as_str()], |r| row_to_node(r))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        collected
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT id, parent_id, daily_note_id, content, node_type, position, \
             status, priority, due_date, created_at, updated_at \
             FROM nodes WHERE node_type = 'task' ORDER BY created_at DESC"
        )?;
        let collected = stmt.query_map([], |r| row_to_node(r))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        collected
    };
    Ok(rows)
}

pub fn update(db: &mut Database, id: &str, req: UpdateNode) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    if let Some(content) = req.content {
        db.conn.execute("UPDATE nodes SET content=?1, updated_at=?2 WHERE id=?3",
            params![content, now, id])?;
    }
    if let Some(status) = req.status {
        db.conn.execute("UPDATE nodes SET status=?1, updated_at=?2 WHERE id=?3",
            params![status.as_str(), now, id])?;
    }
    if let Some(priority) = req.priority {
        db.conn.execute("UPDATE nodes SET priority=?1, updated_at=?2 WHERE id=?3",
            params![priority.as_str(), now, id])?;
    }
    if let Some(due) = req.due_date {
        db.conn.execute("UPDATE nodes SET due_date=?1, updated_at=?2 WHERE id=?3",
            params![due.map(|d| d.format("%Y-%m-%d").to_string()), now, id])?;
    }
    if let Some(pos) = req.position {
        db.conn.execute("UPDATE nodes SET position=?1, updated_at=?2 WHERE id=?3",
            params![pos, now, id])?;
    }
    if let Some(parent) = req.parent_id {
        db.conn.execute("UPDATE nodes SET parent_id=?1, updated_at=?2 WHERE id=?3",
            params![parent, now, id])?;
    }
    Ok(())
}

pub fn delete(db: &mut Database, id: &str) -> Result<()> {
    db.conn.execute("DELETE FROM nodes WHERE id = ?1", params![id])?;
    Ok(())
}

/// Recursively build a node tree from a list of root nodes
pub fn build_tree(db: &mut Database, roots: Vec<Node>) -> Result<Vec<Node>> {
    let mut result = vec![];
    for mut node in roots {
        let children = get_children(db, &node.id)?;
        node.children = build_tree(db, children)?;
        result.push(node);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::models::{NodeType, TaskStatus, Priority};
    use crate::queries::daily_notes;
    use chrono::NaiveDate;

    fn test_db() -> Database { Database::open_in_memory().unwrap() }
    fn test_date() -> NaiveDate { NaiveDate::from_ymd_opt(2026, 3, 5).unwrap() }

    #[test]
    fn test_create_and_get_bullet_node() {
        let mut db = test_db();
        let note = daily_notes::get_or_create(&mut db, test_date()).unwrap();
        let node = create(&mut db, CreateNode {
            parent_id: None,
            daily_note_id: Some(note.id.clone()),
            content: "hello world".into(),
            node_type: NodeType::Bullet,
            position: 0,
            status: None,
            priority: None,
            due_date: None,
        }).unwrap();
        assert_eq!(node.content, "hello world");
        assert_eq!(node.node_type, NodeType::Bullet);

        let fetched = get_by_id(&mut db, &node.id).unwrap().unwrap();
        assert_eq!(fetched.id, node.id);
    }

    #[test]
    fn test_create_task_node() {
        let mut db = test_db();
        let note = daily_notes::get_or_create(&mut db, test_date()).unwrap();
        let node = create(&mut db, CreateNode {
            parent_id: None,
            daily_note_id: Some(note.id),
            content: "do the thing".into(),
            node_type: NodeType::Task,
            position: 0,
            status: Some(TaskStatus::Todo),
            priority: Some(Priority::High),
            due_date: None,
        }).unwrap();
        assert_eq!(node.status, Some(TaskStatus::Todo));
        assert_eq!(node.priority, Some(Priority::High));
    }

    #[test]
    fn test_get_children_ordered_by_position() {
        let mut db = test_db();
        let note = daily_notes::get_or_create(&mut db, test_date()).unwrap();
        let parent = create(&mut db, CreateNode {
            parent_id: None, daily_note_id: Some(note.id.clone()),
            content: "parent".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        create(&mut db, CreateNode {
            parent_id: Some(parent.id.clone()), daily_note_id: Some(note.id.clone()),
            content: "child b".into(), node_type: NodeType::Bullet,
            position: 1, status: None, priority: None, due_date: None,
        }).unwrap();
        create(&mut db, CreateNode {
            parent_id: Some(parent.id.clone()), daily_note_id: Some(note.id.clone()),
            content: "child a".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        let children = get_children(&mut db, &parent.id).unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].content, "child a"); // position 0 first
    }

    #[test]
    fn test_delete_node() {
        let mut db = test_db();
        let note = daily_notes::get_or_create(&mut db, test_date()).unwrap();
        let node = create(&mut db, CreateNode {
            parent_id: None, daily_note_id: Some(note.id),
            content: "bye".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        delete(&mut db, &node.id).unwrap();
        assert!(get_by_id(&mut db, &node.id).unwrap().is_none());
    }

    #[test]
    fn test_get_all_tasks_filters_correctly() {
        let mut db = test_db();
        let note = daily_notes::get_or_create(&mut db, test_date()).unwrap();
        create(&mut db, CreateNode {
            parent_id: None, daily_note_id: Some(note.id.clone()),
            content: "task one".into(), node_type: NodeType::Task,
            position: 0, status: Some(TaskStatus::Todo), priority: Some(Priority::High), due_date: None,
        }).unwrap();
        create(&mut db, CreateNode {
            parent_id: None, daily_note_id: Some(note.id.clone()),
            content: "not a task".into(), node_type: NodeType::Bullet,
            position: 1, status: None, priority: None, due_date: None,
        }).unwrap();
        let tasks = get_all_tasks(&mut db, None).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].content, "task one");
    }

    #[test]
    fn test_build_tree() {
        let mut db = test_db();
        let note = daily_notes::get_or_create(&mut db, test_date()).unwrap();
        let parent = create(&mut db, CreateNode {
            parent_id: None, daily_note_id: Some(note.id.clone()),
            content: "parent".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        create(&mut db, CreateNode {
            parent_id: Some(parent.id.clone()), daily_note_id: Some(note.id.clone()),
            content: "child".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        let roots = get_roots_for_day(&mut db, &note.id).unwrap();
        let tree = build_tree(&mut db, roots).unwrap();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].content, "child");
    }
}
