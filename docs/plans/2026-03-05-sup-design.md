# sup — Terminal Knowledge Base Design

**Date:** 2026-03-05
**Status:** Approved

## Overview

`sup` is a terminal knowledge base and engineering journal. It combines a fast CLI for on-the-fly logging with a rich TUI for viewing, searching, and editing. SQLite is the backend. The TEA (The Elm Architecture) pattern drives the TUI via ratatui. The CLI doubles as an agent-friendly interface via `--json` flags.

---

## Architecture

Cargo workspace with two crates:

```
sup/
├── Cargo.toml              # workspace
├── crates/
│   ├── sup-core/           # data model, SQLite schema, all queries
│   └── sup/                # CLI (clap) + TUI (ratatui) binary
└── docs/plans/
```

**`sup-core`** owns all domain types, the SQLite schema, migrations (`rusqlite` + `rusqlite_migration`), and the query layer. No raw SQL leaks into the binary crate.

**`sup` binary** owns CLI parsing and TUI rendering. TEA pattern: each TUI screen has its own `Model`, `Message`, and `update()` triad. `View` is a pure render from `Model`.

---

## Data Model

```sql
daily_notes
  id, date (unique)

nodes
  id
  parent_id       (nullable — root nodes have none)
  daily_note_id   (nullable — nodes created in a day context)
  content         (text)
  node_type       (bullet | h1 | h2 | h3 | quote | code | task)
  position        (int, sibling ordering)
  -- task-only fields, null for non-task types:
  status          (todo | in_progress | done | cancelled)
  priority        (high | med | low)
  due_date        (nullable date)
  created_at, updated_at

tags
  id, name (full path e.g. "projects/foo/bar"), parent_id (nullable)

node_tags         -- node ↔ tag join table
node_links        -- bidirectional links between any two nodes (task↔node or node↔node)
```

**Key decisions:**
- Tasks are `node_type = 'task'` — they live in the outline tree like any other node. The task view is a filtered query across all days.
- A task's "context" is its child nodes in the tree.
- Hierarchical tags stored as full paths with `parent_id`. Recursive CTEs power parent-encompasses-children queries.
- `node_links` enables bidirectional linking creatable from either side.
- Task carryover: on first `sup today` of a new day, incomplete tasks from prior days are linked into today's note (the task node itself doesn't move — a reference link is created).

---

## CLI Interface

```
# Daily workflow
sup log "thinking about the auth refactor"
sup log --type h2 "Sprint planning meeting"
sup log --tag projects/foo "fixed the bug"
sup task add "implement oauth flow" --priority high --due 2026-03-10
sup task add "write tests" --tag projects/foo

# Viewing
sup today                        # print today's outline
sup day 2026-03-01               # print a past day
sup tasks                        # list all open tasks
sup tasks --tag projects/foo     # filtered by tag (includes children)
sup search "auth refactor"       # surfaces matching nodes + children

# Task management
sup task done <id>
sup task edit <id> --status in_progress --priority low

# Agent-friendly — all read commands support --json
sup tasks --json
sup search "oauth" --json
sup today --json
```

Write commands (`sup log`, `sup task add`) return the created object as JSON when `--json` is passed. Errors go to stderr as `{ "error": "..." }`.

---

## TUI Interface

### Views

| Key | View |
|-----|------|
| `1` | Journal (outline for selected day) |
| `2` | Task view (all tasks, all time, grouped by status) |
| `3` | Split (journal left, tasks right) |

### Navigation & Editing (vim-style)

```
j/k          navigate nodes
h/l          collapse/expand children
[/]          previous/next day (journal view)
Tab          switch focus between panes (split view)

o            add sibling node below
O            add sibling node above
<enter>      edit node inline
<tab>        indent node (make child of node above)
<S-tab>      unindent node (promote to sibling of parent)
J            move node down (reorder siblings)
K            move node up

dd           delete node
t            add/edit tags on focused node
L            link node ↔ another node or task
c            cycle task status (todo→in_progress→done→cancelled)

/            search
:            command mode
```

In insert mode (`o`/`O`): `<enter>` creates sibling, `<tab>` at line start indents to child.

### Visual Details

- Left gutter: node type icon (bullet •, task ☐/☑, heading #, quote ", code `>`)
- Tags appear dimmed/ghost on the right of each node
- In split view, hovering a task highlights its linked nodes in the journal pane and vice versa

---

## Agentic Interface

`--json` on any read command returns structured, stable output. Example shapes:

```jsonc
// sup today --json
{
  "date": "2026-03-05",
  "nodes": [
    {
      "id": "abc123",
      "type": "task",
      "content": "implement oauth flow",
      "status": "in_progress",
      "priority": "high",
      "due_date": "2026-03-10",
      "tags": ["projects/foo"],
      "children": []
    }
  ]
}

// sup search "auth" --json
{
  "query": "auth",
  "results": [
    { "id": "...", "content": "...", "type": "bullet", "children": [], "day": "2026-03-05" }
  ]
}
```

---

## Stretch Goals (out of scope for v1)

- GitHub Issues integration (sync tasks ↔ issues)
- MCP server wrapping the CLI for richer agent tooling
