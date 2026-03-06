# sup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build `sup`, a terminal knowledge base and engineering journal with a CLI for quick logging and a ratatui TUI for rich viewing/editing, backed by SQLite.

**Architecture:** Cargo workspace with `sup-core` (data model, SQLite, query layer) and `sup` (CLI + TUI binary). TEA pattern for TUI screens. Tasks are nodes with `node_type = 'task'` living in the outline tree.

**Tech Stack:** Rust, ratatui, crossterm, clap (derive), rusqlite (bundled), rusqlite_migration, serde + serde_json, chrono, anyhow, uuid

**Design doc:** `docs/plans/2026-03-05-sup-design.md`

---

## Phase 1: Workspace + sup-core Foundation

### Task 1: Cargo Workspace Scaffold

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/sup-core/Cargo.toml`
- Create: `crates/sup-core/src/lib.rs`
- Create: `crates/sup/Cargo.toml`
- Create: `crates/sup/src/main.rs`

**Step 1: Write the workspace Cargo.toml**

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["crates/sup-core", "crates/sup"]
resolver = "2"

[workspace.dependencies]
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
rusqlite = { version = "0.31", features = ["bundled"] }
rusqlite_migration = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
```

**Step 2: Create sup-core crate**

```toml
# crates/sup-core/Cargo.toml
[package]
name = "sup-core"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = { workspace = true }
chrono = { workspace = true }
rusqlite = { workspace = true }
rusqlite_migration = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
```

```rust
// crates/sup-core/src/lib.rs
pub mod db;
pub mod models;
pub mod queries;
```

**Step 3: Create sup binary crate**

```toml
# crates/sup/Cargo.toml
[package]
name = "sup"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "sup"
path = "src/main.rs"

[dependencies]
sup-core = { path = "../sup-core" }
anyhow = { workspace = true }
clap = { version = "4", features = ["derive"] }
serde_json = { workspace = true }
ratatui = "0.29"
crossterm = "0.28"
```

```rust
// crates/sup/src/main.rs
fn main() {
    println!("sup");
}
```

**Step 4: Verify it compiles**

Run: `cargo build`
Expected: compiles with no errors

**Step 5: Commit**

```bash
git add Cargo.toml crates/
git commit -m "feat: scaffold cargo workspace with sup-core and sup crates"
```

---

### Task 2: SQLite Schema + Migrations

**Files:**
- Create: `crates/sup-core/src/db.rs`
- Modify: `crates/sup-core/src/lib.rs`

**Step 1: Write a test for DB initialization**

```rust
// crates/sup-core/src/db.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_db_creates_schema() {
        let db = Database::open_in_memory().unwrap();
        // verify tables exist
        let count: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='nodes'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sup-core test_open_db_creates_schema`
Expected: FAIL — `Database` not defined

**Step 3: Implement Database struct + migrations**

```rust
// crates/sup-core/src/db.rs
use anyhow::Result;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let migrations = Migrations::new(vec![
            M::up(include_str!("migrations/001_initial.sql")),
        ]);
        migrations.to_latest(&self.conn)?;
        Ok(())
    }
}
```

**Step 4: Create the migration SQL**

```bash
mkdir -p crates/sup-core/src/migrations
```

```sql
-- crates/sup-core/src/migrations/001_initial.sql
CREATE TABLE daily_notes (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL UNIQUE
);

CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    parent_id TEXT REFERENCES nodes(id) ON DELETE CASCADE,
    daily_note_id TEXT REFERENCES daily_notes(id),
    content TEXT NOT NULL DEFAULT '',
    node_type TEXT NOT NULL DEFAULT 'bullet',
    position INTEGER NOT NULL DEFAULT 0,
    status TEXT,
    priority TEXT,
    due_date TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    parent_id TEXT REFERENCES tags(id)
);

CREATE TABLE node_tags (
    node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (node_id, tag_id)
);

CREATE TABLE node_links (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    target_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    UNIQUE (source_id, target_id)
);

CREATE INDEX idx_nodes_parent ON nodes(parent_id);
CREATE INDEX idx_nodes_daily_note ON nodes(daily_note_id);
CREATE INDEX idx_nodes_type ON nodes(node_type);
CREATE VIRTUAL TABLE nodes_fts USING fts5(content, content=nodes, content_rowid=rowid);

CREATE TRIGGER nodes_fts_insert AFTER INSERT ON nodes BEGIN
    INSERT INTO nodes_fts(rowid, content) VALUES (new.rowid, new.content);
END;
CREATE TRIGGER nodes_fts_update AFTER UPDATE ON nodes BEGIN
    INSERT INTO nodes_fts(nodes_fts, rowid, content) VALUES ('delete', old.rowid, old.content);
    INSERT INTO nodes_fts(rowid, content) VALUES (new.rowid, new.content);
END;
CREATE TRIGGER nodes_fts_delete AFTER DELETE ON nodes BEGIN
    INSERT INTO nodes_fts(nodes_fts, rowid, content) VALUES ('delete', old.rowid, old.content);
END;
```

Note: `include_str!` requires the path relative to the source file. Add `#![allow(unused)]` if needed during dev.

**Step 5: Run test to verify it passes**

Run: `cargo test -p sup-core test_open_db_creates_schema`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/sup-core/src/db.rs crates/sup-core/src/migrations/
git commit -m "feat: sqlite schema with migrations and fts5 support"
```

---

### Task 3: Domain Types

**Files:**
- Create: `crates/sup-core/src/models.rs`
- Modify: `crates/sup-core/src/lib.rs`

**Step 1: Write tests for model construction**

```rust
// crates/sup-core/src/models.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_roundtrip() {
        assert_eq!(NodeType::Task.as_str(), "task");
        assert_eq!(NodeType::from_str("task").unwrap(), NodeType::Task);
        assert!(NodeType::from_str("bogus").is_err());
    }

    #[test]
    fn test_task_status_cycle() {
        assert_eq!(TaskStatus::Todo.next(), TaskStatus::InProgress);
        assert_eq!(TaskStatus::InProgress.next(), TaskStatus::Done);
        assert_eq!(TaskStatus::Done.next(), TaskStatus::Cancelled);
        assert_eq!(TaskStatus::Cancelled.next(), TaskStatus::Todo);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sup-core test_node_type_roundtrip test_task_status_cycle`
Expected: FAIL

**Step 3: Implement domain types**

```rust
// crates/sup-core/src/models.rs
use anyhow::{bail, Result};
use chrono::{NaiveDate, DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyNote {
    pub id: String,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    Bullet,
    H1,
    H2,
    H3,
    Quote,
    Code,
    Task,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Bullet => "bullet",
            NodeType::H1 => "h1",
            NodeType::H2 => "h2",
            NodeType::H3 => "h3",
            NodeType::Quote => "quote",
            NodeType::Code => "code",
            NodeType::Task => "task",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "bullet" => Ok(NodeType::Bullet),
            "h1" => Ok(NodeType::H1),
            "h2" => Ok(NodeType::H2),
            "h3" => Ok(NodeType::H3),
            "quote" => Ok(NodeType::Quote),
            "code" => Ok(NodeType::Code),
            "task" => Ok(NodeType::Task),
            _ => bail!("unknown node type: {}", s),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            NodeType::Bullet => "•",
            NodeType::H1 | NodeType::H2 | NodeType::H3 => "#",
            NodeType::Quote => "\"",
            NodeType::Code => ">",
            NodeType::Task => "☐",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Done => "done",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "todo" => Ok(TaskStatus::Todo),
            "in_progress" => Ok(TaskStatus::InProgress),
            "done" => Ok(TaskStatus::Done),
            "cancelled" => Ok(TaskStatus::Cancelled),
            _ => bail!("unknown status: {}", s),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            TaskStatus::Todo => TaskStatus::InProgress,
            TaskStatus::InProgress => TaskStatus::Done,
            TaskStatus::Done => TaskStatus::Cancelled,
            TaskStatus::Cancelled => TaskStatus::Todo,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "☐",
            TaskStatus::InProgress => "◐",
            TaskStatus::Done => "☑",
            TaskStatus::Cancelled => "✗",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    High,
    Med,
    Low,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self { Priority::High => "high", Priority::Med => "med", Priority::Low => "low" }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "high" => Ok(Priority::High),
            "med" => Ok(Priority::Med),
            "low" => Ok(Priority::Low),
            _ => bail!("unknown priority: {}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub parent_id: Option<String>,
    pub daily_note_id: Option<String>,
    pub content: String,
    pub node_type: NodeType,
    pub position: i64,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,       // populated by query layer
    pub children: Vec<Node>,     // populated on demand
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sup-core`
Expected: all pass

**Step 5: Commit**

```bash
git add crates/sup-core/src/models.rs
git commit -m "feat: domain types - Node, NodeType, TaskStatus, Priority, Tag"
```

---

## Phase 2: Query Layer

### Task 4: DailyNote Queries

**Files:**
- Create: `crates/sup-core/src/queries/daily_notes.rs`
- Create: `crates/sup-core/src/queries/mod.rs`
- Modify: `crates/sup-core/src/lib.rs`

**Step 1: Write failing tests**

```rust
// crates/sup-core/src/queries/daily_notes.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use chrono::NaiveDate;

    fn test_db() -> Database { Database::open_in_memory().unwrap() }

    #[test]
    fn test_get_or_create_today() {
        let db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = get_or_create(&db, date).unwrap();
        assert_eq!(note.date, date);

        // idempotent
        let note2 = get_or_create(&db, date).unwrap();
        assert_eq!(note.id, note2.id);
    }

    #[test]
    fn test_get_by_date() {
        let db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        assert!(get_by_date(&db, date).unwrap().is_none());
        get_or_create(&db, date).unwrap();
        assert!(get_by_date(&db, date).unwrap().is_some());
    }
}
```

**Step 2: Run to verify failure**

Run: `cargo test -p sup-core daily_notes`
Expected: FAIL

**Step 3: Implement**

```rust
// crates/sup-core/src/queries/daily_notes.rs
use anyhow::Result;
use chrono::NaiveDate;
use uuid::Uuid;
use crate::db::Database;
use crate::models::DailyNote;

pub fn get_or_create(db: &Database, date: NaiveDate) -> Result<DailyNote> {
    if let Some(note) = get_by_date(db, date)? {
        return Ok(note);
    }
    let id = Uuid::new_v4().to_string();
    let date_str = date.format("%Y-%m-%d").to_string();
    db.conn.execute(
        "INSERT INTO daily_notes (id, date) VALUES (?1, ?2)",
        rusqlite::params![id, date_str],
    )?;
    Ok(DailyNote { id, date })
}

pub fn get_by_date(db: &Database, date: NaiveDate) -> Result<Option<DailyNote>> {
    let date_str = date.format("%Y-%m-%d").to_string();
    let mut stmt = db.conn.prepare("SELECT id, date FROM daily_notes WHERE date = ?1")?;
    let mut rows = stmt.query(rusqlite::params![date_str])?;
    if let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let date_str: String = row.get(1)?;
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")?;
        Ok(Some(DailyNote { id, date }))
    } else {
        Ok(None)
    }
}

pub fn list_all(db: &Database) -> Result<Vec<DailyNote>> {
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
```

```rust
// crates/sup-core/src/queries/mod.rs
pub mod daily_notes;
pub mod nodes;
pub mod tags;
pub mod search;
```

**Step 4: Run tests**

Run: `cargo test -p sup-core daily_notes`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/sup-core/src/queries/
git commit -m "feat: daily note get-or-create and list queries"
```

---

### Task 5: Node CRUD Queries

**Files:**
- Create: `crates/sup-core/src/queries/nodes.rs`

**Step 1: Write failing tests**

```rust
// crates/sup-core/src/queries/nodes.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::models::{NodeType, TaskStatus, Priority};
    use crate::queries::daily_notes;
    use chrono::{NaiveDate, Utc};

    fn test_db() -> Database { Database::open_in_memory().unwrap() }

    #[test]
    fn test_create_and_get_bullet_node() {
        let db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = daily_notes::get_or_create(&db, date).unwrap();
        let node = create(&db, CreateNode {
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

        let fetched = get_by_id(&db, &node.id).unwrap().unwrap();
        assert_eq!(fetched.id, node.id);
    }

    #[test]
    fn test_create_task_node() {
        let db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = daily_notes::get_or_create(&db, date).unwrap();
        let node = create(&db, CreateNode {
            parent_id: None,
            daily_note_id: Some(note.id.clone()),
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
    fn test_get_children() {
        let db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = daily_notes::get_or_create(&db, date).unwrap();
        let parent = create(&db, CreateNode {
            parent_id: None, daily_note_id: Some(note.id.clone()),
            content: "parent".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        create(&db, CreateNode {
            parent_id: Some(parent.id.clone()), daily_note_id: Some(note.id.clone()),
            content: "child".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        let children = get_children(&db, &parent.id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].content, "child");
    }

    #[test]
    fn test_delete_node() {
        let db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = daily_notes::get_or_create(&db, date).unwrap();
        let node = create(&db, CreateNode {
            parent_id: None, daily_note_id: Some(note.id.clone()),
            content: "bye".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        delete(&db, &node.id).unwrap();
        assert!(get_by_id(&db, &node.id).unwrap().is_none());
    }

    #[test]
    fn test_get_all_tasks() {
        let db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = daily_notes::get_or_create(&db, date).unwrap();
        create(&db, CreateNode {
            parent_id: None, daily_note_id: Some(note.id.clone()),
            content: "task one".into(), node_type: NodeType::Task,
            position: 0, status: Some(TaskStatus::Todo), priority: Some(Priority::High), due_date: None,
        }).unwrap();
        create(&db, CreateNode {
            parent_id: None, daily_note_id: Some(note.id.clone()),
            content: "not a task".into(), node_type: NodeType::Bullet,
            position: 1, status: None, priority: None, due_date: None,
        }).unwrap();
        let tasks = get_all_tasks(&db, None).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].content, "task one");
    }
}
```

**Step 2: Run to verify failure**

Run: `cargo test -p sup-core nodes`
Expected: FAIL

**Step 3: Implement node queries**

```rust
// crates/sup-core/src/queries/nodes.rs
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
    pub due_date: Option<NaiveDate>,
    pub position: Option<i64>,
    pub parent_id: Option<Option<String>>,
}

fn row_to_node(row: &rusqlite::Row) -> rusqlite::Result<Node> {
    let node_type_str: String = row.get(4)?;
    let status_str: Option<String> = row.get(7)?;
    let priority_str: Option<String> = row.get(8)?;
    let due_date_str: Option<String> = row.get(9)?;
    let created_str: String = row.get(10)?;
    let updated_str: String = row.get(11)?;

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

const SELECT: &str = "SELECT id, parent_id, daily_note_id, content, node_type, position, \
                       node_type, status, priority, due_date, created_at, updated_at FROM nodes";

pub fn create(db: &Database, req: CreateNode) -> Result<Node> {
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

pub fn get_by_id(db: &Database, id: &str) -> Result<Option<Node>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, parent_id, daily_note_id, content, node_type, position, \
         node_type, status, priority, due_date, created_at, updated_at \
         FROM nodes WHERE id = ?1"
    )?;
    let mut rows = stmt.query(params![id])?;
    Ok(rows.next()?.map(|r| row_to_node(r).unwrap()))
}

pub fn get_children(db: &Database, parent_id: &str) -> Result<Vec<Node>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, parent_id, daily_note_id, content, node_type, position, \
         node_type, status, priority, due_date, created_at, updated_at \
         FROM nodes WHERE parent_id = ?1 ORDER BY position ASC"
    )?;
    let rows = stmt.query_map(params![parent_id], |r| row_to_node(r))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_roots_for_day(db: &Database, daily_note_id: &str) -> Result<Vec<Node>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, parent_id, daily_note_id, content, node_type, position, \
         node_type, status, priority, due_date, created_at, updated_at \
         FROM nodes WHERE daily_note_id = ?1 AND parent_id IS NULL ORDER BY position ASC"
    )?;
    let rows = stmt.query_map(params![daily_note_id], |r| row_to_node(r))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_all_tasks(db: &Database, status_filter: Option<&TaskStatus>) -> Result<Vec<Node>> {
    let sql = if let Some(s) = status_filter {
        format!(
            "SELECT id, parent_id, daily_note_id, content, node_type, position, \
             node_type, status, priority, due_date, created_at, updated_at \
             FROM nodes WHERE node_type = 'task' AND status = '{}' ORDER BY created_at DESC",
            s.as_str()
        )
    } else {
        "SELECT id, parent_id, daily_note_id, content, node_type, position, \
         node_type, status, priority, due_date, created_at, updated_at \
         FROM nodes WHERE node_type = 'task' ORDER BY created_at DESC".to_string()
    };
    let mut stmt = db.conn.prepare(&sql)?;
    let rows = stmt.query_map([], |r| row_to_node(r))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn update(db: &Database, id: &str, req: UpdateNode) -> Result<()> {
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
    Ok(())
}

pub fn delete(db: &Database, id: &str) -> Result<()> {
    db.conn.execute("DELETE FROM nodes WHERE id = ?1", params![id])?;
    Ok(())
}

/// Recursively build a node tree from a list of root nodes
pub fn build_tree(db: &Database, roots: Vec<Node>) -> Result<Vec<Node>> {
    let mut result = vec![];
    for mut node in roots {
        let children = get_children(db, &node.id)?;
        node.children = build_tree(db, children)?;
        result.push(node);
    }
    Ok(result)
}
```

**Step 4: Run tests**

Run: `cargo test -p sup-core nodes`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/sup-core/src/queries/nodes.rs
git commit -m "feat: node CRUD queries with task filtering and tree building"
```

---

### Task 6: Tag Queries + Hierarchical Lookup

**Files:**
- Create: `crates/sup-core/src/queries/tags.rs`

**Step 1: Write failing tests**

```rust
// crates/sup-core/src/queries/tags.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn test_db() -> Database { Database::open_in_memory().unwrap() }

    #[test]
    fn test_get_or_create_tag_hierarchy() {
        let db = test_db();
        let tag = get_or_create(&db, "projects/foo/bar").unwrap();
        assert_eq!(tag.name, "projects/foo/bar");

        // parent tags created
        let parent = get_by_name(&db, "projects/foo").unwrap();
        assert!(parent.is_some());
        let grandparent = get_by_name(&db, "projects").unwrap();
        assert!(grandparent.is_some());
    }

    #[test]
    fn test_get_all_with_prefix() {
        let db = test_db();
        get_or_create(&db, "projects/foo").unwrap();
        get_or_create(&db, "projects/bar").unwrap();
        get_or_create(&db, "work").unwrap();

        let results = get_all_with_prefix(&db, "projects").unwrap();
        assert_eq!(results.len(), 3); // projects, projects/foo, projects/bar
    }

    #[test]
    fn test_tag_node() {
        let db = test_db();
        use crate::queries::{daily_notes, nodes};
        use crate::models::NodeType;
        use chrono::NaiveDate;

        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = daily_notes::get_or_create(&db, date).unwrap();
        let node = nodes::create(&db, nodes::CreateNode {
            parent_id: None, daily_note_id: Some(note.id),
            content: "test".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();

        let tag = get_or_create(&db, "projects/foo").unwrap();
        add_tag_to_node(&db, &node.id, &tag.id).unwrap();

        let tags = get_tags_for_node(&db, &node.id).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "projects/foo");
    }
}
```

**Step 2: Run to verify failure**

Run: `cargo test -p sup-core tags`
Expected: FAIL

**Step 3: Implement**

```rust
// crates/sup-core/src/queries/tags.rs
use anyhow::Result;
use rusqlite::params;
use uuid::Uuid;
use crate::db::Database;
use crate::models::Tag;

pub fn get_or_create(db: &Database, full_path: &str) -> Result<Tag> {
    // ensure all ancestors exist
    let parts: Vec<&str> = full_path.split('/').collect();
    let mut parent_id: Option<String> = None;
    let mut last_tag: Option<Tag> = None;

    for i in 0..parts.len() {
        let path = parts[..=i].join("/");
        let tag = get_by_name(db, &path)?;
        let tag = match tag {
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

pub fn get_by_name(db: &Database, name: &str) -> Result<Option<Tag>> {
    let mut stmt = db.conn.prepare("SELECT id, name, parent_id FROM tags WHERE name = ?1")?;
    let mut rows = stmt.query(params![name])?;
    Ok(rows.next()?.map(|r| Tag {
        id: r.get(0).unwrap(),
        name: r.get(1).unwrap(),
        parent_id: r.get(2).unwrap(),
    }))
}

/// Returns the tag with the given prefix AND all its descendants
pub fn get_all_with_prefix(db: &Database, prefix: &str) -> Result<Vec<Tag>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, name, parent_id FROM tags WHERE name = ?1 OR name LIKE ?2"
    )?;
    let like = format!("{}/%", prefix);
    let rows = stmt.query_map(params![prefix, like], |r| {
        Ok(Tag { id: r.get(0)?, name: r.get(1)?, parent_id: r.get(2)? })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn add_tag_to_node(db: &Database, node_id: &str, tag_id: &str) -> Result<()> {
    db.conn.execute(
        "INSERT OR IGNORE INTO node_tags (node_id, tag_id) VALUES (?1, ?2)",
        params![node_id, tag_id],
    )?;
    Ok(())
}

pub fn remove_tag_from_node(db: &Database, node_id: &str, tag_id: &str) -> Result<()> {
    db.conn.execute(
        "DELETE FROM node_tags WHERE node_id = ?1 AND tag_id = ?2",
        params![node_id, tag_id],
    )?;
    Ok(())
}

pub fn get_tags_for_node(db: &Database, node_id: &str) -> Result<Vec<Tag>> {
    let mut stmt = db.conn.prepare(
        "SELECT t.id, t.name, t.parent_id FROM tags t \
         JOIN node_tags nt ON t.id = nt.tag_id WHERE nt.node_id = ?1"
    )?;
    let rows = stmt.query_map(params![node_id], |r| {
        Ok(Tag { id: r.get(0)?, name: r.get(1)?, parent_id: r.get(2)? })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_nodes_for_tag_prefix(db: &Database, prefix: &str) -> Result<Vec<String>> {
    let tags = get_all_with_prefix(db, prefix)?;
    let mut node_ids: Vec<String> = vec![];
    for tag in tags {
        let mut stmt = db.conn.prepare(
            "SELECT node_id FROM node_tags WHERE tag_id = ?1"
        )?;
        let rows = stmt.query_map(params![tag.id], |r| r.get::<_, String>(0))?;
        for id in rows { node_ids.push(id?); }
    }
    node_ids.dedup();
    Ok(node_ids)
}
```

**Step 4: Run tests**

Run: `cargo test -p sup-core tags`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/sup-core/src/queries/tags.rs
git commit -m "feat: hierarchical tag queries with node tagging support"
```

---

### Task 7: Search + Carryover

**Files:**
- Create: `crates/sup-core/src/queries/search.rs`
- Create: `crates/sup-core/src/queries/carryover.rs`
- Modify: `crates/sup-core/src/queries/mod.rs`

**Step 1: Write failing tests**

```rust
// crates/sup-core/src/queries/search.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::queries::{daily_notes, nodes};
    use crate::models::NodeType;
    use chrono::NaiveDate;

    fn test_db() -> Database { Database::open_in_memory().unwrap() }

    #[test]
    fn test_full_text_search() {
        let db = test_db();
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let note = daily_notes::get_or_create(&db, date).unwrap();
        nodes::create(&db, nodes::CreateNode {
            parent_id: None, daily_note_id: Some(note.id.clone()),
            content: "implement oauth flow".into(), node_type: NodeType::Bullet,
            position: 0, status: None, priority: None, due_date: None,
        }).unwrap();
        nodes::create(&db, nodes::CreateNode {
            parent_id: None, daily_note_id: Some(note.id),
            content: "review pull request".into(), node_type: NodeType::Bullet,
            position: 1, status: None, priority: None, due_date: None,
        }).unwrap();

        let results = search_nodes(&db, "oauth").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "implement oauth flow");
    }
}
```

**Step 2: Run to verify failure**

Run: `cargo test -p sup-core search`
Expected: FAIL

**Step 3: Implement search**

```rust
// crates/sup-core/src/queries/search.rs
use anyhow::Result;
use rusqlite::params;
use crate::db::Database;
use crate::models::Node;
use super::nodes::{get_by_id, build_tree, get_children};

pub fn search_nodes(db: &Database, query: &str) -> Result<Vec<Node>> {
    let mut stmt = db.conn.prepare(
        "SELECT n.id FROM nodes n \
         JOIN nodes_fts f ON n.rowid = f.rowid \
         WHERE nodes_fts MATCH ?1 ORDER BY rank"
    )?;
    let ids: Vec<String> = stmt.query_map(params![query], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut nodes = vec![];
    for id in ids {
        if let Some(mut node) = get_by_id(db, &id)? {
            let children = get_children(db, &node.id)?;
            node.children = build_tree(db, children)?;
            nodes.push(node);
        }
    }
    Ok(nodes)
}
```

```rust
// crates/sup-core/src/queries/carryover.rs
use anyhow::Result;
use chrono::{NaiveDate, Utc};
use rusqlite::params;
use uuid::Uuid;
use crate::db::Database;
use crate::models::TaskStatus;
use super::nodes::get_all_tasks;

/// Link incomplete tasks from previous days into today's note
/// Creates node_links entries; does not move task nodes
pub fn carry_over_tasks(db: &Database, today_note_id: &str, today: NaiveDate) -> Result<usize> {
    let all_tasks = get_all_tasks(db, None)?;
    let mut count = 0;
    let now = Utc::now().to_rfc3339();

    for task in all_tasks {
        // skip tasks already from today
        if let Some(ref note_id) = task.daily_note_id {
            if note_id == today_note_id { continue; }
        }
        // only carry todo/in_progress
        let carry = matches!(task.status, Some(TaskStatus::Todo) | Some(TaskStatus::InProgress));
        if !carry { continue; }

        // check if already linked to today's note
        let existing: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM node_links WHERE source_id = ?1 AND target_id = ?1",
            params![task.id],
            |r| r.get(0),
        ).unwrap_or(0);
        if existing > 0 { continue; }

        // create a link from task to today's note sentinel
        let id = Uuid::new_v4().to_string();
        db.conn.execute(
            "INSERT OR IGNORE INTO node_links (id, source_id, target_id, created_at) \
             VALUES (?1, ?2, ?3, ?4)",
            params![id, task.id, today_note_id, now],
        )?;
        count += 1;
    }
    Ok(count)
}
```

**Step 4: Run tests**

Run: `cargo test -p sup-core`
Expected: all PASS

**Step 5: Commit**

```bash
git add crates/sup-core/src/queries/search.rs crates/sup-core/src/queries/carryover.rs
git commit -m "feat: fts5 search and task carryover logic"
```

---

## Phase 3: CLI

### Task 8: CLI Scaffold

**Files:**
- Create: `crates/sup/src/cli/mod.rs`
- Create: `crates/sup/src/cli/args.rs`
- Create: `crates/sup/src/config.rs`
- Modify: `crates/sup/src/main.rs`

**Step 1: Implement the CLI arg structure**

No test here — this is pure wiring. Verify by running the binary.

```rust
// crates/sup/src/cli/args.rs
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "sup", about = "Terminal knowledge base and engineering journal")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output as JSON (machine-readable)
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Append a node to today's journal
    Log {
        content: String,
        #[arg(long, default_value = "bullet")]
        r#type: String,
        #[arg(long)]
        tag: Option<String>,
    },
    /// Show today's journal
    Today,
    /// Show a specific day's journal
    Day { date: String },
    /// Task subcommands
    Task {
        #[command(subcommand)]
        cmd: TaskCommand,
    },
    /// List tasks
    Tasks {
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    /// Search nodes
    Search { query: String },
    /// Launch the TUI
    Tui,
}

#[derive(Subcommand)]
pub enum TaskCommand {
    Add {
        title: String,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        due: Option<String>,
        #[arg(long)]
        tag: Option<String>,
    },
    Done { id: String },
    Edit {
        id: String,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        due: Option<String>,
    },
}
```

```rust
// crates/sup/src/config.rs
use std::path::PathBuf;

pub fn db_path() -> PathBuf {
    let home = dirs::home_dir().expect("no home dir");
    let dir = home.join(".sup");
    std::fs::create_dir_all(&dir).ok();
    dir.join("sup.db")
}
```

Add `dirs = "5"` to `crates/sup/Cargo.toml` dependencies.

```rust
// crates/sup/src/main.rs
mod cli;
mod config;
mod commands;

use clap::Parser;
use cli::args::Cli;
use anyhow::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_path = config::db_path();
    let db = sup_core::db::Database::open(db_path.to_str().unwrap())?;
    commands::dispatch(db, cli)
}
```

```rust
// crates/sup/src/commands/mod.rs
use anyhow::Result;
use crate::cli::args::{Cli, Command};
use sup_core::db::Database;

pub mod log;
pub mod today;
pub mod tasks;
pub mod search;

pub fn dispatch(db: Database, cli: Cli) -> Result<()> {
    match cli.command {
        Command::Log { content, r#type, tag } => log::run(&db, content, r#type, tag, cli.json),
        Command::Today => today::run(&db, chrono::Local::now().date_naive(), cli.json),
        Command::Day { date } => {
            let d = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")?;
            today::run(&db, d, cli.json)
        },
        Command::Tasks { tag, status } => tasks::list(&db, tag, status, cli.json),
        Command::Task { cmd } => tasks::dispatch_task(&db, cmd, cli.json),
        Command::Search { query } => search::run(&db, query, cli.json),
        Command::Tui => crate::tui::run(db),
    }
}
```

**Step 2: Verify it compiles and `--help` works**

Run: `cargo build -p sup && ./target/debug/sup --help`
Expected: prints help with all subcommands

**Step 3: Commit**

```bash
git add crates/sup/src/
git commit -m "feat: CLI scaffold with clap - all subcommands wired"
```

---

### Task 9: CLI Commands Implementation

**Files:**
- Create: `crates/sup/src/commands/log.rs`
- Create: `crates/sup/src/commands/today.rs`
- Create: `crates/sup/src/commands/tasks.rs`
- Create: `crates/sup/src/commands/search.rs`
- Create: `crates/sup/src/output.rs`

**Step 1: Implement output formatting helper**

```rust
// crates/sup/src/output.rs
use sup_core::models::Node;

pub fn print_node_tree(nodes: &[Node], indent: usize, json: bool) {
    if json { return; } // caller handles json
    for node in nodes {
        let prefix = "  ".repeat(indent);
        let icon = if node.node_type == sup_core::models::NodeType::Task {
            node.status.as_ref().map(|s| s.icon()).unwrap_or("☐")
        } else {
            node.node_type.icon()
        };
        let tag_str = if node.tags.is_empty() {
            String::new()
        } else {
            format!("  \x1b[2m#{}\x1b[0m", node.tags.join(" #"))
        };
        println!("{}{} {}{}", prefix, icon, node.content, tag_str);
        print_node_tree(&node.children, indent + 1, false);
    }
}
```

**Step 2: Implement `sup log`**

```rust
// crates/sup/src/commands/log.rs
use anyhow::Result;
use chrono::Local;
use sup_core::db::Database;
use sup_core::models::NodeType;
use sup_core::queries::{daily_notes, nodes, tags};

pub fn run(db: &Database, content: String, type_str: String, tag: Option<String>, json: bool) -> Result<()> {
    let today = Local::now().date_naive();
    let note = daily_notes::get_or_create(db, today)?;

    // find next position
    let roots = nodes::get_roots_for_day(db, &note.id)?;
    let position = roots.len() as i64;

    let node_type = NodeType::from_str(&type_str).unwrap_or(NodeType::Bullet);
    let mut node = nodes::create(db, nodes::CreateNode {
        parent_id: None,
        daily_note_id: Some(note.id),
        content,
        node_type,
        position,
        status: None, priority: None, due_date: None,
    })?;

    if let Some(tag_str) = tag {
        let tag = tags::get_or_create(db, &tag_str)?;
        tags::add_tag_to_node(db, &node.id, &tag.id)?;
        node.tags = vec![tag.name];
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&node)?);
    } else {
        println!("logged: {} {}", node.node_type.icon(), node.content);
    }
    Ok(())
}
```

**Step 3: Implement `sup today` / `sup day`**

```rust
// crates/sup/src/commands/today.rs
use anyhow::Result;
use chrono::NaiveDate;
use sup_core::db::Database;
use sup_core::queries::{daily_notes, nodes, carryover};
use crate::output::print_node_tree;

pub fn run(db: &Database, date: NaiveDate, json: bool) -> Result<()> {
    let note = daily_notes::get_or_create(db, date)?;

    // carryover on today only
    if date == chrono::Local::now().date_naive() {
        carryover::carry_over_tasks(db, &note.id, date)?;
    }

    let roots = nodes::get_roots_for_day(db, &note.id)?;
    let tree = nodes::build_tree(db, roots)?;

    if json {
        let out = serde_json::json!({ "date": date.to_string(), "nodes": tree });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("# {}", date.format("%A, %B %d %Y"));
        print_node_tree(&tree, 0, false);
    }
    Ok(())
}
```

**Step 4: Implement `sup tasks` + `sup task` subcommands**

```rust
// crates/sup/src/commands/tasks.rs
use anyhow::Result;
use sup_core::db::Database;
use sup_core::models::{NodeType, Priority, TaskStatus};
use sup_core::queries::{nodes, tags, daily_notes};
use crate::cli::args::TaskCommand;
use crate::output::print_node_tree;
use chrono::{Local, NaiveDate};

pub fn list(db: &Database, tag: Option<String>, status: Option<String>, json: bool) -> Result<()> {
    let status_filter = status.as_deref().and_then(|s| TaskStatus::from_str(s).ok());
    let mut tasks = nodes::get_all_tasks(db, status_filter.as_ref())?;

    if let Some(tag_str) = &tag {
        let node_ids = tags::get_nodes_for_tag_prefix(db, tag_str)?;
        tasks.retain(|t| node_ids.contains(&t.id));
    }

    // attach children as context
    let mut tasks_with_ctx = vec![];
    for mut task in tasks {
        let children = nodes::get_children(db, &task.id)?;
        task.children = nodes::build_tree(db, children)?;
        tasks_with_ctx.push(task);
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&tasks_with_ctx)?);
    } else {
        print_node_tree(&tasks_with_ctx, 0, false);
    }
    Ok(())
}

pub fn dispatch_task(db: &Database, cmd: TaskCommand, json: bool) -> Result<()> {
    match cmd {
        TaskCommand::Add { title, priority, due, tag } => {
            let today = Local::now().date_naive();
            let note = daily_notes::get_or_create(db, today)?;
            let roots = nodes::get_roots_for_day(db, &note.id)?;
            let position = roots.len() as i64;

            let due_date = due.as_deref()
                .map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d"))
                .transpose()?;

            let mut node = nodes::create(db, nodes::CreateNode {
                parent_id: None,
                daily_note_id: Some(note.id),
                content: title,
                node_type: NodeType::Task,
                position,
                status: Some(TaskStatus::Todo),
                priority: priority.as_deref().and_then(|p| Priority::from_str(p).ok()),
                due_date,
            })?;

            if let Some(tag_str) = tag {
                let t = tags::get_or_create(db, &tag_str)?;
                tags::add_tag_to_node(db, &node.id, &t.id)?;
                node.tags = vec![t.name];
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&node)?);
            } else {
                println!("task created [{}]: {}", node.id, node.content);
            }
        }
        TaskCommand::Done { id } => {
            nodes::update(db, &id, nodes::UpdateNode {
                content: None,
                status: Some(TaskStatus::Done),
                priority: None, due_date: None, position: None, parent_id: None,
            })?;
            println!("done ✓");
        }
        TaskCommand::Edit { id, status, priority, due } => {
            nodes::update(db, &id, nodes::UpdateNode {
                content: None,
                status: status.as_deref().and_then(|s| TaskStatus::from_str(s).ok()),
                priority: priority.as_deref().and_then(|p| Priority::from_str(p).ok()),
                due_date: due.as_deref()
                    .map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                    .unwrap_or(None)
                    .map(Some).unwrap_or(None),
                position: None, parent_id: None,
            })?;
            println!("updated");
        }
    }
    Ok(())
}
```

**Step 5: Implement `sup search`**

```rust
// crates/sup/src/commands/search.rs
use anyhow::Result;
use sup_core::db::Database;
use sup_core::queries::search;
use crate::output::print_node_tree;

pub fn run(db: &Database, query: String, json: bool) -> Result<()> {
    let results = search::search_nodes(db, &query)?;
    if json {
        let out = serde_json::json!({ "query": query, "results": results });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        if results.is_empty() {
            println!("no results for \"{}\"", query);
        } else {
            println!("{} result(s):", results.len());
            print_node_tree(&results, 0, false);
        }
    }
    Ok(())
}
```

**Step 6: Smoke-test the CLI end-to-end**

```bash
cargo build -p sup
./target/debug/sup log "testing the cli"
./target/debug/sup log --type task "first task"
./target/debug/sup task add "second task" --priority high
./target/debug/sup today
./target/debug/sup tasks
./target/debug/sup search "testing"
./target/debug/sup tasks --json
```

Expected: all commands produce output without errors.

**Step 7: Commit**

```bash
git add crates/sup/src/
git commit -m "feat: all CLI commands - log, today, day, tasks, task add/done/edit, search with --json"
```

---

## Phase 4: TUI

### Task 10: TUI App Shell + TEA Scaffold

**Files:**
- Create: `crates/sup/src/tui/mod.rs`
- Create: `crates/sup/src/tui/app.rs`
- Create: `crates/sup/src/tui/events.rs`
- Modify: `crates/sup/src/main.rs`

**Step 1: Implement the TEA shell**

```rust
// crates/sup/src/tui/events.rs
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub fn next_event() -> anyhow::Result<AppEvent> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(k) = event::read()? {
            return Ok(AppEvent::Key(k));
        }
    }
    Ok(AppEvent::Tick)
}
```

```rust
// crates/sup/src/tui/app.rs
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::Frame;
use sup_core::db::Database;
use crate::tui::events::AppEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Journal,
    Tasks,
    Split,
}

pub struct App {
    pub db: Database,
    pub view: View,
    pub should_quit: bool,
}

pub enum Message {
    Quit,
    SwitchView(View),
    Noop,
}

impl App {
    pub fn new(db: Database) -> Self {
        Self { db, view: View::Journal, should_quit: false }
    }

    pub fn handle_event(&self, event: AppEvent) -> Message {
        match event {
            AppEvent::Key(k) => match (k.code, k.modifiers) {
                (KeyCode::Char('q'), _) => Message::Quit,
                (KeyCode::Char('1'), _) => Message::SwitchView(View::Journal),
                (KeyCode::Char('2'), _) => Message::SwitchView(View::Tasks),
                (KeyCode::Char('3'), _) => Message::SwitchView(View::Split),
                _ => Message::Noop,
            },
            AppEvent::Tick => Message::Noop,
        }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::Quit => self.should_quit = true,
            Message::SwitchView(v) => self.view = v,
            Message::Noop => {}
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        use ratatui::widgets::{Block, Paragraph};
        use ratatui::layout::Alignment;
        let text = match self.view {
            View::Journal => "Journal View (j/k to navigate, 2 for tasks, 3 for split, q to quit)",
            View::Tasks => "Task View (1 for journal, 3 for split, q to quit)",
            View::Split => "Split View (1 for journal, 2 for tasks, q to quit)",
        };
        frame.render_widget(
            Paragraph::new(text).alignment(Alignment::Center).block(Block::bordered().title("sup")),
            frame.area(),
        );
    }
}
```

```rust
// crates/sup/src/tui/mod.rs
mod app;
mod events;
pub mod journal;
pub mod tasks_view;

use anyhow::Result;
use crossterm::{execute, terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use sup_core::db::Database;

pub fn run(db: Database) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::new(db);
    loop {
        terminal.draw(|f| app.render(f))?;
        let event = events::next_event()?;
        let msg = app.handle_event(event);
        app.update(msg);
        if app.should_quit { break; }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
```

**Step 2: Wire into main**

Add `mod tui;` to `main.rs` and update `commands::dispatch` to call `tui::run(db)` for `Command::Tui`.

**Step 3: Smoke-test**

Run: `cargo run -p sup -- tui`
Expected: terminal clears, shows placeholder text, `q` quits cleanly.

**Step 4: Commit**

```bash
git add crates/sup/src/tui/
git commit -m "feat: TUI shell with TEA loop, view switching, clean exit"
```

---

### Task 11: Journal View — Render + Navigate

**Files:**
- Create: `crates/sup/src/tui/journal.rs`
- Modify: `crates/sup/src/tui/app.rs`

**Step 1: Implement journal state and rendering**

```rust
// crates/sup/src/tui/journal.rs
use chrono::{Local, NaiveDate};
use ratatui::{Frame, layout::Rect, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, List, ListItem, ListState}};
use sup_core::{db::Database, models::{Node, NodeType, TaskStatus}, queries::{daily_notes, nodes}};
use anyhow::Result;

pub struct JournalState {
    pub date: NaiveDate,
    pub nodes: Vec<FlatNode>,   // flattened tree for rendering
    pub list_state: ListState,
}

pub struct FlatNode {
    pub node: Node,
    pub depth: usize,
    pub collapsed: bool,
}

impl JournalState {
    pub fn new(db: &Database) -> Result<Self> {
        let date = Local::now().date_naive();
        let mut state = Self { date, nodes: vec![], list_state: ListState::default() };
        state.reload(db)?;
        Ok(state)
    }

    pub fn reload(&mut self, db: &Database) -> Result<()> {
        let note = daily_notes::get_or_create(db, self.date)?;
        let roots = nodes::get_roots_for_day(db, &note.id)?;
        let tree = nodes::build_tree(db, roots)?;
        self.nodes = flatten_tree(&tree, 0);
        if self.list_state.selected().is_none() && !self.nodes.is_empty() {
            self.list_state.select(Some(0));
        }
        Ok(())
    }

    pub fn prev_day(&mut self, db: &Database) -> Result<()> {
        self.date = self.date.pred_opt().unwrap_or(self.date);
        self.list_state.select(None);
        self.reload(db)
    }

    pub fn next_day(&mut self, db: &Database) -> Result<()> {
        let today = Local::now().date_naive();
        if self.date < today {
            self.date = self.date.succ_opt().unwrap_or(self.date);
            self.list_state.select(None);
            self.reload(db)?;
        }
        Ok(())
    }

    pub fn move_up(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i > 0 { self.list_state.select(Some(i - 1)); }
    }

    pub fn move_down(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i + 1 < self.nodes.len() { self.list_state.select(Some(i + 1)); }
    }

    pub fn selected_node(&self) -> Option<&FlatNode> {
        self.list_state.selected().and_then(|i| self.nodes.get(i))
    }
}

fn flatten_tree(nodes: &[Node], depth: usize) -> Vec<FlatNode> {
    let mut result = vec![];
    for node in nodes {
        result.push(FlatNode { node: node.clone(), depth, collapsed: false });
        result.extend(flatten_tree(&node.children, depth + 1));
    }
    result
}

pub fn render(state: &mut JournalState, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = state.nodes.iter().map(|flat| {
        let indent = "  ".repeat(flat.depth);
        let icon = match &flat.node.node_type {
            NodeType::Task => flat.node.status.as_ref()
                .map(|s| s.icon()).unwrap_or("☐"),
            t => t.icon(),
        };
        let content_style = match flat.node.node_type {
            NodeType::H1 => Style::default().add_modifier(Modifier::BOLD),
            NodeType::H2 => Style::default().add_modifier(Modifier::BOLD),
            NodeType::Quote => Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            NodeType::Code => Style::default().fg(Color::Cyan),
            _ => Style::default(),
        };
        let tag_str = if flat.node.tags.is_empty() {
            String::new()
        } else {
            format!("  #{}", flat.node.tags.join(" #"))
        };
        ListItem::new(Line::from(vec![
            Span::raw(format!("{}{} ", indent, icon)),
            Span::styled(flat.node.content.clone(), content_style),
            Span::styled(tag_str, Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();

    let title = format!("  {}  ", state.date.format("%A, %B %d %Y"));
    let list = List::new(items)
        .block(Block::bordered().title(title))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut state.list_state);
}
```

**Step 2: Integrate into App**

Update `app.rs`:
- Add `journal: Option<JournalState>` to `App`
- Initialize it in `App::new()`
- Route `j`/`k`/`[`/`]` keys to journal state methods
- Call `journal::render()` in `app.render()` when view is `Journal`

**Step 3: Smoke-test**

Run: `cargo run -p sup -- tui`
Expected: journal renders with today's nodes, `j`/`k` moves cursor, `[` goes to yesterday.

**Step 4: Commit**

```bash
git add crates/sup/src/tui/journal.rs crates/sup/src/tui/app.rs
git commit -m "feat: journal view with outline rendering, navigation, day switching"
```

---

### Task 12: Journal View — Node Editing (Add/Edit/Delete/Reorder)

**Files:**
- Modify: `crates/sup/src/tui/journal.rs`
- Modify: `crates/sup/src/tui/app.rs`
- Create: `crates/sup/src/tui/editor.rs`

**Step 1: Implement inline editor widget**

```rust
// crates/sup/src/tui/editor.rs
/// Single-line inline text editor with vim-style insert mode
pub struct InlineEditor {
    pub content: String,
    pub cursor: usize,
    pub active: bool,
}

impl InlineEditor {
    pub fn new(initial: &str) -> Self {
        Self { content: initial.to_string(), cursor: initial.len(), active: true }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> EditorResult {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Char(c) => {
                self.content.insert(self.cursor, c);
                self.cursor += 1;
                EditorResult::Continue
            }
            KeyCode::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.content.remove(self.cursor);
                EditorResult::Continue
            }
            KeyCode::Left if self.cursor > 0 => { self.cursor -= 1; EditorResult::Continue }
            KeyCode::Right if self.cursor < self.content.len() => { self.cursor += 1; EditorResult::Continue }
            KeyCode::Enter => EditorResult::Commit(self.content.clone()),
            KeyCode::Esc => EditorResult::Cancel,
            _ => EditorResult::Continue,
        }
    }
}

pub enum EditorResult {
    Continue,
    Commit(String),
    Cancel,
}
```

**Step 2: Add editing operations to JournalState**

Add to `journal.rs`:

```rust
pub fn add_node_below(&mut self, db: &Database) -> Result<()> {
    // creates a new empty node after cursor, enters edit mode
    // implementation uses nodes::create with next position
    todo!("implemented in step 3")
}

pub fn delete_selected(&mut self, db: &Database) -> Result<()> {
    if let Some(flat) = self.selected_node() {
        let id = flat.node.id.clone();
        nodes::delete(db, &id)?;
        self.reload(db)?;
    }
    Ok(())
}

pub fn indent_selected(&mut self, db: &Database) -> Result<()> {
    // makes selected node a child of the node above it
    todo!()
}

pub fn unindent_selected(&mut self, db: &Database) -> Result<()> {
    // promotes selected node to sibling of its parent
    todo!()
}
```

**Step 3: Implement the full editing flow**

Implement all `todo!()` stubs using `nodes::update()` and `nodes::create()`. The key insight: `indent` sets `parent_id` to the sibling above's id; `unindent` sets `parent_id` to the current parent's `parent_id`.

**Step 4: Wire keys in app.rs**

```
o → add_node_below + enter edit mode
O → add_node_above + enter edit mode
<enter> → edit selected node (load content into InlineEditor)
dd → delete_selected
<tab> → indent_selected
<S-tab> → unindent_selected
J → swap position with next sibling
K → swap position with prev sibling
```

**Step 5: Smoke-test**

```bash
cargo run -p sup -- tui
# press o, type "new node", enter → node appears
# press dd → node deleted
# press <tab> → node indented
```

**Step 6: Commit**

```bash
git add crates/sup/src/tui/
git commit -m "feat: journal node add/edit/delete/indent/reorder operations"
```

---

### Task 13: Task View + Split View

**Files:**
- Create: `crates/sup/src/tui/tasks_view.rs`
- Modify: `crates/sup/src/tui/app.rs`

**Step 1: Implement task view state and rendering**

```rust
// crates/sup/src/tui/tasks_view.rs
use ratatui::{Frame, layout::Rect, style::{Color, Style}, widgets::{Block, List, ListItem, ListState}};
use sup_core::{db::Database, models::{Node, TaskStatus}, queries::nodes};
use anyhow::Result;

pub struct TasksState {
    pub tasks: Vec<Node>,
    pub list_state: ListState,
}

impl TasksState {
    pub fn new(db: &Database) -> Result<Self> {
        let mut s = Self { tasks: vec![], list_state: ListState::default() };
        s.reload(db)?;
        Ok(s)
    }

    pub fn reload(&mut self, db: &Database) -> Result<()> {
        let mut tasks = nodes::get_all_tasks(db, None)?;
        for task in &mut tasks {
            let children = nodes::get_children(db, &task.id)?;
            task.children = nodes::build_tree(db, children)?;
        }
        self.tasks = tasks;
        Ok(())
    }

    pub fn move_up(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i > 0 { self.list_state.select(Some(i - 1)); }
    }

    pub fn move_down(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i + 1 < self.tasks.len() { self.list_state.select(Some(i + 1)); }
    }

    pub fn cycle_status(&mut self, db: &Database) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            if let Some(task) = self.tasks.get(i) {
                let new_status = task.status.as_ref()
                    .map(|s| s.next())
                    .unwrap_or(TaskStatus::Todo);
                nodes::update(db, &task.id, nodes::UpdateNode {
                    status: Some(new_status),
                    content: None, priority: None, due_date: None, position: None, parent_id: None,
                })?;
                self.reload(db)?;
            }
        }
        Ok(())
    }
}

pub fn render(state: &mut TasksState, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = state.tasks.iter().map(|task| {
        let icon = task.status.as_ref().map(|s| s.icon()).unwrap_or("☐");
        let priority_badge = match task.priority {
            Some(ref p) => match p {
                sup_core::models::Priority::High => " [H]",
                sup_core::models::Priority::Med => " [M]",
                sup_core::models::Priority::Low => " [L]",
            },
            None => "",
        };
        ListItem::new(format!("{} {}{}", icon, task.content, priority_badge))
    }).collect();

    let list = List::new(items)
        .block(Block::bordered().title("Tasks"))
        .highlight_style(Style::default().bg(Color::DarkGray));
    frame.render_stateful_widget(list, area, &mut state.list_state);
}
```

**Step 2: Add split layout to app.render()**

```rust
// In app.rs render():
use ratatui::layout::{Layout, Direction, Constraint};

View::Split => {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(frame.area());
    journal::render(self.journal.as_mut().unwrap(), frame, chunks[0]);
    tasks_view::render(self.tasks.as_mut().unwrap(), frame, chunks[1]);
}
```

**Step 3: Wire `Tab` to toggle active pane in split view**

Add `active_pane: Pane` to `App` (Journal or Tasks), route `j`/`k`/`c` to the active pane.

**Step 4: Smoke-test**

```bash
cargo run -p sup -- tui
# press 2 → tasks view
# press j/k to navigate
# press c → cycles task status
# press 3 → split view
```

**Step 5: Commit**

```bash
git add crates/sup/src/tui/tasks_view.rs crates/sup/src/tui/app.rs
git commit -m "feat: task view with status cycling, split view layout"
```

---

### Task 14: Search Overlay + Linking

**Files:**
- Create: `crates/sup/src/tui/search_overlay.rs`
- Modify: `crates/sup/src/tui/app.rs`

**Step 1: Implement search overlay**

```rust
// crates/sup/src/tui/search_overlay.rs
use ratatui::{Frame, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style}, widgets::{Block, List, ListItem, ListState, Paragraph}};
use sup_core::{db::Database, models::Node, queries::search};

pub struct SearchOverlay {
    pub query: String,
    pub results: Vec<Node>,
    pub list_state: ListState,
    pub active: bool,
}

impl SearchOverlay {
    pub fn new() -> Self {
        Self { query: String::new(), results: vec![], list_state: ListState::default(), active: false }
    }

    pub fn open(&mut self) { self.active = true; self.query.clear(); self.results.clear(); }
    pub fn close(&mut self) { self.active = false; }

    pub fn handle_char(&mut self, c: char, db: &Database) {
        self.query.push(c);
        self.results = search::search_nodes(db, &self.query).unwrap_or_default();
        if !self.results.is_empty() { self.list_state.select(Some(0)); }
    }

    pub fn handle_backspace(&mut self, db: &Database) {
        self.query.pop();
        self.results = if self.query.is_empty() { vec![] }
            else { search::search_nodes(db, &self.query).unwrap_or_default() };
    }
}

pub fn render(state: &mut SearchOverlay, frame: &mut Frame) {
    let area = centered_rect(80, 60, frame.area());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let input = Paragraph::new(state.query.as_str())
        .block(Block::bordered().title("Search (Esc to close)"));
    frame.render_widget(input, chunks[0]);

    let items: Vec<ListItem> = state.results.iter()
        .map(|n| ListItem::new(n.content.clone()))
        .collect();
    let list = List::new(items)
        .block(Block::bordered())
        .highlight_style(Style::default().bg(Color::DarkGray));
    frame.render_stateful_widget(list, chunks[1], &mut state.list_state);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

**Step 2: Wire `/` key to open search overlay in app.rs**

When `search.active` is true, route all key input to the overlay first. `Esc` closes it.

**Step 3: Implement `L` — bidirectional linking**

When `L` is pressed on a selected node, open a search overlay to pick a target. On selection, insert into `node_links`:

```rust
db.conn.execute(
    "INSERT OR IGNORE INTO node_links (id, source_id, target_id, created_at) VALUES (?1,?2,?3,?4)",
    params![Uuid::new_v4().to_string(), source_id, target_id, Utc::now().to_rfc3339()],
)?;
```

**Step 4: Smoke-test**

```bash
cargo run -p sup -- tui
# press / → search overlay opens
# type "test" → results appear
# Esc → closes overlay
# select a node, press L → link picker opens
```

**Step 5: Commit**

```bash
git add crates/sup/src/tui/search_overlay.rs crates/sup/src/tui/app.rs
git commit -m "feat: search overlay with live results, bidirectional node linking"
```

---

### Task 15: Tag Editing in TUI

**Files:**
- Create: `crates/sup/src/tui/tag_editor.rs`
- Modify: `crates/sup/src/tui/app.rs`

**Step 1: Implement tag editor popup**

```rust
// crates/sup/src/tui/tag_editor.rs
// Small popup: shows current tags, allows adding/removing
// Input: type tag name + Enter to add, select existing + dd to remove
pub struct TagEditor {
    pub node_id: String,
    pub current_tags: Vec<sup_core::models::Tag>,
    pub input: String,
    pub active: bool,
}
// Render: bordered popup with current tags listed, input field at bottom
// On Enter: call tags::get_or_create() + tags::add_tag_to_node()
// On Esc: close
```

Full implementation follows the same pattern as `SearchOverlay` — a popup with list + input. Wire `t` key in app.rs to open it with the selected node's id and current tags.

**Step 2: Wire `t` key**

**Step 3: Commit**

```bash
git add crates/sup/src/tui/tag_editor.rs
git commit -m "feat: tag editor popup for adding/removing tags in TUI"
```

---

## Final: Polish + Install

### Task 16: Install Script + README

**Files:**
- Create: `README.md`
- Create: `Makefile`

```makefile
# Makefile
install:
	cargo install --path crates/sup

test:
	cargo test --workspace

build:
	cargo build --release
```

**README.md** should cover:
- Installation (`make install` or `cargo install --path crates/sup`)
- DB location (`~/.sup/sup.db`)
- CLI quick reference (key commands)
- TUI key bindings table
- `--json` agent usage example

**Commit:**

```bash
git add README.md Makefile
git commit -m "docs: README with install instructions and key bindings reference"
```

---

## Summary

| Phase | Tasks | Outcome |
|-------|-------|---------|
| 1 — Foundation | 1–3 | Workspace, schema, domain types |
| 2 — Query Layer | 4–7 | Full CRUD, tags, search, carryover |
| 3 — CLI | 8–9 | All CLI commands with `--json` |
| 4 — TUI | 10–15 | Full TUI: journal, tasks, split, search, linking, tags |
| Final | 16 | Install + docs |

Each task ends with a commit. Run `cargo test --workspace` before each commit to catch regressions early.
