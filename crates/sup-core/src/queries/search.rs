use anyhow::Result;
use rusqlite::params;
use crate::db::Database;
use crate::models::Node;
use super::nodes::{get_by_id, get_children, build_tree};

/// Full-text search nodes using FTS5. Returns matching nodes with their children attached.
pub fn search_nodes(db: &mut Database, query: &str) -> Result<Vec<Node>> {
    // FTS5 match query
    let ids: Vec<String> = {
        let mut stmt = db.conn.prepare(
            "SELECT n.id FROM nodes n \
             JOIN nodes_fts f ON n.rowid = f.rowid \
             WHERE nodes_fts MATCH ?1 ORDER BY rank"
        )?;
        let x = stmt.query_map(params![query], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?; x
    };

    let mut results = vec![];
    for id in ids {
        if let Some(mut node) = get_by_id(db, &id)? {
            let children = get_children(db, &node.id)?;
            node.children = build_tree(db, children)?;
            results.push(node);
        }
    }
    Ok(results)
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
    fn test_search_finds_matching_nodes() {
        let mut db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = daily_notes::get_or_create(&mut db, date).unwrap();
        nodes::create(&mut db, nodes::CreateNode {
            parent_id: None, daily_note_id: Some(note.id.clone()),
            content: "implement oauth flow".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        nodes::create(&mut db, nodes::CreateNode {
            parent_id: None, daily_note_id: Some(note.id),
            content: "review pull request".into(), node_type: NodeType::Bullet,
            position: 1, status: None, priority: None, due_date: None,
        }).unwrap();

        let results = search_nodes(&mut db, "oauth").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "implement oauth flow");
    }

    #[test]
    fn test_search_returns_children() {
        let mut db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = daily_notes::get_or_create(&mut db, date).unwrap();
        let parent = nodes::create(&mut db, nodes::CreateNode {
            parent_id: None, daily_note_id: Some(note.id.clone()),
            content: "authentication system".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        nodes::create(&mut db, nodes::CreateNode {
            parent_id: Some(parent.id.clone()), daily_note_id: Some(note.id),
            content: "jwt tokens".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();

        let results = search_nodes(&mut db, "authentication").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].children.len(), 1);
        assert_eq!(results[0].children[0].content, "jwt tokens");
    }

    #[test]
    fn test_search_no_results() {
        let mut db = test_db();
        let results = search_nodes(&mut db, "nonexistent").unwrap();
        assert_eq!(results.len(), 0);
    }
}
