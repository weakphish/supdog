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
