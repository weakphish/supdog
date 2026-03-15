# supdog Tauri Rewrite — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite supdog from a Rust TUI app to a Tauri + Svelte desktop app with daily journal, tag pages, block linking, mind maps, and quick capture.

**Architecture:** Tauri v2 Rust backend with SQLite (rusqlite) handles all data. SvelteKit SPA frontend (adapter-static, ssr=false) communicates via Tauri `#[command]` invoke. Two windows: main app + quick capture popup.

**Tech Stack:** Rust, Tauri v2, SvelteKit, Svelte 5 (runes), TypeScript, SQLite/rusqlite, FTS5, adapter-static

---

## File Structure

```
supdog/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── src/
│   │   ├── main.rs              # Tauri bootstrap, register commands
│   │   ├── db.rs                # SQLite connection pool, migration runner
│   │   ├── models.rs            # Block, DailyNote, Tag, BlockLink, MindMap, MindMapNode
│   │   ├── commands/
│   │   │   ├── mod.rs           # Re-exports all command modules
│   │   │   ├── blocks.rs        # CRUD blocks, reorder, reparent
│   │   │   ├── daily_notes.rs   # Get/create daily notes
│   │   │   ├── tags.rs          # CRUD tags, hierarchical creation
│   │   │   ├── links.rs         # Create/delete block links, query backlinks
│   │   │   ├── search.rs        # FTS5 search with filters
│   │   │   └── mindmaps.rs      # CRUD mind maps, nodes, send-to-journal
│   │   ├── quick_capture.rs     # Quick capture window management + global hotkey
│   │   └── migrations/
│   │       └── 001_initial.sql  # Full schema
├── src/
│   ├── app.html
│   ├── app.css                  # Global styles, typography, design tokens
│   ├── routes/
│   │   ├── +layout.svelte       # App shell: sidebar + main content area
│   │   ├── +layout.ts           # ssr = false
│   │   ├── +page.svelte         # Redirects to /journal/today
│   │   ├── journal/
│   │   │   └── [date]/
│   │   │       └── +page.svelte # Journal view for a specific date
│   │   ├── tag/
│   │   │   └── [...path]/
│   │   │       └── +page.svelte # Tag page view
│   │   ├── mindmap/
│   │   │   └── [id]/
│   │   │       └── +page.svelte # Mind map canvas
│   │   └── capture/
│   │       └── +page.svelte     # Quick capture window (separate Tauri window)
│   ├── lib/
│   │   ├── types.ts             # TypeScript types mirroring Rust models
│   │   ├── api.ts               # Tauri invoke wrappers
│   │   ├── stores/
│   │   │   ├── journal.svelte.ts   # Journal state (current date, blocks tree)
│   │   │   ├── tags.svelte.ts      # Tags list, current tag page
│   │   │   ├── search.svelte.ts    # Search query, results, filters
│   │   │   └── sidebar.svelte.ts   # Sidebar collapsed state, navigation
│   │   └── components/
│   │       ├── Sidebar.svelte         # Minimal sidebar: journal, tags, mind maps
│   │       ├── BlockTree.svelte       # Recursive block renderer
│   │       ├── BlockItem.svelte       # Single block: display + inline edit
│   │       ├── BlockEditor.svelte     # Inline text editor with # and [[ triggers
│   │       ├── TaskCheckbox.svelte    # Task status checkbox with cycle
│   │       ├── TagPill.svelte         # Clickable tag pill
│   │       ├── SearchOverlay.svelte   # / search floating panel
│   │       ├── TagAutocomplete.svelte # # autocomplete dropdown
│   │       ├── LinkSearch.svelte      # [[ block/task search dropdown
│   │       ├── DateNav.svelte         # Day navigation (prev/next/picker)
│   │       ├── TagPageTasks.svelte    # Open tasks section on tag page
│   │       ├── TagPageBlocks.svelte   # Chronological blocks on tag page
│   │       └── MindMapCanvas.svelte   # Canvas with nodes and connections
├── static/
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
└── package.json
```

---

## Chunk 1: Project Scaffold, Database, and Models

### Task 1: Initialize Tauri + SvelteKit project

**Files:**
- Create: `package.json`, `svelte.config.js`, `vite.config.ts`, `tsconfig.json`, `src/app.html`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`
- Create: `src/routes/+layout.ts`, `src/routes/+page.svelte`

- [ ] **Step 1: Scaffold SvelteKit project**

Run:
```bash
cd /Users/weakfish/Developer/supdog
# Remove old crates (will be replaced)
# Create new SvelteKit project in a temp dir, then move files
npx sv create supdog-new --template minimal --types ts --no-add-ons --no-install
```

- [ ] **Step 2: Move SvelteKit files into project root**

Move the SvelteKit scaffold files (`package.json`, `svelte.config.js`, `vite.config.ts`, `tsconfig.json`, `src/`) into the supdog root, replacing/alongside the old `crates/` structure.

- [ ] **Step 3: Install dependencies and add adapter-static**

Run:
```bash
npm install
npm i -D @sveltejs/adapter-static
```

- [ ] **Step 4: Configure SvelteKit as SPA**

Update `svelte.config.js`:
```js
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: '200.html'
    })
  }
};

export default config;
```

Create `src/routes/+layout.ts`:
```ts
export const ssr = false;
```

- [ ] **Step 5: Add Tauri v2 to the project**

Run:
```bash
npm install -D @tauri-apps/cli@latest
npm install @tauri-apps/api@latest
npx tauri init
```

After `tauri init`, update `src-tauri/tauri.conf.json`:
```json
{
  "build": {
    "devUrl": "http://localhost:5173",
    "frontendDist": "../build"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "supdog",
        "width": 1024,
        "height": 768
      }
    ]
  },
  "bundle": {
    "active": true,
    "icon": []
  }
}
```

Note: `tauri init` also generates `src-tauri/capabilities/default.json` — keep it as-is.

- [ ] **Step 6: Verify the scaffold builds**

Run:
```bash
npm run build
cd src-tauri && cargo build
```

Expected: Both build without errors.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri v2 + SvelteKit project"
```

---

### Task 2: SQLite schema and migration

**Files:**
- Create: `src-tauri/src/migrations/001_initial.sql`
- Create: `src-tauri/src/db.rs`

- [ ] **Step 1: Write the migration SQL**

Create `src-tauri/src/migrations/001_initial.sql`:
```sql
CREATE TABLE daily_notes (
    id TEXT PRIMARY KEY NOT NULL,
    date TEXT NOT NULL UNIQUE
);

CREATE TABLE blocks (
    id TEXT PRIMARY KEY NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    block_type TEXT NOT NULL DEFAULT 'bullet',
    parent_id TEXT REFERENCES blocks(id) ON DELETE CASCADE,
    daily_note_id TEXT REFERENCES daily_notes(id) ON DELETE CASCADE,
    position INTEGER NOT NULL DEFAULT 0,
    status TEXT,
    priority TEXT,
    due_date TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE INDEX idx_blocks_parent ON blocks(parent_id);
CREATE INDEX idx_blocks_daily_note ON blocks(daily_note_id);
CREATE INDEX idx_blocks_type ON blocks(block_type);

CREATE TABLE tags (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    parent_id TEXT REFERENCES tags(id) ON DELETE SET NULL
);

CREATE TABLE block_tags (
    block_id TEXT NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (block_id, tag_id)
);

CREATE TABLE block_links (
    id TEXT PRIMARY KEY NOT NULL,
    source_id TEXT NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    target_id TEXT NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(source_id, target_id)
);

CREATE TABLE mind_maps (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE mind_map_nodes (
    id TEXT PRIMARY KEY NOT NULL,
    mind_map_id TEXT NOT NULL REFERENCES mind_maps(id) ON DELETE CASCADE,
    block_id TEXT NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    x REAL NOT NULL DEFAULT 0.0,
    y REAL NOT NULL DEFAULT 0.0,
    UNIQUE(block_id)
);

-- FTS5 for full-text search on block content.
-- Note: blocks uses TEXT PRIMARY KEY, but SQLite still assigns an implicit integer rowid.
-- The FTS5 content_rowid and triggers reference this implicit rowid.
CREATE VIRTUAL TABLE blocks_fts USING fts5(content, content_rowid='rowid');

-- Auto-sync triggers
CREATE TRIGGER blocks_ai AFTER INSERT ON blocks BEGIN
    INSERT INTO blocks_fts(rowid, content) VALUES (NEW.rowid, NEW.content);
END;
CREATE TRIGGER blocks_ad AFTER DELETE ON blocks BEGIN
    INSERT INTO blocks_fts(blocks_fts, rowid, content) VALUES ('delete', OLD.rowid, OLD.content);
END;
CREATE TRIGGER blocks_au AFTER UPDATE OF content ON blocks BEGIN
    INSERT INTO blocks_fts(blocks_fts, rowid, content) VALUES ('delete', OLD.rowid, OLD.content);
    INSERT INTO blocks_fts(rowid, content) VALUES (NEW.rowid, NEW.content);
END;

PRAGMA foreign_keys = ON;
```

- [ ] **Step 2: Write db.rs**

Create `src-tauri/src/db.rs`:
```rust
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

pub struct DbState(pub Mutex<Connection>);

pub fn init_db(app_data_dir: PathBuf) -> Result<Connection, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(&app_data_dir)?;
    let db_path = app_data_dir.join("supdog.db");
    let mut conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

    let migrations = Migrations::new(vec![
        M::up(include_str!("migrations/001_initial.sql")),
    ]);
    migrations.to_latest(&mut conn)?;

    Ok(conn)
}
```

- [ ] **Step 3: Add Rust dependencies to Cargo.toml**

Update `src-tauri/Cargo.toml` to include:
```toml
[dependencies]
tauri = { version = "2", features = ["global-shortcut"] }
tauri-plugin-shell = "2"
rusqlite = { version = "0.31", features = ["bundled"] }
rusqlite_migration = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 4: Verify migration runs**

Write a temporary test in `main.rs` that calls `init_db` with a temp dir and asserts the tables exist:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_migration_runs() {
        let tmp = TempDir::new().unwrap();
        let conn = db::init_db(tmp.path().to_path_buf()).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='blocks'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
```

Run:
```bash
cd src-tauri && cargo test
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db.rs src-tauri/src/migrations/001_initial.sql src-tauri/Cargo.toml
git commit -m "feat: add SQLite schema, migrations, and db module"
```

---

### Task 3: Rust models

**Files:**
- Create: `src-tauri/src/models.rs`

- [ ] **Step 1: Write models.rs**

Create `src-tauri/src/models.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Bullet,
    H1,
    H2,
    H3,
    Quote,
    Code,
    Task,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    High,
    Med,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub content: String,
    pub block_type: BlockType,
    pub parent_id: Option<String>,
    pub daily_note_id: Option<String>,
    pub position: i64,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub due_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub children: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyNote {
    pub id: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockLink {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindMap {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindMapNode {
    pub id: String,
    pub mind_map_id: String,
    pub block_id: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub block: Block,
    pub parent_content: Option<String>,
    pub daily_note_date: Option<String>,
}
```

- [ ] **Step 2: Wire models into main.rs**

Add `mod db;` and `mod models;` to `src-tauri/src/main.rs`.

- [ ] **Step 3: Verify it compiles**

Run:
```bash
cd src-tauri && cargo check
```

Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/main.rs
git commit -m "feat: add Rust data models"
```

---

### Task 4: Tauri commands — daily notes and blocks

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/daily_notes.rs`
- Create: `src-tauri/src/commands/blocks.rs`

- [ ] **Step 1: Write test for get_or_create_daily_note**

Add to a test module in `daily_notes.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::TempDir;

    fn test_conn() -> Connection {
        let tmp = TempDir::new().unwrap();
        db::init_db(tmp.path().to_path_buf()).unwrap()
    }

    #[test]
    fn test_get_or_create_daily_note() {
        let conn = test_conn();
        let note = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(note.date, "2026-03-15");

        // Calling again returns same note
        let note2 = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(note.id, note2.id);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test test_get_or_create_daily_note`
Expected: FAIL — function not defined

- [ ] **Step 3: Implement daily_notes.rs**

Create `src-tauri/src/commands/daily_notes.rs`:
```rust
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use crate::models::DailyNote;
use crate::db::DbState;

pub fn get_or_create_daily_note_impl(conn: &Connection, date: &str) -> Result<DailyNote, String> {
    let existing: Option<DailyNote> = conn
        .query_row(
            "SELECT id, date FROM daily_notes WHERE date = ?1",
            params![date],
            |row| Ok(DailyNote { id: row.get(0)?, date: row.get(1)? }),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(note) = existing {
        return Ok(note);
    }

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO daily_notes (id, date) VALUES (?1, ?2)",
        params![id, date],
    )
    .map_err(|e| e.to_string())?;

    Ok(DailyNote { id, date: date.to_string() })
}

#[tauri::command]
pub fn get_or_create_daily_note(state: tauri::State<DbState>, date: String) -> Result<DailyNote, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_or_create_daily_note_impl(&conn, &date)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test test_get_or_create_daily_note`
Expected: PASS

- [ ] **Step 5: Write tests for block CRUD**

Add to `blocks.rs` test module:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::commands::daily_notes::get_or_create_daily_note_impl;
    use tempfile::TempDir;

    fn test_conn() -> Connection {
        let tmp = TempDir::new().unwrap();
        db::init_db(tmp.path().to_path_buf()).unwrap()
    }

    #[test]
    fn test_create_and_get_blocks() {
        let conn = test_conn();
        let note = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let block = create_block_impl(&conn, &note.id, None, "hello world", "bullet", 0).unwrap();
        assert_eq!(block.content, "hello world");

        let blocks = get_blocks_for_date_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].content, "hello world");
    }

    #[test]
    fn test_nested_blocks() {
        let conn = test_conn();
        let note = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let parent = create_block_impl(&conn, &note.id, None, "parent", "bullet", 0).unwrap();
        let child = create_block_impl(&conn, &note.id, Some(&parent.id), "child", "bullet", 0).unwrap();

        let blocks = get_blocks_for_date_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].children.len(), 1);
        assert_eq!(blocks[0].children[0].content, "child");
    }

    #[test]
    fn test_update_block() {
        let conn = test_conn();
        let note = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let block = create_block_impl(&conn, &note.id, None, "original", "bullet", 0).unwrap();
        update_block_impl(&conn, &block.id, Some("updated"), None, None, None, None).unwrap();

        let blocks = get_blocks_for_date_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(blocks[0].content, "updated");
    }

    #[test]
    fn test_delete_block() {
        let conn = test_conn();
        let note = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let block = create_block_impl(&conn, &note.id, None, "to delete", "bullet", 0).unwrap();
        delete_block_impl(&conn, &block.id).unwrap();

        let blocks = get_blocks_for_date_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(blocks.len(), 0);
    }

    #[test]
    fn test_reorder_blocks() {
        let conn = test_conn();
        let note = get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let a = create_block_impl(&conn, &note.id, None, "a", "bullet", 0).unwrap();
        let b = create_block_impl(&conn, &note.id, None, "b", "bullet", 1).unwrap();
        let c = create_block_impl(&conn, &note.id, None, "c", "bullet", 2).unwrap();

        // Move c to position 0
        reorder_block_impl(&conn, &c.id, None, 0).unwrap();

        let blocks = get_blocks_for_date_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(blocks[0].content, "c");
        assert_eq!(blocks[1].content, "a");
        assert_eq!(blocks[2].content, "b");
    }
}
```

- [ ] **Step 6: Run tests to verify they fail**

Run: `cd src-tauri && cargo test blocks::tests`
Expected: FAIL

- [ ] **Step 7: Implement blocks.rs**

Create `src-tauri/src/commands/blocks.rs`:
```rust
use rusqlite::{params, Connection};
use uuid::Uuid;
use std::collections::HashMap;
use crate::models::{Block, BlockType};
use crate::db::DbState;

fn row_to_block(row: &rusqlite::Row) -> rusqlite::Result<Block> {
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
    // Fetch all blocks for this date in one query
    let mut stmt = conn.prepare(
        "SELECT b.* FROM blocks b JOIN daily_notes dn ON b.daily_note_id = dn.id WHERE dn.date = ?1 ORDER BY b.position"
    ).map_err(|e| e.to_string())?;
    let all_blocks: Vec<Block> = stmt.query_map(params![date], |row| row_to_block(row))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Fetch tags for all blocks in this date
    let mut tag_stmt = conn.prepare(
        "SELECT bt.block_id, t.name FROM block_tags bt JOIN tags t ON bt.tag_id = t.id JOIN blocks b ON bt.block_id = b.id JOIN daily_notes dn ON b.daily_note_id = dn.id WHERE dn.date = ?1"
    ).map_err(|e| e.to_string())?;
    let mut tags_map: HashMap<String, Vec<String>> = HashMap::new();
    let _ = tag_stmt.query_map(params![date], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .for_each(|(bid, tname)| { tags_map.entry(bid).or_default().push(tname); });

    // Build tree: collect into map, then assemble parent-child
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
    // Build dynamic UPDATE with only provided fields
    let mut sets = vec!["updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')".to_string()];
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(v) = content { sets.push(format!("content = ?{}", sets.len() + 1)); values.push(Box::new(v.to_string())); }
    if let Some(v) = block_type { sets.push(format!("block_type = ?{}", sets.len() + 1)); values.push(Box::new(v.to_string())); }
    if let Some(v) = status { sets.push(format!("status = ?{}", sets.len() + 1)); values.push(Box::new(v.to_string())); }
    if let Some(v) = priority { sets.push(format!("priority = ?{}", sets.len() + 1)); values.push(Box::new(v.to_string())); }
    if let Some(v) = due_date { sets.push(format!("due_date = ?{}", sets.len() + 1)); values.push(Box::new(v.to_string())); }
    values.push(Box::new(id.to_string()));
    let sql = format!("UPDATE blocks SET {} WHERE id = ?{}", sets.join(", "), sets.len() + 1);
    conn.execute(&sql, rusqlite::params_from_iter(values.iter().map(|v| v.as_ref())))
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_block_impl(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM blocks WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn reorder_block_impl(conn: &Connection, id: &str, new_parent_id: Option<&str>, new_position: i64) -> Result<(), String> {
    // Get block's current parent to determine sibling scope
    let current_parent: Option<String> = conn.query_row(
        "SELECT parent_id FROM blocks WHERE id = ?1", params![id], |r| r.get(0)
    ).map_err(|e| e.to_string())?;

    // Update the block's position (and parent if changing)
    conn.execute(
        "UPDATE blocks SET parent_id = ?1, position = ?2 WHERE id = ?3",
        params![new_parent_id, new_position, id],
    ).map_err(|e| e.to_string())?;

    // Renumber all siblings under the target parent
    let mut stmt = conn.prepare(
        "SELECT id FROM blocks WHERE parent_id IS ?1 AND id != ?2 ORDER BY position"
    ).map_err(|e| e.to_string())?;
    let sibling_ids: Vec<String> = stmt.query_map(params![new_parent_id, id], |r| r.get(0))
        .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    let mut pos = 0i64;
    for sid in &sibling_ids {
        if pos == new_position { pos += 1; } // skip the slot for our block
        conn.execute("UPDATE blocks SET position = ?1 WHERE id = ?2", params![pos, sid])
            .map_err(|e| e.to_string())?;
        pos += 1;
    }
    Ok(())
}

pub fn reparent_block_impl(conn: &Connection, id: &str, new_parent_id: Option<&str>, position: i64) -> Result<(), String> {
    reorder_block_impl(conn, id, new_parent_id, position)
}
```

Each function also needs a Tauri `#[command]` wrapper that extracts `DbState` and delegates to the `_impl` function, following the same pattern as `get_or_create_daily_note`.

- [ ] **Step 8: Run tests to verify they pass**

Run: `cd src-tauri && cargo test blocks::tests`
Expected: All PASS

- [ ] **Step 9: Create commands/mod.rs and wire into main.rs**

Create `src-tauri/src/commands/mod.rs`:
```rust
pub mod blocks;
pub mod daily_notes;
```

Update `main.rs` to register commands:
```rust
mod db;
mod models;
mod commands;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            let conn = db::init_db(app_data_dir)?;
            app.manage(db::DbState(std::sync::Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::daily_notes::get_or_create_daily_note,
            commands::blocks::create_block,
            commands::blocks::get_blocks_for_date,
            commands::blocks::update_block,
            commands::blocks::delete_block,
            commands::blocks::reorder_block,
            commands::blocks::reparent_block,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 10: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: No errors.

- [ ] **Step 11: Commit**

```bash
git add src-tauri/src/commands/ src-tauri/src/main.rs
git commit -m "feat: add daily notes and blocks Tauri commands with tests"
```

---

### Task 5: Tauri commands — tags

**Files:**
- Create: `src-tauri/src/commands/tags.rs`

- [ ] **Step 1: Write tests for tag operations**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::TempDir;

    fn test_conn() -> Connection {
        let tmp = TempDir::new().unwrap();
        db::init_db(tmp.path().to_path_buf()).unwrap()
    }

    #[test]
    fn test_create_tag_with_hierarchy() {
        let conn = test_conn();
        let tag = create_tag_impl(&conn, "project/migration").unwrap();
        assert_eq!(tag.name, "project/migration");

        // Parent "project" should exist
        let parent: String = conn
            .query_row("SELECT name FROM tags WHERE id = ?1", params![tag.parent_id.unwrap()], |r| r.get(0))
            .unwrap();
        assert_eq!(parent, "project");
    }

    #[test]
    fn test_add_tag_to_block() {
        let conn = test_conn();
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
        let conn = test_conn();
        let note = crate::commands::daily_notes::get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let block = crate::commands::blocks::create_block_impl(&conn, &note.id, None, "tagged block", "bullet", 0).unwrap();
        let tag = create_tag_impl(&conn, "work").unwrap();
        add_tag_to_block_impl(&conn, &block.id, &tag.id).unwrap();

        let result = get_blocks_by_tag_impl(&conn, "work").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "tagged block");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test tags::tests`
Expected: FAIL

- [ ] **Step 3: Implement tags.rs**

Create `src-tauri/src/commands/tags.rs`:
```rust
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
    // Find the tag and all its descendants
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

    // Separate tasks (todo/in_progress) from other blocks
    let tasks: Vec<_> = blocks.iter().filter(|b| matches!(b.status.as_ref(), Some(s) if s == &crate::models::TaskStatus::Todo || s == &crate::models::TaskStatus::InProgress)).cloned().collect();
    let other: Vec<_> = blocks.iter().filter(|b| b.block_type != crate::models::BlockType::Task || matches!(b.status.as_ref(), Some(s) if s == &crate::models::TaskStatus::Done || s == &crate::models::TaskStatus::Cancelled)).cloned().collect();

    // TODO: attach children to each task block (reuse tree-building from blocks.rs)
    Ok(serde_json::json!({ "tasks": tasks, "blocks": other }))
}
```

Each function also needs a Tauri `#[command]` wrapper following the same pattern as daily_notes. Note: `row_to_block` in blocks.rs should be made `pub` so tags.rs can reuse it.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test tags::tests`
Expected: All PASS

- [ ] **Step 5: Register tag commands in mod.rs and main.rs**

Add `pub mod tags;` to `commands/mod.rs`. Register commands in `main.rs`.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/tags.rs src-tauri/src/commands/mod.rs src-tauri/src/main.rs
git commit -m "feat: add tag commands with hierarchical creation"
```

---

### Task 6: Tauri commands — links and search

**Files:**
- Create: `src-tauri/src/commands/links.rs`
- Create: `src-tauri/src/commands/search.rs`

- [ ] **Step 1: Write tests for links**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::commands::{daily_notes, blocks};
    use tempfile::TempDir;

    fn test_conn() -> Connection {
        let tmp = TempDir::new().unwrap();
        db::init_db(tmp.path().to_path_buf()).unwrap()
    }

    #[test]
    fn test_create_and_query_link() {
        let conn = test_conn();
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
        let conn = test_conn();
        let note = daily_notes::get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        let source = blocks::create_block_impl(&conn, &note.id, None, "source", "bullet", 0).unwrap();
        let target = blocks::create_block_impl(&conn, &note.id, None, "target", "task", 1).unwrap();

        let link = create_link_impl(&conn, &source.id, &target.id).unwrap();
        delete_link_impl(&conn, &link.id).unwrap();

        let backlinks = get_backlinks_impl(&conn, &target.id).unwrap();
        assert_eq!(backlinks.len(), 0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test links::tests`
Expected: FAIL

- [ ] **Step 3: Implement links.rs**

Create `src-tauri/src/commands/links.rs`:
```rust
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
    Ok(BlockLink { id, source_id: source_id.into(), target_id: target_id.into(), created_at: String::new() })
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
```

Each function also needs a Tauri `#[command]` wrapper.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test links::tests`
Expected: PASS

- [ ] **Step 5: Write tests for search**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::commands::{daily_notes, blocks};
    use tempfile::TempDir;

    fn test_conn() -> Connection {
        let tmp = TempDir::new().unwrap();
        db::init_db(tmp.path().to_path_buf()).unwrap()
    }

    #[test]
    fn test_fts_search() {
        let conn = test_conn();
        let note = daily_notes::get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        blocks::create_block_impl(&conn, &note.id, None, "discussed migration strategy", "bullet", 0).unwrap();
        blocks::create_block_impl(&conn, &note.id, None, "unrelated note about lunch", "bullet", 1).unwrap();

        let results = search_impl(&conn, "migration", None, None).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].block.content.contains("migration"));
    }

    #[test]
    fn test_search_with_type_filter() {
        let conn = test_conn();
        let note = daily_notes::get_or_create_daily_note_impl(&conn, "2026-03-15").unwrap();
        blocks::create_block_impl(&conn, &note.id, None, "migration note", "bullet", 0).unwrap();
        blocks::create_block_impl(&conn, &note.id, None, "migration task", "task", 1).unwrap();

        let results = search_impl(&conn, "migration", Some("task"), None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].block.block_type, BlockType::Task);
    }
}
```

- [ ] **Step 6: Run tests to verify they fail**

Run: `cd src-tauri && cargo test search::tests`
Expected: FAIL

- [ ] **Step 7: Implement search.rs**

Create `src-tauri/src/commands/search.rs`:
```rust
use rusqlite::{params, Connection};
use crate::models::SearchResult;
use crate::commands::blocks::row_to_block;
use crate::db::DbState;

/// FTS5 search with optional block_type and tag filters.
/// `status:open` in the query maps to todo + in_progress.
pub fn search_impl(conn: &Connection, query: &str, block_type_filter: Option<&str>, tag_filter: Option<&str>) -> Result<Vec<SearchResult>, String> {
    // Build the query dynamically based on filters
    let mut sql = String::from(
        "SELECT b.*, p.content as parent_content, dn.date as daily_note_date
         FROM blocks_fts fts
         JOIN blocks b ON b.rowid = fts.rowid
         LEFT JOIN blocks p ON b.parent_id = p.id
         LEFT JOIN daily_notes dn ON b.daily_note_id = dn.id"
    );
    let mut conditions = vec!["fts.blocks_fts MATCH ?1".to_string()];
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

    sql.push_str(&format!(" WHERE {} ORDER BY fts.rank LIMIT 50", conditions.join(" AND ")));

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let results = stmt.query_map(rusqlite::params_from_iter(&param_values), |row| {
        let block = row_to_block(row)?;
        let parent_content: Option<String> = row.get("parent_content")?;
        let daily_note_date: Option<String> = row.get("daily_note_date")?;
        Ok(SearchResult { block, parent_content, daily_note_date })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    Ok(results)
}
```

Tauri `#[command]` wrapper: `search(query: String, block_type_filter: Option<String>, tag_filter: Option<String>)`.

- [ ] **Step 8: Run tests to verify they pass**

Run: `cd src-tauri && cargo test search::tests`
Expected: PASS

- [ ] **Step 9: Register in mod.rs and main.rs**

Add `pub mod links;` and `pub mod search;` to `commands/mod.rs`. Register all new commands in `main.rs`.

- [ ] **Step 10: Run full test suite**

Run: `cd src-tauri && cargo test`
Expected: All tests PASS

- [ ] **Step 11: Commit**

```bash
git add src-tauri/src/commands/
git commit -m "feat: add link and search commands with FTS5"
```

---

### Task 7: Tauri commands — mind maps

**Files:**
- Create: `src-tauri/src/commands/mindmaps.rs`

- [ ] **Step 1: Write tests for mind map operations**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::commands::{daily_notes, blocks};
    use tempfile::TempDir;

    fn test_conn() -> Connection {
        let tmp = TempDir::new().unwrap();
        db::init_db(tmp.path().to_path_buf()).unwrap()
    }

    #[test]
    fn test_create_mind_map() {
        let conn = test_conn();
        let mm = create_mind_map_impl(&conn, "brainstorm").unwrap();
        assert_eq!(mm.name, "brainstorm");
    }

    #[test]
    fn test_add_node_to_mind_map() {
        let conn = test_conn();
        let mm = create_mind_map_impl(&conn, "brainstorm").unwrap();
        let node = add_mind_map_node_impl(&conn, &mm.id, "idea", 100.0, 200.0).unwrap();
        assert_eq!(node.x, 100.0);
        assert_eq!(node.y, 200.0);

        let nodes = get_mind_map_nodes_impl(&conn, &mm.id).unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_send_to_journal() {
        let conn = test_conn();
        let mm = create_mind_map_impl(&conn, "brainstorm").unwrap();
        let node = add_mind_map_node_impl(&conn, &mm.id, "idea to journal", 0.0, 0.0).unwrap();

        send_nodes_to_journal_impl(&conn, &[node.block_id.clone()], "2026-03-15").unwrap();

        // Node should be removed from mind map
        let nodes = get_mind_map_nodes_impl(&conn, &mm.id).unwrap();
        assert_eq!(nodes.len(), 0);

        // Block should now belong to today's journal
        let journal_blocks = crate::commands::blocks::get_blocks_for_date_impl(&conn, "2026-03-15").unwrap();
        assert_eq!(journal_blocks.len(), 1);
        assert_eq!(journal_blocks[0].content, "idea to journal");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test mindmaps::tests`
Expected: FAIL

- [ ] **Step 3: Implement mindmaps.rs**

Create `src-tauri/src/commands/mindmaps.rs`:
```rust
use rusqlite::{params, Connection};
use uuid::Uuid;
use crate::models::{MindMap, MindMapNode};
use crate::commands::daily_notes::get_or_create_daily_note_impl;
use crate::db::DbState;

pub fn create_mind_map_impl(conn: &Connection, name: &str) -> Result<MindMap, String> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO mind_maps (id, name) VALUES (?1, ?2)", params![id, name]
    ).map_err(|e| e.to_string())?;
    conn.query_row("SELECT id, name, created_at, updated_at FROM mind_maps WHERE id = ?1", params![id],
        |r| Ok(MindMap { id: r.get(0)?, name: r.get(1)?, created_at: r.get(2)?, updated_at: r.get(3)? })
    ).map_err(|e| e.to_string())
}

pub fn get_mind_maps_impl(conn: &Connection) -> Result<Vec<MindMap>, String> {
    let mut stmt = conn.prepare("SELECT id, name, created_at, updated_at FROM mind_maps ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;
    let maps = stmt.query_map([], |r| Ok(MindMap { id: r.get(0)?, name: r.get(1)?, created_at: r.get(2)?, updated_at: r.get(3)? }))
        .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    Ok(maps)
}

pub fn delete_mind_map_impl(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM mind_maps WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

/// Creates a Block (no daily_note_id) and a MindMapNode linking it to the mind map.
pub fn add_mind_map_node_impl(conn: &Connection, mind_map_id: &str, content: &str, x: f64, y: f64) -> Result<MindMapNode, String> {
    let block_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO blocks (id, content, block_type, position) VALUES (?1, ?2, 'bullet', 0)",
        params![block_id, content],
    ).map_err(|e| e.to_string())?;

    let node_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO mind_map_nodes (id, mind_map_id, block_id, x, y) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![node_id, mind_map_id, block_id, x, y],
    ).map_err(|e| e.to_string())?;

    Ok(MindMapNode { id: node_id, mind_map_id: mind_map_id.into(), block_id, x, y })
}

pub fn update_node_position_impl(conn: &Connection, node_id: &str, x: f64, y: f64) -> Result<(), String> {
    conn.execute("UPDATE mind_map_nodes SET x = ?1, y = ?2 WHERE id = ?3", params![x, y, node_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_mind_map_nodes_impl(conn: &Connection, mind_map_id: &str) -> Result<Vec<MindMapNode>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, mind_map_id, block_id, x, y FROM mind_map_nodes WHERE mind_map_id = ?1"
    ).map_err(|e| e.to_string())?;
    let nodes = stmt.query_map(params![mind_map_id], |r| Ok(MindMapNode {
        id: r.get(0)?, mind_map_id: r.get(1)?, block_id: r.get(2)?, x: r.get(3)?, y: r.get(4)?
    })).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    Ok(nodes)
}

/// Moves blocks from mind map into the journal for the given date.
pub fn send_nodes_to_journal_impl(conn: &Connection, block_ids: &[String], date: &str) -> Result<(), String> {
    let note = get_or_create_daily_note_impl(conn, date)?;
    // Get max position in the journal for appending
    let max_pos: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) FROM blocks WHERE daily_note_id = ?1 AND parent_id IS NULL",
        params![note.id], |r| r.get(0)
    ).unwrap_or(-1);

    for (i, block_id) in block_ids.iter().enumerate() {
        conn.execute(
            "UPDATE blocks SET daily_note_id = ?1, position = ?2 WHERE id = ?3",
            params![note.id, max_pos + 1 + i as i64, block_id],
        ).map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM mind_map_nodes WHERE block_id = ?1", params![block_id])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

Each function also needs a Tauri `#[command]` wrapper.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test mindmaps::tests`
Expected: PASS

- [ ] **Step 5: Register in mod.rs and main.rs**

Add `pub mod mindmaps;` to `commands/mod.rs`. Register all new commands.

- [ ] **Step 6: Run full test suite**

Run: `cd src-tauri && cargo test`
Expected: All tests PASS

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands/
git commit -m "feat: add mind map commands with send-to-journal"
```

---

## Chunk 2: Svelte Frontend — Types, API Layer, and Core Components

### Task 8: TypeScript types and Tauri API wrapper

**Files:**
- Create: `src/lib/types.ts`
- Create: `src/lib/api.ts`

- [ ] **Step 1: Write types.ts**

Create `src/lib/types.ts`:
```ts
export type BlockType = 'bullet' | 'h1' | 'h2' | 'h3' | 'quote' | 'code' | 'task';
export type TaskStatus = 'todo' | 'in_progress' | 'done' | 'cancelled';
export type Priority = 'high' | 'med' | 'low';

export interface Block {
  id: string;
  content: string;
  block_type: BlockType;
  parent_id: string | null;
  daily_note_id: string | null;
  position: number;
  status: TaskStatus | null;
  priority: Priority | null;
  due_date: string | null;
  created_at: string;
  updated_at: string;
  tags: string[];
  children: Block[];
}

export interface DailyNote {
  id: string;
  date: string;
}

export interface Tag {
  id: string;
  name: string;
  parent_id: string | null;
}

export interface BlockLink {
  id: string;
  source_id: string;
  target_id: string;
  created_at: string;
}

export interface MindMap {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface MindMapNode {
  id: string;
  mind_map_id: string;
  block_id: string;
  x: number;
  y: number;
}

export interface SearchResult {
  block: Block;
  parent_content: string | null;
  daily_note_date: string | null;
}
```

- [ ] **Step 2: Write api.ts**

Create `src/lib/api.ts`:
```ts
import { invoke } from '@tauri-apps/api/core';
import type { Block, DailyNote, Tag, BlockLink, MindMap, MindMapNode, SearchResult } from './types';

// Daily Notes
export const getOrCreateDailyNote = (date: string) =>
  invoke<DailyNote>('get_or_create_daily_note', { date });

// Blocks
export const getBlocksForDate = (date: string) =>
  invoke<Block[]>('get_blocks_for_date', { date });

export const createBlock = (dailyNoteId: string, parentId: string | null, content: string, blockType: string, position: number) =>
  invoke<Block>('create_block', { dailyNoteId, parentId, content, blockType, position });

export const updateBlock = (id: string, content?: string, blockType?: string, status?: string, priority?: string, dueDate?: string) =>
  invoke<void>('update_block', { id, content, blockType, status, priority, dueDate });

export const deleteBlock = (id: string) =>
  invoke<void>('delete_block', { id });

export const reorderBlock = (id: string, newParentId: string | null, newPosition: number) =>
  invoke<void>('reorder_block', { id, newParentId, newPosition });

export const reparentBlock = (id: string, newParentId: string | null, position: number) =>
  invoke<void>('reparent_block', { id, newParentId, position });

// Tags
export const getAllTags = () =>
  invoke<Tag[]>('get_all_tags');

export const createTag = (name: string) =>
  invoke<Tag>('create_tag', { name });

export const addTagToBlock = (blockId: string, tagId: string) =>
  invoke<void>('add_tag_to_block', { blockId, tagId });

export const removeTagFromBlock = (blockId: string, tagId: string) =>
  invoke<void>('remove_tag_from_block', { blockId, tagId });

export const getBlocksByTag = (tagName: string) =>
  invoke<{ tasks: Block[]; blocks: Block[] }>('get_blocks_by_tag', { tagName });

// Links
export const createLink = (sourceId: string, targetId: string) =>
  invoke<BlockLink>('create_link', { sourceId, targetId });

export const deleteLink = (id: string) =>
  invoke<void>('delete_link', { id });

export const getBacklinks = (blockId: string) =>
  invoke<Block[]>('get_backlinks', { blockId });

export const getForwardLinks = (blockId: string) =>
  invoke<Block[]>('get_forward_links', { blockId });

// Search
export const search = (query: string, blockTypeFilter?: string, tagFilter?: string) =>
  invoke<SearchResult[]>('search', { query, blockTypeFilter, tagFilter });

// Mind Maps
export const createMindMap = (name: string) =>
  invoke<MindMap>('create_mind_map', { name });

export const getMindMaps = () =>
  invoke<MindMap[]>('get_mind_maps');

export const deleteMindMap = (id: string) =>
  invoke<void>('delete_mind_map', { id });

export const addMindMapNode = (mindMapId: string, content: string, x: number, y: number) =>
  invoke<MindMapNode>('add_mind_map_node', { mindMapId, content, x, y });

export const updateNodePosition = (nodeId: string, x: number, y: number) =>
  invoke<void>('update_node_position', { nodeId, x, y });

export const getMindMapNodes = (mindMapId: string) =>
  invoke<MindMapNode[]>('get_mind_map_nodes', { mindMapId });

export const sendNodesToJournal = (blockIds: string[], date: string) =>
  invoke<void>('send_nodes_to_journal', { blockIds, date });
```

- [ ] **Step 3: Verify TypeScript compiles**

Run: `npx tsc --noEmit`
Expected: No errors (or only SvelteKit-related type issues that don't affect these files).

- [ ] **Step 4: Commit**

```bash
git add src/lib/types.ts src/lib/api.ts
git commit -m "feat: add TypeScript types and Tauri API wrapper"
```

---

### Task 9: Global styles and design tokens

**Files:**
- Create: `src/app.css`

- [ ] **Step 1: Write app.css with design tokens**

Create `src/app.css`:
```css
:root {
  /* Typography */
  --font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --font-mono: 'JetBrains Mono', 'SF Mono', 'Fira Code', monospace;

  --text-xs: 0.75rem;
  --text-sm: 0.875rem;
  --text-base: 1rem;
  --text-lg: 1.125rem;
  --text-xl: 1.25rem;
  --text-2xl: 1.5rem;
  --text-3xl: 2rem;

  --leading-tight: 1.25;
  --leading-normal: 1.5;
  --leading-relaxed: 1.75;

  /* Spacing */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;
  --space-6: 1.5rem;
  --space-8: 2rem;
  --space-12: 3rem;
  --space-16: 4rem;

  /* Colors — light, editorial palette */
  --bg: #fafafa;
  --bg-surface: #ffffff;
  --bg-muted: #f5f5f5;
  --bg-hover: #f0f0f0;

  --text-primary: #1a1a1a;
  --text-secondary: #6b6b6b;
  --text-muted: #a0a0a0;
  --text-inverse: #ffffff;

  --border: #e5e5e5;
  --border-strong: #d0d0d0;

  --accent: #3b3b3b;
  --accent-hover: #2a2a2a;

  --tag-bg: #f0f0f0;
  --tag-text: #555;

  --task-todo: #e0a000;
  --task-in-progress: #3080e0;
  --task-done: #40a040;
  --task-cancelled: #a0a0a0;

  /* Sidebar */
  --sidebar-width: 240px;
  --sidebar-collapsed-width: 48px;

  /* Shadows */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.04);
  --shadow-md: 0 2px 8px rgba(0, 0, 0, 0.06);
  --shadow-lg: 0 4px 24px rgba(0, 0, 0, 0.08);

  /* Transitions */
  --transition-fast: 100ms ease;
  --transition-normal: 200ms ease;

  /* Block nesting indent */
  --indent: 24px;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  font-family: var(--font-sans);
  font-size: var(--text-base);
  line-height: var(--leading-normal);
  color: var(--text-primary);
  background: var(--bg);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

::selection {
  background: var(--accent);
  color: var(--text-inverse);
}
```

- [ ] **Step 2: Commit**

```bash
git add src/app.css src/app.html
git commit -m "feat: add global styles and design tokens"
```

---

### Task 10: App layout shell and sidebar

**Files:**
- Create: `src/routes/+layout.svelte`
- Create: `src/lib/stores/sidebar.svelte.ts`
- Create: `src/lib/components/Sidebar.svelte`

- [ ] **Step 1: Write sidebar store**

Create `src/lib/stores/sidebar.svelte.ts`:
```ts
import { getAllTags, getMindMaps } from '$lib/api';
import type { Tag, MindMap } from '$lib/types';

let collapsed = $state(false);
let tags = $state<Tag[]>([]);
let mindMaps = $state<MindMap[]>([]);

export function sidebarState() {
  return {
    get collapsed() { return collapsed; },
    set collapsed(v: boolean) { collapsed = v; },
    get tags() { return tags; },
    get mindMaps() { return mindMaps; },
    toggle() { collapsed = !collapsed; },
    async loadTags() {
      tags = await getAllTags();
    },
    async loadMindMaps() {
      mindMaps = await getMindMaps();
    }
  };
}
```

- [ ] **Step 2: Write Sidebar component**

Create `src/lib/components/Sidebar.svelte`:
```svelte
<script lang="ts">
  import { sidebarState } from '$lib/stores/sidebar.svelte';
  import type { Tag } from '$lib/types';

  const sidebar = sidebarState();

  function todayDate(): string {
    return new Date().toISOString().split('T')[0];
  }

  // Group tags into tree structure for display
  function topLevelTags(tags: Tag[]): Tag[] {
    return tags.filter(t => !t.parent_id);
  }
</script>

<nav class="sidebar" class:collapsed={sidebar.collapsed}>
  <div class="sidebar-header">
    <button class="toggle" onclick={() => sidebar.toggle()}>
      {sidebar.collapsed ? '→' : '←'}
    </button>
  </div>

  {#if !sidebar.collapsed}
    <div class="sidebar-content">
      <a href="/journal/{todayDate()}" class="nav-item">Journal</a>

      <div class="nav-section">
        <span class="nav-label">Tags</span>
        {#each topLevelTags(sidebar.tags) as tag}
          <a href="/tag/{tag.name}" class="nav-item nav-indent">{tag.name}</a>
        {/each}
      </div>

      <div class="nav-section">
        <span class="nav-label">Mind Maps</span>
        {#each sidebar.mindMaps as mm}
          <a href="/mindmap/{mm.id}" class="nav-item nav-indent">{mm.name}</a>
        {/each}
      </div>
    </div>
  {/if}
</nav>

<style>
  .sidebar {
    width: var(--sidebar-width);
    height: 100vh;
    border-right: 1px solid var(--border);
    background: var(--bg-surface);
    display: flex;
    flex-direction: column;
    transition: width var(--transition-normal);
    overflow: hidden;
    flex-shrink: 0;
  }
  .sidebar.collapsed {
    width: var(--sidebar-collapsed-width);
  }
  .sidebar-header {
    padding: var(--space-3);
    display: flex;
    justify-content: flex-end;
  }
  .toggle {
    background: none;
    border: none;
    cursor: pointer;
    font-size: var(--text-sm);
    color: var(--text-muted);
    padding: var(--space-1) var(--space-2);
  }
  .toggle:hover {
    color: var(--text-primary);
  }
  .sidebar-content {
    padding: 0 var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
  .nav-item {
    display: block;
    padding: var(--space-1) var(--space-2);
    color: var(--text-secondary);
    text-decoration: none;
    font-size: var(--text-sm);
    border-radius: 4px;
  }
  .nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .nav-indent {
    padding-left: var(--space-6);
  }
  .nav-section {
    margin-top: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
  .nav-label {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    padding: 0 var(--space-2);
  }
</style>
```

- [ ] **Step 3: Write root layout**

Create `src/routes/+layout.svelte`:
```svelte
<script lang="ts">
  import '../app.css';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { sidebarState } from '$lib/stores/sidebar.svelte';
  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  const sidebar = sidebarState();

  onMount(() => {
    sidebar.loadTags();
    sidebar.loadMindMaps();
  });
</script>

<div class="app-shell">
  <Sidebar />
  <main class="main-content">
    {@render children()}
  </main>
</div>

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }
  .main-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-8) var(--space-12);
    max-width: 720px;
    margin: 0 auto;
  }
</style>
```

Note: `{@render children()}` requires Svelte 5's snippet-based slots. The `children` prop is automatically provided by SvelteKit layouts.

- [ ] **Step 4: Write redirect page**

Create `src/routes/+page.svelte`:
```svelte
<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  onMount(() => {
    const today = new Date().toISOString().split('T')[0];
    goto(`/journal/${today}`);
  });
</script>
```

- [ ] **Step 5: Verify frontend builds**

Run: `npm run build`
Expected: Build succeeds.

- [ ] **Step 6: Commit**

```bash
git add src/routes/ src/lib/stores/ src/lib/components/ src/app.css
git commit -m "feat: add app layout shell with sidebar"
```

---

### Task 11: BlockItem, BlockTree, and TaskCheckbox components

**Files:**
- Create: `src/lib/components/BlockItem.svelte`
- Create: `src/lib/components/BlockTree.svelte`
- Create: `src/lib/components/TaskCheckbox.svelte`
- Create: `src/lib/components/TagPill.svelte`

- [ ] **Step 1: Write TaskCheckbox component**

Create `src/lib/components/TaskCheckbox.svelte`:
```svelte
<script lang="ts">
  import type { TaskStatus } from '$lib/types';

  let { status, onchange }: { status: TaskStatus; onchange: (newStatus: TaskStatus) => void } = $props();

  const cycle: Record<TaskStatus, TaskStatus> = {
    todo: 'in_progress',
    in_progress: 'done',
    done: 'cancelled',
    cancelled: 'todo',
  };

  function handleClick() {
    onchange(cycle[status]);
  }
</script>

<button class="task-checkbox {status}" onclick={handleClick} title={status}>
  {#if status === 'todo'}☐{/if}
  {#if status === 'in_progress'}◐{/if}
  {#if status === 'done'}☑{/if}
  {#if status === 'cancelled'}☒{/if}
</button>

<style>
  .task-checkbox {
    background: none;
    border: none;
    cursor: pointer;
    font-size: var(--text-base);
    padding: 0;
    line-height: 1;
    flex-shrink: 0;
  }
  .todo { color: var(--task-todo); }
  .in_progress { color: var(--task-in-progress); }
  .done { color: var(--task-done); }
  .cancelled { color: var(--task-cancelled); }
</style>
```

- [ ] **Step 2: Write TagPill component**

Create `src/lib/components/TagPill.svelte`:
```svelte
<script lang="ts">
  let { name }: { name: string } = $props();
</script>

<a href="/tag/{name}" class="tag-pill">#{name}</a>

<style>
  .tag-pill {
    display: inline-block;
    font-size: var(--text-xs);
    padding: 1px 8px;
    background: var(--tag-bg);
    color: var(--tag-text);
    border-radius: 3px;
    text-decoration: none;
    margin-left: var(--space-2);
  }
  .tag-pill:hover {
    background: var(--border-strong);
  }
</style>
```

- [ ] **Step 3: Write BlockItem component**

Create `src/lib/components/BlockItem.svelte`:
```svelte
<script lang="ts">
  import type { Block } from '$lib/types';
  import TaskCheckbox from './TaskCheckbox.svelte';
  import TagPill from './TagPill.svelte';
  import { updateBlock } from '$lib/api';

  let { block, onedit, ondelete }: {
    block: Block;
    onedit: (id: string) => void;
    ondelete: (id: string) => void;
  } = $props();

  let editing = $state(false);
  let editContent = $state(block.content);

  function startEdit() {
    editing = true;
    editContent = block.content;
  }

  async function commitEdit() {
    if (editContent !== block.content) {
      await updateBlock(block.id, editContent);
    }
    editing = false;
    onedit(block.id);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      commitEdit();
    }
    if (e.key === 'Escape') {
      editing = false;
    }
  }

  async function handleStatusChange(newStatus: TaskStatus) {
    await updateBlock(block.id, undefined, undefined, newStatus);
    onedit(block.id);
  }

  const isDone = $derived(block.status === 'done');
  const isCancelled = $derived(block.status === 'cancelled');
</script>

<div class="block-item" class:done={isDone} class:cancelled={isCancelled}>
  <div class="block-line">
    {#if block.block_type === 'task' && block.status}
      <TaskCheckbox status={block.status} onchange={handleStatusChange} />
    {/if}

    {#if editing}
      <input
        class="block-editor"
        bind:value={editContent}
        onkeydown={handleKeydown}
        onblur={commitEdit}
        autofocus
      />
    {:else}
      <span
        class="block-content"
        class:task-content={block.block_type === 'task'}
        onclick={startEdit}
        role="button"
        tabindex="0"
      >
        {block.content}
      </span>
    {/if}

    {#each block.tags as tag}
      <TagPill name={tag} />
    {/each}
  </div>
</div>

<style>
  .block-item {
    padding: var(--space-1) 0;
  }
  .block-line {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
  }
  .block-content {
    cursor: text;
    flex: 1;
  }
  .block-content:hover {
    background: var(--bg-hover);
    border-radius: 2px;
  }
  .task-content {
    /* no special style beyond checkbox */
  }
  .done .block-content {
    text-decoration: line-through;
    color: var(--text-muted);
  }
  .cancelled .block-content {
    text-decoration: line-through;
    color: var(--text-muted);
  }
  .block-editor {
    flex: 1;
    border: none;
    outline: none;
    font: inherit;
    padding: var(--space-1);
    background: var(--bg-muted);
    border-radius: 2px;
  }
</style>
```

- [ ] **Step 4: Write BlockTree component**

Create `src/lib/components/BlockTree.svelte`:
```svelte
<script lang="ts">
  import type { Block } from '$lib/types';
  import BlockItem from './BlockItem.svelte';

  let { blocks, depth = 0, onedit, ondelete }: {
    blocks: Block[];
    depth?: number;
    onedit: (id: string) => void;
    ondelete: (id: string) => void;
  } = $props();
</script>

<div class="block-tree" style="padding-left: {depth > 0 ? 'var(--indent)' : '0'}">
  {#each blocks as block (block.id)}
    <BlockItem {block} {onedit} {ondelete} />
    {#if block.children.length > 0}
      <svelte:self blocks={block.children} depth={depth + 1} {onedit} {ondelete} />
    {/if}
  {/each}
</div>

<style>
  .block-tree {
    /* Minimal — just indentation via inline style */
  }
</style>
```

- [ ] **Step 5: Verify frontend builds**

Run: `npm run build`
Expected: Build succeeds.

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/
git commit -m "feat: add BlockItem, BlockTree, TaskCheckbox, TagPill components"
```

---

## Chunk 3: Journal View, Tag Page, and Search

### Task 12: Journal view

**Files:**
- Create: `src/lib/stores/journal.svelte.ts`
- Create: `src/routes/journal/[date]/+page.svelte`
- Create: `src/lib/components/DateNav.svelte`

- [ ] **Step 1: Write journal store**

Create `src/lib/stores/journal.svelte.ts`:
```ts
import { getOrCreateDailyNote, getBlocksForDate, createBlock, deleteBlock as apiDeleteBlock } from '$lib/api';
import type { Block, DailyNote } from '$lib/types';

let currentDate = $state('');
let dailyNote = $state<DailyNote | null>(null);
let blocks = $state<Block[]>([]);
let loading = $state(false);

export function journalState() {
  return {
    get currentDate() { return currentDate; },
    get dailyNote() { return dailyNote; },
    get blocks() { return blocks; },
    get loading() { return loading; },

    async loadDate(date: string) {
      loading = true;
      currentDate = date;
      dailyNote = await getOrCreateDailyNote(date);
      blocks = await getBlocksForDate(date);
      loading = false;
    },

    async refresh() {
      if (currentDate) {
        blocks = await getBlocksForDate(currentDate);
      }
    },

    async addBlock(parentId: string | null, content: string, blockType: string, position: number) {
      if (!dailyNote) return;
      await createBlock(dailyNote.id, parentId, content, blockType, position);
      await this.refresh();
    },

    async removeBlock(id: string) {
      await apiDeleteBlock(id);
      await this.refresh();
    }
  };
}
```

- [ ] **Step 2: Write DateNav component**

Create `src/lib/components/DateNav.svelte`:
```svelte
<script lang="ts">
  import { goto } from '$app/navigation';

  let { date }: { date: string } = $props();

  function offsetDate(days: number): string {
    const d = new Date(date + 'T00:00:00');
    d.setDate(d.getDate() + days);
    return d.toISOString().split('T')[0];
  }

  function formatDate(dateStr: string): string {
    const d = new Date(dateStr + 'T00:00:00');
    return d.toLocaleDateString('en-US', { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' });
  }

  function prev() { goto(`/journal/${offsetDate(-1)}`); }
  function next() { goto(`/journal/${offsetDate(1)}`); }
  function goToday() {
    const today = new Date().toISOString().split('T')[0];
    goto(`/journal/${today}`);
  }
</script>

<div class="date-nav">
  <button onclick={prev} class="nav-btn" title="Previous day">←</button>
  <h1 class="date-header">{formatDate(date)}</h1>
  <button onclick={next} class="nav-btn" title="Next day">→</button>
  <button onclick={goToday} class="today-btn">Today</button>
</div>

<style>
  .date-nav {
    display: flex;
    align-items: baseline;
    gap: var(--space-4);
    margin-bottom: var(--space-8);
  }
  .date-header {
    font-size: var(--text-2xl);
    font-weight: 600;
    line-height: var(--leading-tight);
  }
  .nav-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: var(--text-lg);
    color: var(--text-muted);
    padding: var(--space-1);
  }
  .nav-btn:hover { color: var(--text-primary); }
  .today-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    font-size: var(--text-sm);
    color: var(--text-secondary);
    padding: var(--space-1) var(--space-3);
  }
  .today-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
</style>
```

- [ ] **Step 3: Write journal page**

Create `src/routes/journal/[date]/+page.svelte`:
```svelte
<script lang="ts">
  import { page } from '$app/state';
  import BlockTree from '$lib/components/BlockTree.svelte';
  import DateNav from '$lib/components/DateNav.svelte';
  import { journalState } from '$lib/stores/journal.svelte';

  const journal = journalState();

  // Reactive: reload when date param changes
  $effect(() => {
    const date = page.params.date;
    if (date) {
      journal.loadDate(date);
    }
  });

  function handleEdit(id: string) {
    journal.refresh();
  }

  function handleDelete(id: string) {
    journal.removeBlock(id);
  }

  function handleKeydown(e: KeyboardEvent) {
    // Left/right arrow for day navigation when not editing
    if (e.key === 'ArrowLeft' && e.altKey) {
      const d = new Date(page.params.date + 'T00:00:00');
      d.setDate(d.getDate() - 1);
      window.location.href = `/journal/${d.toISOString().split('T')[0]}`;
    }
    if (e.key === 'ArrowRight' && e.altKey) {
      const d = new Date(page.params.date + 'T00:00:00');
      d.setDate(d.getDate() + 1);
      window.location.href = `/journal/${d.toISOString().split('T')[0]}`;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<DateNav date={page.params.date} />

{#if journal.loading}
  <p class="loading">Loading...</p>
{:else if journal.blocks.length === 0}
  <p class="empty">No entries yet. Start typing to add one.</p>
{:else}
  <BlockTree blocks={journal.blocks} onedit={handleEdit} ondelete={handleDelete} />
{/if}

<style>
  .loading, .empty {
    color: var(--text-muted);
    font-size: var(--text-sm);
    padding: var(--space-8) 0;
  }
</style>
```

- [ ] **Step 4: Verify frontend builds**

Run: `npm run build`
Expected: Build succeeds.

- [ ] **Step 5: Commit**

```bash
git add src/routes/journal/ src/lib/stores/journal.svelte.ts src/lib/components/DateNav.svelte
git commit -m "feat: add journal view with date navigation"
```

---

### Task 13: Tag page view

**Files:**
- Create: `src/lib/stores/tags.svelte.ts`
- Create: `src/routes/tag/[...path]/+page.svelte`
- Create: `src/lib/components/TagPageTasks.svelte`
- Create: `src/lib/components/TagPageBlocks.svelte`

- [ ] **Step 1: Write tags store**

Create `src/lib/stores/tags.svelte.ts`:
```ts
import { getBlocksByTag } from '$lib/api';
import type { Block } from '$lib/types';

let currentTag = $state('');
let tasks = $state<Block[]>([]);
let blocks = $state<Block[]>([]);
let loading = $state(false);

export function tagPageState() {
  return {
    get currentTag() { return currentTag; },
    get tasks() { return tasks; },
    get blocks() { return blocks; },
    get loading() { return loading; },

    async loadTag(tagName: string) {
      loading = true;
      currentTag = tagName;
      const result = await getBlocksByTag(tagName);
      tasks = result.tasks;
      blocks = result.blocks;
      loading = false;
    },

    async refresh() {
      if (currentTag) await this.loadTag(currentTag);
    }
  };
}
```

- [ ] **Step 2: Write TagPageTasks component**

Create `src/lib/components/TagPageTasks.svelte`:
```svelte
<script lang="ts">
  import type { Block } from '$lib/types';
  import BlockItem from './BlockItem.svelte';
  import BlockTree from './BlockTree.svelte';

  let { tasks, onedit }: { tasks: Block[]; onedit: (id: string) => void } = $props();
</script>

{#if tasks.length > 0}
  <section class="tasks-section">
    <h3 class="section-label">Open Tasks</h3>
    {#each tasks as task (task.id)}
      <div class="task-with-context">
        <BlockItem block={task} {onedit} ondelete={() => {}} />
        {#if task.children.length > 0}
          <BlockTree blocks={task.children} depth={1} {onedit} ondelete={() => {}} />
        {/if}
      </div>
    {/each}
  </section>
{/if}

<style>
  .tasks-section {
    margin-bottom: var(--space-8);
  }
  .section-label {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    margin-bottom: var(--space-3);
  }
  .task-with-context {
    margin-bottom: var(--space-3);
  }
</style>
```

- [ ] **Step 3: Write TagPageBlocks component**

Create `src/lib/components/TagPageBlocks.svelte`:
```svelte
<script lang="ts">
  import type { Block } from '$lib/types';
  import BlockItem from './BlockItem.svelte';

  let { blocks, onedit }: { blocks: Block[]; onedit: (id: string) => void } = $props();

  // Group blocks by date
  interface DateGroup {
    date: string;
    blocks: Block[];
  }

  const grouped: DateGroup[] = $derived.by(() => {
    const groups = new Map<string, Block[]>();
    for (const block of blocks) {
      const date = block.created_at.split('T')[0];
      if (!groups.has(date)) groups.set(date, []);
      groups.get(date)!.push(block);
    }
    return Array.from(groups.entries())
      .sort(([a], [b]) => b.localeCompare(a))
      .map(([date, blocks]) => ({ date, blocks }));
  });

  function formatDate(dateStr: string): string {
    const d = new Date(dateStr + 'T00:00:00');
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  }
</script>

{#if grouped.length > 0}
  <section>
    <h3 class="section-label">Blocks</h3>
    {#each grouped as group (group.date)}
      <div class="date-group">
        <a href="/journal/{group.date}" class="date-divider">{formatDate(group.date)}</a>
        {#each group.blocks as block (block.id)}
          <BlockItem {block} {onedit} ondelete={() => {}} />
        {/each}
      </div>
    {/each}
  </section>
{/if}

<style>
  .section-label {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    margin-bottom: var(--space-3);
  }
  .date-group {
    margin-bottom: var(--space-4);
  }
  .date-divider {
    display: block;
    font-size: var(--text-xs);
    color: var(--text-muted);
    text-decoration: none;
    margin-bottom: var(--space-2);
    padding-bottom: var(--space-1);
    border-bottom: 1px solid var(--border);
  }
  .date-divider:hover {
    color: var(--text-secondary);
  }
</style>
```

- [ ] **Step 4: Write tag page route**

Create `src/routes/tag/[...path]/+page.svelte`:
```svelte
<script lang="ts">
  import { page } from '$app/state';
  import TagPageTasks from '$lib/components/TagPageTasks.svelte';
  import TagPageBlocks from '$lib/components/TagPageBlocks.svelte';
  import { tagPageState } from '$lib/stores/tags.svelte';

  const tagPage = tagPageState();

  $effect(() => {
    const path = page.params.path;
    if (path) {
      tagPage.loadTag(path);
    }
  });

  function handleEdit() {
    tagPage.refresh();
  }
</script>

<h1 class="tag-header">#{tagPage.currentTag}</h1>

{#if tagPage.loading}
  <p class="loading">Loading...</p>
{:else}
  <TagPageTasks tasks={tagPage.tasks} onedit={handleEdit} />
  <TagPageBlocks blocks={tagPage.blocks} onedit={handleEdit} />
{/if}

<style>
  .tag-header {
    font-size: var(--text-3xl);
    font-weight: 700;
    margin-bottom: var(--space-8);
  }
  .loading {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }
</style>
```

- [ ] **Step 5: Verify frontend builds**

Run: `npm run build`
Expected: Build succeeds.

- [ ] **Step 6: Commit**

```bash
git add src/routes/tag/ src/lib/stores/tags.svelte.ts src/lib/components/TagPage*
git commit -m "feat: add tag page view with tasks and blocks sections"
```

---

### Task 14: Search overlay

**Files:**
- Create: `src/lib/stores/search.svelte.ts`
- Create: `src/lib/components/SearchOverlay.svelte`

- [ ] **Step 1: Write search store**

Create `src/lib/stores/search.svelte.ts`:
```ts
import { search as apiSearch } from '$lib/api';
import type { SearchResult } from '$lib/types';

let open = $state(false);
let query = $state('');
let results = $state<SearchResult[]>([]);
let selectedIndex = $state(0);
let loading = $state(false);

export function searchState() {
  return {
    get open() { return open; },
    set open(v: boolean) { open = v; if (!v) { query = ''; results = []; selectedIndex = 0; } },
    get query() { return query; },
    set query(v: string) { query = v; },
    get results() { return results; },
    get selectedIndex() { return selectedIndex; },
    set selectedIndex(v: number) { selectedIndex = v; },
    get loading() { return loading; },

    async doSearch(q: string) {
      query = q;
      if (q.length < 2) { results = []; return; }
      loading = true;
      results = await apiSearch(q);
      selectedIndex = 0;
      loading = false;
    },

    moveUp() {
      if (selectedIndex > 0) selectedIndex--;
    },

    moveDown() {
      if (selectedIndex < results.length - 1) selectedIndex++;
    },

    selectedResult(): SearchResult | null {
      return results[selectedIndex] ?? null;
    }
  };
}
```

- [ ] **Step 2: Write SearchOverlay component**

Create `src/lib/components/SearchOverlay.svelte`:
```svelte
<script lang="ts">
  import { goto } from '$app/navigation';
  import { searchState } from '$lib/stores/search.svelte';
  import TagPill from './TagPill.svelte';

  const searchStore = searchState();

  let inputEl: HTMLInputElement;

  $effect(() => {
    if (searchStore.open && inputEl) {
      inputEl.focus();
    }
  });

  let debounceTimer: ReturnType<typeof setTimeout>;

  function handleInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => searchStore.doSearch(value), 150);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      searchStore.open = false;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      searchStore.moveUp();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      searchStore.moveDown();
    } else if (e.key === 'Enter') {
      const result = searchStore.selectedResult();
      if (result?.daily_note_date) {
        goto(`/journal/${result.daily_note_date}`);
        searchStore.open = false;
      }
    }
  }
</script>

{#if searchStore.open}
  <div class="search-backdrop" onclick={() => searchStore.open = false} role="presentation">
    <div class="search-panel" onclick={(e) => e.stopPropagation()} role="dialog">
      <input
        bind:this={inputEl}
        class="search-input"
        placeholder="Search..."
        value={searchStore.query}
        oninput={handleInput}
        onkeydown={handleKeydown}
      />

      {#if searchStore.results.length > 0}
        <div class="search-results">
          {#each searchStore.results as result, i (result.block.id)}
            <button
              class="search-result"
              class:selected={i === searchStore.selectedIndex}
              onclick={() => {
                if (result.daily_note_date) goto(`/journal/${result.daily_note_date}`);
                searchStore.open = false;
              }}
            >
              <span class="result-content">{result.block.content}</span>
              {#if result.parent_content}
                <span class="result-parent">{result.parent_content}</span>
              {/if}
              <div class="result-meta">
                {#if result.daily_note_date}
                  <span class="result-date">{result.daily_note_date}</span>
                {/if}
                {#each result.block.tags as tag}
                  <TagPill name={tag} />
                {/each}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .search-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.2);
    display: flex;
    justify-content: center;
    padding-top: 15vh;
    z-index: 100;
  }
  .search-panel {
    background: var(--bg-surface);
    border-radius: 8px;
    box-shadow: var(--shadow-lg);
    width: 560px;
    max-height: 60vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  .search-input {
    border: none;
    outline: none;
    font: inherit;
    font-size: var(--text-lg);
    padding: var(--space-4) var(--space-6);
    border-bottom: 1px solid var(--border);
  }
  .search-results {
    overflow-y: auto;
    padding: var(--space-2) 0;
  }
  .search-result {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--space-2) var(--space-6);
    font: inherit;
  }
  .search-result:hover, .search-result.selected {
    background: var(--bg-hover);
  }
  .result-content {
    display: block;
    font-size: var(--text-sm);
  }
  .result-parent {
    display: block;
    font-size: var(--text-xs);
    color: var(--text-muted);
    margin-top: 2px;
  }
  .result-meta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: 4px;
  }
  .result-date {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }
</style>
```

- [ ] **Step 3: Wire search into layout**

Update `src/routes/+layout.svelte` to include the search overlay and listen for `/` key:

Add to the script:
```ts
import SearchOverlay from '$lib/components/SearchOverlay.svelte';
import { searchState } from '$lib/stores/search.svelte';

const searchStore = searchState();

function handleGlobalKeydown(e: KeyboardEvent) {
  if (e.key === '/' && !searchStore.open && !(e.target instanceof HTMLInputElement) && !(e.target instanceof HTMLTextAreaElement)) {
    e.preventDefault();
    searchStore.open = true;
  }
}
```

Add to the template:
```svelte
<svelte:window onkeydown={handleGlobalKeydown} />
<SearchOverlay />
```

- [ ] **Step 4: Verify frontend builds**

Run: `npm run build`
Expected: Build succeeds.

- [ ] **Step 5: Commit**

```bash
git add src/lib/stores/search.svelte.ts src/lib/components/SearchOverlay.svelte src/routes/+layout.svelte
git commit -m "feat: add search overlay with FTS5 and / hotkey"
```

---

## Chunk 4: Quick Capture, Mind Map, and Polish

### Task 15: Quick capture window

**Files:**
- Create: `src/routes/capture/+page.svelte`
- Create: `src-tauri/src/quick_capture.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Configure second window in tauri.conf.json**

Add a `"capture"` window to the `windows` array in `tauri.conf.json`:
```json
{
  "label": "capture",
  "url": "/capture",
  "title": "",
  "width": 600,
  "height": 80,
  "resizable": false,
  "decorations": false,
  "alwaysOnTop": true,
  "visible": false,
  "center": true
}
```

- [ ] **Step 2: Write quick_capture.rs**

Create `src-tauri/src/quick_capture.rs`:
```rust
use tauri::{AppHandle, Manager};

pub fn setup_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    app.plugin(tauri_plugin_global_shortcut::Builder::new().build())?;

    app.global_shortcut().on_shortcut("CmdOrCtrl+Shift+Space", |app, _event, _shortcut| {
        if let Some(window) = app.get_webview_window("capture") {
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    })?;

    Ok(())
}
```

Note: This requires adding `tauri-plugin-global-shortcut` to `Cargo.toml`:
```toml
tauri-plugin-global-shortcut = "2"
```

- [ ] **Step 3: Wire into main.rs setup**

Add to the `setup` closure:
```rust
quick_capture::setup_global_shortcut(app.handle())?;
```

- [ ] **Step 4: Write capture page**

Create `src/routes/capture/+page.svelte`:
```svelte
<script lang="ts">
  import { createBlock, getOrCreateDailyNote, createTag, addTagToBlock } from '$lib/api';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  let input = $state('');
  let inputEl: HTMLInputElement;

  $effect(() => {
    inputEl?.focus();
  });

  async function submit() {
    if (!input.trim()) return;

    let content = input.trim();
    let blockType = 'bullet';
    const tagNames: string[] = [];

    // Parse [] prefix for tasks
    if (content.startsWith('[] ')) {
      blockType = 'task';
      content = content.slice(3);
    }

    // Parse #tag tokens
    const tagRegex = /#([\w/]+)/g;
    let match;
    while ((match = tagRegex.exec(content)) !== null) {
      tagNames.push(match[1]);
    }
    content = content.replace(tagRegex, '').trim();

    const today = new Date().toISOString().split('T')[0];
    const note = await getOrCreateDailyNote(today);
    const block = await createBlock(note.id, null, content, blockType, Date.now());

    // Apply tags
    for (const name of tagNames) {
      const tag = await createTag(name);
      await addTagToBlock(block.id, tag.id);
    }

    // Emit event so main window can refresh
    const { emit } = await import('@tauri-apps/api/event');
    await emit('journal-refresh');

    input = '';
    await getCurrentWindow().hide();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      submit();
    }
    if (e.key === 'Escape') {
      input = '';
      getCurrentWindow().hide();
    }
  }
</script>

<div class="capture">
  <input
    bind:this={inputEl}
    bind:value={input}
    class="capture-input"
    placeholder="What's on your mind? ([] for task, #tag)"
    onkeydown={handleKeydown}
  />
  <div class="hint">
    <span>bullet</span>
    <span>[] task</span>
    <span>#tag</span>
    <span>Enter ↵</span>
  </div>
</div>

<style>
  .capture {
    padding: 16px;
    background: #fff;
    height: 100vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
  .capture-input {
    border: none;
    outline: none;
    font-size: 18px;
    font-family: var(--font-sans, -apple-system, sans-serif);
    width: 100%;
  }
  .hint {
    display: flex;
    gap: 12px;
    margin-top: 8px;
    font-size: 11px;
    color: #aaa;
  }
</style>
```

- [ ] **Step 5: Listen for journal-refresh event in main window**

Update `src/routes/journal/[date]/+page.svelte` to listen for the `journal-refresh` event from Tauri:
```ts
import { listen } from '@tauri-apps/api/event';
import { onMount } from 'svelte';

onMount(() => {
  const unlisten = listen('journal-refresh', () => {
    journal.refresh();
  });
  return () => { unlisten.then(fn => fn()); };
});
```

- [ ] **Step 6: Verify it compiles**

Run: `cd src-tauri && cargo check` and `npm run build`
Expected: No errors.

- [ ] **Step 7: Commit**

```bash
git add src/routes/capture/ src-tauri/src/quick_capture.rs src-tauri/Cargo.toml src-tauri/tauri.conf.json src-tauri/src/main.rs src/routes/journal/
git commit -m "feat: add quick capture window with global hotkey"
```

---

### Task 16: Mind map view

**Files:**
- Create: `src/routes/mindmap/[id]/+page.svelte`
- Create: `src/lib/components/MindMapCanvas.svelte`

- [ ] **Step 1: Write MindMapCanvas component**

Create `src/lib/components/MindMapCanvas.svelte`:

This component needs a canvas-like interaction. Use SVG for rendering nodes and connections, with drag support.

```svelte
<script lang="ts">
  import {
    getMindMapNodes,
    addMindMapNode,
    updateNodePosition,
    createLink,
    sendNodesToJournal,
    getForwardLinks,
  } from '$lib/api';
  import type { MindMapNode, Block, BlockLink } from '$lib/types';

  let { mindMapId }: { mindMapId: string } = $props();

  interface CanvasNode {
    node: MindMapNode;
    block: Block;
  }

  let nodes = $state<CanvasNode[]>([]);
  let connections = $state<{ from: string; to: string }[]>([]);
  let selected = $state<Set<string>>(new Set());
  let dragging = $state<string | null>(null);
  let dragOffset = $state({ x: 0, y: 0 });

  async function load() {
    const raw = await getMindMapNodes(mindMapId);
    // Fetch block content for each node via getBlocksForDate won't work here
    // since mind map blocks have no daily_note_id. We need a getBlock command,
    // or the backend should return nodes joined with their block data.
    // For now, map raw nodes and fetch block content inline:
    const loaded: CanvasNode[] = [];
    for (const n of raw) {
      // The backend get_mind_map_nodes should be extended to JOIN blocks
      // and return block content alongside node positions.
      loaded.push({ node: n, block: { id: n.block_id, content: '', block_type: 'bullet', parent_id: null, daily_note_id: null, position: 0, status: null, priority: null, due_date: null, created_at: '', updated_at: '', tags: [], children: [] } });
    }
    nodes = loaded;

    // Load connections: for all block_ids in this mind map, query forward links
    const allConns: { from: string; to: string }[] = [];
    const blockIds = new Set(nodes.map(n => n.node.block_id));
    for (const n of nodes) {
      const fwd = await getForwardLinks(n.node.block_id);
      for (const target of fwd) {
        if (blockIds.has(target.id)) {
          allConns.push({ from: n.node.block_id, to: target.id });
        }
      }
    }
    connections = allConns;
  }

  // Note: A production implementation should add a `get_mind_map_nodes_with_blocks`
  // backend command that JOINs blocks in a single query for efficiency.

  import { onMount } from 'svelte';
  onMount(() => { load(); });

  async function handleCanvasDblClick(e: MouseEvent) {
    const svg = e.currentTarget as SVGSVGElement;
    const rect = svg.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    await addMindMapNode(mindMapId, '', x, y);
    await load();
  }

  function handleNodeMouseDown(nodeId: string, e: MouseEvent) {
    dragging = nodeId;
    const node = nodes.find(n => n.node.id === nodeId);
    if (node) {
      dragOffset = { x: e.clientX - node.node.x, y: e.clientY - node.node.y };
    }
  }

  function handleMouseMove(e: MouseEvent) {
    if (dragging) {
      const node = nodes.find(n => n.node.id === dragging);
      if (node) {
        node.node.x = e.clientX - dragOffset.x;
        node.node.y = e.clientY - dragOffset.y;
      }
    }
  }

  async function handleMouseUp() {
    if (dragging) {
      const node = nodes.find(n => n.node.id === dragging);
      if (node) {
        await updateNodePosition(node.node.id, node.node.x, node.node.y);
      }
      dragging = null;
    }
  }

  function toggleSelect(nodeId: string) {
    if (selected.has(nodeId)) {
      selected.delete(nodeId);
    } else {
      selected.add(nodeId);
    }
    selected = new Set(selected); // trigger reactivity
  }

  async function sendSelected() {
    const blockIds = nodes
      .filter(n => selected.has(n.node.id))
      .map(n => n.node.block_id);
    const today = new Date().toISOString().split('T')[0];
    await sendNodesToJournal(blockIds, today);
    selected = new Set();
    await load();
  }
</script>

<div class="mind-map-toolbar">
  {#if selected.size > 0}
    <button onclick={sendSelected} class="toolbar-btn">
      Send {selected.size} to journal
    </button>
  {/if}
</div>

<svg
  class="mind-map-canvas"
  ondblclick={handleCanvasDblClick}
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
>
  <!-- Connections -->
  {#each connections as conn}
    {@const from = nodes.find(n => n.node.block_id === conn.from)}
    {@const to = nodes.find(n => n.node.block_id === conn.to)}
    {#if from && to}
      <line
        x1={from.node.x}
        y1={from.node.y}
        x2={to.node.x}
        y2={to.node.y}
        stroke="var(--border-strong)"
        stroke-width="1"
      />
    {/if}
  {/each}

  <!-- Nodes -->
  {#each nodes as n (n.node.id)}
    <g
      transform="translate({n.node.x}, {n.node.y})"
      onmousedown={(e) => handleNodeMouseDown(n.node.id, e)}
      onclick={() => toggleSelect(n.node.id)}
      class="mind-map-node"
      class:selected={selected.has(n.node.id)}
    >
      <rect
        x="-60" y="-20" width="120" height="40"
        rx="6" ry="6"
        fill="var(--bg-surface)"
        stroke={selected.has(n.node.id) ? 'var(--accent)' : 'var(--border)'}
        stroke-width={selected.has(n.node.id) ? 2 : 1}
      />
      <text
        text-anchor="middle"
        dominant-baseline="central"
        font-size="13"
        fill="var(--text-primary)"
      >
        {n.block?.content || ''}
      </text>
    </g>
  {/each}
</svg>

<style>
  .mind-map-toolbar {
    padding: var(--space-2) var(--space-4);
    display: flex;
    gap: var(--space-2);
    border-bottom: 1px solid var(--border);
  }
  .toolbar-btn {
    font-size: var(--text-sm);
    padding: var(--space-1) var(--space-3);
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-surface);
    cursor: pointer;
  }
  .toolbar-btn:hover {
    background: var(--bg-hover);
  }
  .mind-map-canvas {
    width: 100%;
    height: calc(100vh - 48px);
    cursor: crosshair;
  }
  .mind-map-node {
    cursor: grab;
  }
  .mind-map-node:active {
    cursor: grabbing;
  }
</style>
```

- [ ] **Step 2: Write mind map page route**

Create `src/routes/mindmap/[id]/+page.svelte`:
```svelte
<script lang="ts">
  import { page } from '$app/state';
  import MindMapCanvas from '$lib/components/MindMapCanvas.svelte';
</script>

<MindMapCanvas mindMapId={page.params.id} />
```

- [ ] **Step 3: Verify frontend builds**

Run: `npm run build`
Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add src/routes/mindmap/ src/lib/components/MindMapCanvas.svelte
git commit -m "feat: add mind map canvas view with drag, select, and send-to-journal"
```

---

### Task 17: Block editor enhancements (tag autocomplete, link search)

**Files:**
- Create: `src/lib/components/BlockEditor.svelte`
- Create: `src/lib/components/TagAutocomplete.svelte`
- Create: `src/lib/components/LinkSearch.svelte`
- Modify: `src/lib/components/BlockItem.svelte`

- [ ] **Step 1: Write TagAutocomplete component**

Create `src/lib/components/TagAutocomplete.svelte`:
```svelte
<script lang="ts">
  import { getAllTags, createTag } from '$lib/api';
  import type { Tag } from '$lib/types';

  let { query, onselect, onclose }: {
    query: string;
    onselect: (tag: Tag) => void;
    onclose: () => void;
  } = $props();

  let allTags = $state<Tag[]>([]);
  let selectedIndex = $state(0);

  const filtered = $derived(
    allTags.filter(t => t.name.toLowerCase().includes(query.toLowerCase())).slice(0, 8)
  );

  $effect(() => {
    getAllTags().then(t => allTags = t);
  });

  $effect(() => {
    selectedIndex = 0;
  });

  export function handleKeydown(e: KeyboardEvent): boolean {
    if (e.key === 'ArrowDown') { selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1); return true; }
    if (e.key === 'ArrowUp') { selectedIndex = Math.max(selectedIndex - 1, 0); return true; }
    if (e.key === 'Enter' && filtered.length > 0) {
      onselect(filtered[selectedIndex]);
      return true;
    }
    if (e.key === 'Escape') { onclose(); return true; }
    return false;
  }
</script>

<div class="autocomplete">
  {#each filtered as tag, i (tag.id)}
    <button
      class="autocomplete-item"
      class:selected={i === selectedIndex}
      onclick={() => onselect(tag)}
    >
      #{tag.name}
    </button>
  {/each}
  {#if filtered.length === 0 && query.length > 0}
    <button class="autocomplete-item create" onclick={async () => { const tag = await createTag(query); onselect(tag); }}>Create #{query}</button>
  {/if}
</div>

<style>
  .autocomplete {
    position: absolute;
    top: 100%;
    left: 0;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    box-shadow: var(--shadow-md);
    min-width: 200px;
    z-index: 50;
  }
  .autocomplete-item {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--space-2) var(--space-3);
    font: inherit;
    font-size: var(--text-sm);
  }
  .autocomplete-item:hover, .autocomplete-item.selected {
    background: var(--bg-hover);
  }
  .create {
    color: var(--text-muted);
    font-style: italic;
  }
</style>
```

- [ ] **Step 2: Write LinkSearch component**

Create `src/lib/components/LinkSearch.svelte`:
```svelte
<script lang="ts">
  import { search } from '$lib/api';
  import type { SearchResult } from '$lib/types';

  let { query, onselect, onclose }: {
    query: string;
    onselect: (blockId: string) => void;
    onclose: () => void;
  } = $props();

  let results = $state<SearchResult[]>([]);
  let selectedIndex = $state(0);

  $effect(() => {
    if (query.length >= 2) {
      search(query).then(r => { results = r.slice(0, 8); selectedIndex = 0; });
    } else {
      results = [];
    }
  });

  export function handleKeydown(e: KeyboardEvent): boolean {
    if (e.key === 'ArrowDown') { selectedIndex = Math.min(selectedIndex + 1, results.length - 1); return true; }
    if (e.key === 'ArrowUp') { selectedIndex = Math.max(selectedIndex - 1, 0); return true; }
    if (e.key === 'Enter' && results.length > 0) {
      onselect(results[selectedIndex].block.id);
      return true;
    }
    if (e.key === 'Escape') { onclose(); return true; }
    return false;
  }
</script>

<div class="link-search">
  {#each results as result, i (result.block.id)}
    <button
      class="link-item"
      class:selected={i === selectedIndex}
      onclick={() => onselect(result.block.id)}
    >
      <span class="link-content">{result.block.content}</span>
      {#if result.daily_note_date}
        <span class="link-date">{result.daily_note_date}</span>
      {/if}
    </button>
  {/each}
</div>

<style>
  .link-search {
    position: absolute;
    top: 100%;
    left: 0;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    box-shadow: var(--shadow-md);
    min-width: 300px;
    z-index: 50;
  }
  .link-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--space-2) var(--space-3);
    font: inherit;
    font-size: var(--text-sm);
    gap: var(--space-2);
  }
  .link-item:hover, .link-item.selected {
    background: var(--bg-hover);
  }
  .link-content { flex: 1; }
  .link-date {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }
</style>
```

- [ ] **Step 3: Write BlockEditor component**

Create `src/lib/components/BlockEditor.svelte` — a richer inline editor that detects `#` and `[[` triggers and shows the appropriate autocomplete/search dropdown.

```svelte
<script lang="ts">
  import TagAutocomplete from './TagAutocomplete.svelte';
  import LinkSearch from './LinkSearch.svelte';
  import { addTagToBlock, createTag, createLink } from '$lib/api';

  let { blockId, initialContent, oncommit, oncancel }: {
    blockId: string;
    initialContent: string;
    oncommit: (content: string) => void;
    oncancel: () => void;
  } = $props();

  let content = $state(initialContent);
  let mode = $state<'normal' | 'tag' | 'link'>('normal');
  let triggerQuery = $state('');
  let inputEl: HTMLInputElement;

  $effect(() => { inputEl?.focus(); });

  function handleInput(e: Event) {
    content = (e.target as HTMLInputElement).value;

    // Detect # trigger
    const hashMatch = content.match(/#([\w/]*)$/);
    if (hashMatch) {
      mode = 'tag';
      triggerQuery = hashMatch[1];
      return;
    }

    // Detect [[ trigger
    const linkMatch = content.match(/\[\[([^\]]*)$/);
    if (linkMatch) {
      mode = 'link';
      triggerQuery = linkMatch[1];
      return;
    }

    mode = 'normal';
    triggerQuery = '';
  }

  async function handleTagSelect(tag: { id: string; name: string }) {
    // Remove the #query from content
    content = content.replace(/#[\w/]*$/, '').trim();
    await addTagToBlock(blockId, tag.id);
    mode = 'normal';
    inputEl?.focus();
  }

  async function handleLinkSelect(targetBlockId: string) {
    // Remove the [[query from content
    content = content.replace(/\[\[[^\]]*$/, '').trim();
    await createLink(blockId, targetBlockId);
    mode = 'normal';
    inputEl?.focus();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey && mode === 'normal') {
      e.preventDefault();
      oncommit(content);
    }
    if (e.key === 'Escape' && mode === 'normal') {
      oncancel();
    }
  }
</script>

<div class="block-editor-wrapper">
  <input
    bind:this={inputEl}
    class="block-editor-input"
    value={content}
    oninput={handleInput}
    onkeydown={handleKeydown}
  />

  {#if mode === 'tag'}
    <TagAutocomplete
      query={triggerQuery}
      onselect={handleTagSelect}
      onclose={() => { mode = 'normal'; }}
    />
  {/if}

  {#if mode === 'link'}
    <LinkSearch
      query={triggerQuery}
      onselect={handleLinkSelect}
      onclose={() => { mode = 'normal'; }}
    />
  {/if}
</div>

<style>
  .block-editor-wrapper {
    position: relative;
    flex: 1;
  }
  .block-editor-input {
    width: 100%;
    border: none;
    outline: none;
    font: inherit;
    padding: var(--space-1);
    background: var(--bg-muted);
    border-radius: 2px;
  }
</style>
```

- [ ] **Step 4: Update BlockItem to use BlockEditor**

Modify `src/lib/components/BlockItem.svelte`: replace the inline `<input>` with `<BlockEditor>` when editing.

- [ ] **Step 5: Verify frontend builds**

Run: `npm run build`
Expected: Build succeeds.

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/BlockEditor.svelte src/lib/components/TagAutocomplete.svelte src/lib/components/LinkSearch.svelte src/lib/components/BlockItem.svelte
git commit -m "feat: add block editor with # tag autocomplete and [[ link search"
```

---

### Task 18: Keyboard shortcuts and journal editing interactions

**Files:**
- Modify: `src/routes/journal/[date]/+page.svelte`
- Modify: `src/lib/components/BlockItem.svelte`

- [ ] **Step 1: Add keyboard event handler to journal page**

Update `src/routes/journal/[date]/+page.svelte` to track the currently focused block index and dispatch keyboard actions. Add a `focusedIndex` state and pass an `onfocus` callback to each BlockItem:

```svelte
<script lang="ts">
  // ... existing imports ...
  import { createBlock, reparentBlock, reorderBlock, updateBlock } from '$lib/api';

  let focusedBlockId = $state<string | null>(null);

  async function handleBlockCommit(blockId: string, content: string) {
    await updateBlock(blockId, content);
    // Create a new sibling block below the committed one
    if (journal.dailyNote) {
      const block = findBlock(journal.blocks, blockId);
      const newPos = block ? block.position + 1 : 0;
      await journal.addBlock(block?.parent_id ?? null, '', 'bullet', newPos);
    }
    await journal.refresh();
  }

  function findBlock(blocks: Block[], id: string): Block | null {
    for (const b of blocks) {
      if (b.id === id) return b;
      const found = findBlock(b.children, id);
      if (found) return found;
    }
    return null;
  }

  function findParentBlock(blocks: Block[], childId: string, parent: Block | null = null): Block | null {
    for (const b of blocks) {
      if (b.id === childId) return parent;
      const found = findParentBlock(b.children, childId, b);
      if (found) return found;
    }
    return null;
  }

  function findPreviousSibling(blocks: Block[], blockId: string): Block | null {
    for (let i = 1; i < blocks.length; i++) {
      if (blocks[i].id === blockId) return blocks[i - 1];
    }
    for (const b of blocks) {
      const found = findPreviousSibling(b.children, blockId);
      if (found) return found;
    }
    return null;
  }
</script>
```

- [ ] **Step 2: Add Tab/Shift+Tab for indent/outdent**

In `BlockEditor.svelte`, add to `handleKeydown`:
```ts
if (e.key === 'Tab' && !e.shiftKey) {
  e.preventDefault();
  // Find previous sibling from parent context and nest under it
  onindent(blockId);
}
if (e.key === 'Tab' && e.shiftKey) {
  e.preventDefault();
  onoutdent(blockId);
}
```

Add `onindent` and `onoutdent` props to BlockEditor and BlockItem, wired to the journal page:
```ts
async function handleIndent(blockId: string) {
  const prev = findPreviousSibling(journal.blocks, blockId);
  if (prev) {
    await reparentBlock(blockId, prev.id, prev.children.length);
    await journal.refresh();
  }
}

async function handleOutdent(blockId: string) {
  const parent = findParentBlock(journal.blocks, blockId);
  if (parent) {
    const grandparent = findParentBlock(journal.blocks, parent.id);
    await reparentBlock(blockId, grandparent?.id ?? null, parent.position + 1);
    await journal.refresh();
  }
}
```

- [ ] **Step 3: Add Cmd+Enter to toggle task**

In `BlockEditor.svelte`, add to `handleKeydown`:
```ts
if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
  e.preventDefault();
  const isTask = block.block_type === 'task';
  await updateBlock(blockId, undefined, isTask ? 'bullet' : 'task', isTask ? undefined : 'todo');
  onedit(blockId);
}
```

- [ ] **Step 4: Add Alt+Up/Alt+Down to reorder**

In `BlockEditor.svelte`, add to `handleKeydown`:
```ts
if (e.key === 'ArrowUp' && e.altKey) {
  e.preventDefault();
  await reorderBlock(blockId, block.parent_id ?? null, Math.max(0, block.position - 1));
  onedit(blockId);
}
if (e.key === 'ArrowDown' && e.altKey) {
  e.preventDefault();
  await reorderBlock(blockId, block.parent_id ?? null, block.position + 1);
  onedit(blockId);
}
```

- [ ] **Step 5: Verify all keyboard shortcuts work**

Run the app with `npm run tauri dev` and manually test:
- Enter creates new block
- Tab indents, Shift+Tab outdents
- Cmd+Enter toggles task
- Alt+Up/Down reorders

- [ ] **Step 6: Commit**

```bash
git add src/routes/journal/ src/lib/components/
git commit -m "feat: add keyboard shortcuts for journal editing"
```

---

### Task 19: End-to-end integration test

**Files:**
- Run existing Rust tests + manual verification

- [ ] **Step 1: Run full Rust test suite**

Run: `cd src-tauri && cargo test`
Expected: All tests PASS

- [ ] **Step 2: Run frontend build**

Run: `npm run build`
Expected: No errors

- [ ] **Step 3: Run full Tauri build**

Run: `npm run tauri build`
Expected: Produces a working `.app` / `.dmg` on macOS

- [ ] **Step 4: Manual smoke test**

Launch the app and verify:
1. Journal view loads for today
2. Can add blocks by typing
3. Tab/Shift+Tab nests/unnests
4. Cmd+Enter toggles task
5. Tags work (type `#sometag`)
6. Search works (press `/`)
7. Quick capture works (Cmd+Shift+Space)
8. Tag pages aggregate correctly
9. Day navigation works
10. Sidebar shows tags and mind maps

- [ ] **Step 5: Commit any fixes**

```bash
git add -A
git commit -m "fix: integration test fixes"
```

---

### Task 20: Clean up old codebase

**Files:**
- Remove: `crates/` directory (old TUI code)
- Update: `Cargo.toml` (workspace root — remove old workspace members)
- Update: `README.md`

- [ ] **Step 1: Remove old crates directory**

```bash
rm -rf crates/
```

- [ ] **Step 2: Update root Cargo.toml**

Remove the `[workspace]` members pointing to `crates/sup` and `crates/sup-core`. The root `Cargo.toml` may no longer be needed if `src-tauri/Cargo.toml` is the only Rust project.

- [ ] **Step 3: Update .gitignore**

Add `.superpowers/` to `.gitignore` if not already present.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore: remove old TUI codebase, update project structure"
```
