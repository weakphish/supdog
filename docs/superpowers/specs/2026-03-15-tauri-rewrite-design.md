# supdog Tauri Rewrite — Design Spec

## Overview

Rewrite supdog from a Rust TUI application to a Tauri desktop app with a Svelte frontend. The core identity remains a **daily engineering journal** — the primary input surface is today's journal page. Organization emerges through use (tagging, linking) rather than upfront structure.

### Design Philosophy

- **Dump first, organize naturally.** The journal is where everything enters. Tags and links let you find things later without requiring thought at capture time.
- **Tags are for aggregation.** A tag page collects all blocks and tasks with that tag onto one page. "Show me everything about `#project/foo`."
- **Links are for association.** Explicit connections between a journal block and a task from a different day. "This specific block relates to this specific task."
- **Child blocks travel with parents.** Blocks nested under a task are its context — they follow it to tag pages, search results, and anywhere the task appears.

### Visual Aesthetic

Clean, editorial, typographic. Generous whitespace, minimal chrome. Content-first, not widget-first. Inspired by: Anytype's website, Semafor, component.gallery, herecomesthemoon.net, microsoft.ai.

---

## Data Model

### Block

The universal content unit. Everything is a block.

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Primary key |
| `content` | String | Text content |
| `block_type` | Enum | `bullet`, `h1`, `h2`, `h3`, `quote`, `code`, `task` |
| `parent_id` | UUID? | Parent block for arbitrary nesting (no depth limit) |
| `daily_note_id` | UUID? | Which day this block was created in (nullable for mind-map-originated blocks) |
| `position` | i64 | Ordering among siblings |
| `status` | Enum? | `todo`, `in_progress`, `done`, `cancelled` (only when block_type = task) |
| `priority` | Enum? | `high`, `med`, `low` (only when block_type = task) |
| `due_date` | Date? | Task deadline |
| `created_at` | DateTime | |
| `updated_at` | DateTime | |

### DailyNote

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Primary key |
| `date` | Date | Unique, one per day |

### Tag

Hierarchical. `project/migration` auto-creates `project` as parent.

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Primary key |
| `name` | String | Full path (e.g., `project/migration`) |
| `parent_id` | UUID? | Parent tag |

### BlockTag

Junction table: `block_id`, `tag_id`.

### BlockLink

Directional association between blocks.

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Primary key |
| `source_id` | UUID | The block that references |
| `target_id` | UUID | The block being referenced |
| `created_at` | DateTime | |

Unique constraint on `(source_id, target_id)`.

### FTS

SQLite FTS5 virtual table on block content, with auto-sync triggers on insert/update/delete.

---

## Architecture

```
supdog/
├── src-tauri/              # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs         # Tauri bootstrap
│   │   ├── db.rs           # SQLite connection, migrations
│   │   ├── models.rs       # Block, Tag, DailyNote, BlockLink
│   │   ├── commands/       # Tauri #[command] handlers
│   │   │   ├── blocks.rs
│   │   │   ├── daily_notes.rs
│   │   │   ├── tags.rs
│   │   │   ├── links.rs
│   │   │   └── search.rs
│   │   └── quick_capture.rs  # Quick-capture window management
│   └── Cargo.toml
├── src/                    # Svelte frontend
│   ├── lib/
│   │   ├── stores/         # Svelte stores (reactive state)
│   │   ├── components/     # UI components
│   │   └── types.ts        # TypeScript types mirroring Rust models
│   ├── routes/             # Views
│   │   ├── journal/        # Daily journal (default view)
│   │   ├── tag/            # Tag page (aggregation)
│   │   └── mindmap/        # Mind map capture mode
│   └── app.html
└── package.json
```

**Key decisions:**

- Full rewrite — no shared `sup-core` crate. The Tauri backend owns the data layer directly.
- No CLI. Quick capture window covers the fast-input use case.
- Svelte with SvelteKit (static adapter, no SSR).
- Two Tauri windows: main app window + quick-capture window (Spotlight-style), managed via Tauri's multi-window API with a global hotkey.
- SQLite file at platform-appropriate app data directory (via Tauri's path API).

---

## Views

### Journal (default view)

The front door. Lands on today's date.

- Date displayed as a large typographic header.
- Blocks render as an outline with subtle indentation for nesting — no tree lines, no icons. Just indentation and whitespace.
- Tasks are visually distinct but not loud: small checkbox, same typeface. Status via styling (strikethrough for done, muted for cancelled).
- Tags appear as small, quiet pills inline.
- Navigate between days via left/right arrows or a minimal date bar.
- Editing is inline: click a block to edit, enter to create a sibling below, tab/shift-tab to nest/unnest.

### Tag Page

Aggregation view. Accessed by clicking a tag pill or from the sidebar.

- Tag name as a bold header.
- **Open tasks section** at top — each task with its nested context blocks visible.
- **Blocks section** below — reverse chronological, date separators as subtle typographic dividers.
- Related/child tags shown as quiet links.
- Tasks are fully interactive on this page (cycle status, expand/collapse context).
- Clicking a block's date jumps to that day's journal with the block highlighted.

### Mind Map

Spatial capture mode for brainstorming.

- Full-canvas view. Clean nodes with good typography.
- Double-click empty space to create a node. Type immediately.
- Drag from node edge to connect nodes.
- Right-click or shortcut menu: convert to task, add tag, send to journal.
- Select multiple nodes and "send to journal" creates them as a nested outline under today's entry.
- Nodes can exist without a daily_note_id until sent to journal.

### Quick Capture

Spotlight/quake-style popup window.

- Summoned via global hotkey (works system-wide, even when app is in background).
- Floating, borderless input field.
- Type naturally. Prefix with `[] ` to make it a task, `#tag` to tag inline. Otherwise it's a bullet.
- Enter commits to today's journal. Escape dismisses.
- No chrome, no buttons — just the input and a subtle hint line.

---

## Interactions & Navigation

### Global

- **Keyboard-first.** Mouse works for everything, but power users shouldn't need it.
- **`/`** opens search anywhere — FTS5 live results, navigable with arrow keys. Results show block with parent context.
- **`Cmd+K`** (or configured hotkey) for quick capture when app is focused. Global hotkey works system-wide.
- **Sidebar** is minimal: Journal, Tags (expandable list), Mind Maps. Collapsible to icon-only.

### Journal Editing

- Click or `Enter` on a block to edit inline. `Escape` to finish.
- `Enter` at end of block creates a sibling below.
- `Tab` indents (nests under previous sibling). `Shift+Tab` outdents.
- `Cmd+Enter` toggles a block between bullet and task.
- Drag to reorder, or keyboard shortcuts for move up/down.
- `#` while editing triggers tag autocomplete.
- `[[` while editing triggers block/task search for linking.
- Deleting a parent block prompts: delete children or promote them.

### Linking

- Type `[[` in any block to search for a task or block to link to. Select it, link is created.
- On the linked target: a "Referenced by" section shows all source blocks, in chronological order with date context.
- On the source block: a subtle indicator shows it's linked. Click to jump to the target.

### Tag Page

- Click any tag pill to navigate to that tag's page.
- Click a block's date to jump to that journal day with the block highlighted.
- Tasks on the tag page are fully interactive.

### Mind Map

- Double-click canvas to create a node.
- Drag from node edge to connect.
- Right-click menu: convert to task, add tag, send to journal.
- Multi-select + "send to journal" creates nested outline under today.

---

## Search

- **SQLite FTS5** on block content. Sub-millisecond queries for a single-user local dataset.
- **`/`** opens a floating search bar (minimal, centered — similar aesthetic to quick capture).
- Results stream in as you type.
- Each result shows: block content, parent block (dimmed), date, tags.
- `Enter` on a result navigates to that block in its journal day, highlighted.
- Searches across: blocks, tasks, tag names.
- Filter chips: `type:task`, `tag:project/foo`, `status:open`, date ranges.

---

## Data Storage

- **Single SQLite file** at platform app data directory (via Tauri's path API).
- **Migrations** managed in Rust, run on app startup.
- **No sync in v1.** Local-only. Single-file design is friendly to future sync (Turso, Cloudflare D1, file-based sync).
- **Backup** via SQLite `.backup` API is a nice-to-have, not v1 scope.

---

## Out of Scope (v1)

- Sync / multi-device
- Collaborative editing
- Plugin system
- Mobile app
- Export formats
- Auto-backup
- CLI
