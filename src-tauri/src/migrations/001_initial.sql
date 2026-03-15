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

