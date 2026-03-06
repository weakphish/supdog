use anyhow::Result;
use rusqlite::params;
use uuid::Uuid;
use crate::db::Database;
use crate::models::Tag;

fn row_to_tag(row: &rusqlite::Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
    })
}

/// Get or create a tag by full path, ensuring all ancestor tags also exist.
/// e.g. "projects/foo/bar" creates "projects", "projects/foo", "projects/foo/bar"
pub fn get_or_create(db: &mut Database, full_path: &str) -> Result<Tag> {
    let parts: Vec<&str> = full_path.split('/').collect();
    let mut parent_id: Option<String> = None;
    let mut last_tag: Option<Tag> = None;

    for i in 0..parts.len() {
        let path = parts[..=i].join("/");
        let existing = get_by_name(db, &path)?;
        let tag = match existing {
            Some(t) => t,
            None => {
                let id = Uuid::new_v4().to_string();
                db.conn.execute(
                    "INSERT INTO tags (id, name, parent_id) VALUES (?1, ?2, ?3)",
                    params![id, path, parent_id],
                )?;
                Tag { id, name: path, parent_id: parent_id.clone() }
            }
        };
        parent_id = Some(tag.id.clone());
        last_tag = Some(tag);
    }
    Ok(last_tag.unwrap())
}

pub fn get_by_name(db: &mut Database, name: &str) -> Result<Option<Tag>> {
    let mut stmt = db.conn.prepare("SELECT id, name, parent_id FROM tags WHERE name = ?1")?;
    let mut rows = stmt.query(params![name])?;
    Ok(rows.next()?.map(|r| row_to_tag(r).unwrap()))
}

/// Returns the tag with the given name AND all its descendants (e.g. searching
/// "projects" returns "projects", "projects/foo", "projects/foo/bar")
pub fn get_all_with_prefix(db: &mut Database, prefix: &str) -> Result<Vec<Tag>> {
    let like = format!("{}/%", prefix);
    let mut stmt = db.conn.prepare(
        "SELECT id, name, parent_id FROM tags WHERE name = ?1 OR name LIKE ?2 ORDER BY name"
    )?;
    let rows = stmt.query_map(params![prefix, like], |r| row_to_tag(r))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn add_tag_to_node(db: &mut Database, node_id: &str, tag_id: &str) -> Result<()> {
    db.conn.execute(
        "INSERT OR IGNORE INTO node_tags (node_id, tag_id) VALUES (?1, ?2)",
        params![node_id, tag_id],
    )?;
    Ok(())
}

pub fn remove_tag_from_node(db: &mut Database, node_id: &str, tag_id: &str) -> Result<()> {
    db.conn.execute(
        "DELETE FROM node_tags WHERE node_id = ?1 AND tag_id = ?2",
        params![node_id, tag_id],
    )?;
    Ok(())
}

pub fn get_tags_for_node(db: &mut Database, node_id: &str) -> Result<Vec<Tag>> {
    let mut stmt = db.conn.prepare(
        "SELECT t.id, t.name, t.parent_id FROM tags t \
         JOIN node_tags nt ON t.id = nt.tag_id WHERE nt.node_id = ?1 ORDER BY t.name"
    )?;
    let rows = stmt.query_map(params![node_id], |r| row_to_tag(r))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Returns all node IDs tagged with the given prefix (including descendants)
pub fn get_nodes_for_tag_prefix(db: &mut Database, prefix: &str) -> Result<Vec<String>> {
    let tags = get_all_with_prefix(db, prefix)?;
    let tag_ids: Vec<String> = tags.into_iter().map(|t| t.id).collect();
    if tag_ids.is_empty() {
        return Ok(vec![]);
    }
    // Build parameterized IN query
    let placeholders: String = tag_ids.iter().enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!("SELECT DISTINCT node_id FROM node_tags WHERE tag_id IN ({})", placeholders);
    let mut stmt = db.conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(tag_ids.iter()), |r| r.get::<_, String>(0))?;
    let ids = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::queries::{daily_notes, nodes};
    use crate::models::NodeType;
    use chrono::NaiveDate;

    fn test_db() -> Database { Database::open_in_memory().unwrap() }

    #[test]
    fn test_get_or_create_creates_hierarchy() {
        let mut db = test_db();
        let tag = get_or_create(&mut db, "projects/foo/bar").unwrap();
        assert_eq!(tag.name, "projects/foo/bar");

        let parent = get_by_name(&mut db, "projects/foo").unwrap();
        assert!(parent.is_some(), "projects/foo should exist");

        let grandparent = get_by_name(&mut db, "projects").unwrap();
        assert!(grandparent.is_some(), "projects should exist");
    }

    #[test]
    fn test_get_or_create_idempotent() {
        let mut db = test_db();
        let t1 = get_or_create(&mut db, "work").unwrap();
        let t2 = get_or_create(&mut db, "work").unwrap();
        assert_eq!(t1.id, t2.id);
    }

    #[test]
    fn test_get_all_with_prefix_includes_descendants() {
        let mut db = test_db();
        get_or_create(&mut db, "projects/foo").unwrap();
        get_or_create(&mut db, "projects/bar").unwrap();
        get_or_create(&mut db, "work").unwrap();

        let results = get_all_with_prefix(&mut db, "projects").unwrap();
        // should include "projects", "projects/foo", "projects/bar"
        assert_eq!(results.len(), 3);
        let names: Vec<&str> = results.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"projects"));
        assert!(names.contains(&"projects/foo"));
        assert!(names.contains(&"projects/bar"));
        assert!(!names.contains(&"work"));
    }

    #[test]
    fn test_add_and_get_tags_for_node() {
        let mut db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = daily_notes::get_or_create(&mut db, date).unwrap();
        let node = nodes::create(&mut db, nodes::CreateNode {
            parent_id: None, daily_note_id: Some(note.id),
            content: "test node".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();

        let tag = get_or_create(&mut db, "projects/foo").unwrap();
        add_tag_to_node(&mut db, &node.id, &tag.id).unwrap();

        let tags = get_tags_for_node(&mut db, &node.id).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "projects/foo");
    }

    #[test]
    fn test_remove_tag_from_node() {
        let mut db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = daily_notes::get_or_create(&mut db, date).unwrap();
        let node = nodes::create(&mut db, nodes::CreateNode {
            parent_id: None, daily_note_id: Some(note.id),
            content: "tagged node".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        let tag = get_or_create(&mut db, "work").unwrap();
        add_tag_to_node(&mut db, &node.id, &tag.id).unwrap();
        remove_tag_from_node(&mut db, &node.id, &tag.id).unwrap();
        let tags = get_tags_for_node(&mut db, &node.id).unwrap();
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_get_nodes_for_tag_prefix() {
        let mut db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = daily_notes::get_or_create(&mut db, date).unwrap();
        let node1 = nodes::create(&mut db, nodes::CreateNode {
            parent_id: None, daily_note_id: Some(note.id.clone()),
            content: "n1".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        let node2 = nodes::create(&mut db, nodes::CreateNode {
            parent_id: None, daily_note_id: Some(note.id),
            content: "n2".into(), node_type: NodeType::Bullet,
            position: 1, status: None, priority: None, due_date: None,
        }).unwrap();

        let tag_foo = get_or_create(&mut db, "projects/foo").unwrap();
        let tag_bar = get_or_create(&mut db, "projects/bar").unwrap();
        add_tag_to_node(&mut db, &node1.id, &tag_foo.id).unwrap();
        add_tag_to_node(&mut db, &node2.id, &tag_bar.id).unwrap();

        let node_ids = get_nodes_for_tag_prefix(&mut db, "projects").unwrap();
        assert_eq!(node_ids.len(), 2);
    }
}
