# sup

A terminal knowledge base and engineering journal. Daily notes with an outline structure, first-class tasks, hierarchical tags, and fast search — backed by SQLite.

## Install

```bash
make install
# or
cargo install --path crates/sup
```

## Data

Your journal is stored at `~/.sup/sup.db` (SQLite). Back it up like any other file.

## CLI

```bash
# Log entries to today's journal
sup log "thinking about the auth refactor"
sup log --type h2 "Sprint planning"
sup log --tag projects/foo "tagged entry"

# Tasks
sup task add "implement oauth" --priority high --due 2026-03-15
sup task add "write tests" --tag projects/foo
sup task done <id>
sup task edit <id> --status in_progress

# View
sup today                         # today's outline
sup day 2026-03-01                # a past day
sup tasks                         # all open tasks
sup tasks --tag projects          # filtered (includes children)
sup search "oauth"                # full-text search

# Launch TUI
sup tui
```

## CLI — `--json` (agent/script friendly)

Every read command supports `--json` for machine-readable output:

```bash
sup today --json
sup tasks --json
sup tasks --tag projects --json
sup search "oauth" --json
sup task add "new task" --priority high --json
```

Example output:
```json
{
  "date": "2026-03-05",
  "nodes": [
    {
      "id": "abc123",
      "content": "implement oauth flow",
      "node_type": "Task",
      "status": "InProgress",
      "priority": "High",
      "tags": ["projects/foo"],
      "children": []
    }
  ]
}
```

## TUI Keybindings

### Views

| Key | Action |
|-----|--------|
| `1` | Journal view |
| `2` | Task view |
| `3` | Split view (journal + tasks) |
| `Tab` | Switch active pane (split view) |
| `q` | Quit |

### Journal Navigation

| Key | Action |
|-----|--------|
| `j` / `k` | Move cursor down / up |
| `[` / `]` | Previous / next day |

### Journal Editing

| Key | Action |
|-----|--------|
| `o` | Add node below cursor |
| `Enter` | Edit selected node |
| `d` | Delete selected node |
| `Tab` | Indent node (make child) |
| `Shift+Tab` | Unindent node (promote) |
| `t` | Edit tags on selected node |
| `L` | Link selected node to another |
| `/` | Search (full-text) |

### Editor (inline)

| Key | Action |
|-----|--------|
| `Enter` | Commit edit |
| `Esc` | Cancel (deletes node if empty) |
| `←` / `→` | Move cursor |
| `Backspace` | Delete character |

### Task View

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate tasks |
| `c` | Cycle status (todo→in_progress→done→cancelled) |

### Tag Editor

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate existing tags |
| `d` | Remove selected tag |
| `Enter` | Add typed tag |
| `Esc` | Close |

## Tags

Tags are hierarchical. `#projects/foo/bar` is a child of `#projects/foo`, which is a child of `#projects`. Filtering by `--tag projects` includes all descendants.

## Node Types

| Type | CLI `--type` | Appearance |
|------|-------------|-----------|
| Bullet | `bullet` (default) | `•` |
| Heading 1 | `h1` | `#` bold |
| Heading 2 | `h2` | `#` bold |
| Heading 3 | `h3` | `#` bold |
| Quote | `quote` | `"` italic gray |
| Code | `code` | `>` cyan |
| Task | (via `sup task add`) | `☐` / `◐` / `☑` / `✗` |
