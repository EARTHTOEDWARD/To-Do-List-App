# Graph-OS

> **Local-first visual To-Do & knowledge graph — syncs seamlessly through Supabase, scales from a single Mac to all your devices.**

Graph-OS links **tasks**, **LLM chats**, **code commits**, and **documents** into one typed graph. Each node is immutable; relationships are append-only edges. A CRDT op-log keeps every device consistent while letting you work entirely offline.

## Current Status

✅ **Sprint 0 & 1 Complete**: Basic scaffolding and hello-world local graph
- ✅ Rust workspace with `todo-core`, `todo-cli`, `todo-sync` crates
- ✅ SQLite database with append-only op-log 
- ✅ CLI for adding and listing tasks
- ✅ Tauri desktop app with React frontend
- ⏳ Desktop app dependencies (install with pnpm)

## Quick Start

```bash
# 1. Bootstrap the development environment
./scripts/bootstrap.sh

# 2. Initialize the todo system
cargo run -p todo-cli -- init

# 3. Add your first task
cargo run -p todo-cli -- add "Bootstrap repo skeleton"

# 4. List tasks
cargo run -p todo-cli -- ls

# 5. Start desktop app (after installing deps)
cd apps/desktop-tauri
pnpm install
pnpm tauri dev
```

## Architecture Overview

```
┌──── Desktop (Tauri) ──┐     real‑time CRDT ops     ┌──── Supabase Cloud ─┐
│ SQLite + op‑log (.jsonl)│  ←───────────────→        │ Postgres + Realtime │
└────────────────────────┘                           │  pgvector (or Qdrant)│
                                                    └──────────────────────┘
```

## Repository Structure

```
crates/        # Rust core, sync, CLI
├── todo-core/          # Rust lib: object CRUD, CRDT logic
├── todo-sync/          # Rust bin: pushes/pulls ops via Supabase (placeholder)
└── todo-cli/           # Rust bin: `todo add`, `todo ls`, etc.

apps/          # Tauri desktop, VS Code ext
├── desktop-tauri/      # React + Tauri desktop app
└── vscode-extension/   # VS Code extension (placeholder)

infra/         # Supabase migrations & scripts
├── supabase/           # SQL migrations, policies (to be added)
└── scripts/            # Bootstrap and utility scripts
```

## CLI Commands

```bash
# Add a new task
cargo run -p todo-cli -- add "Write documentation" --description "Create comprehensive README"

# List all tasks  
cargo run -p todo-cli -- ls

# List all tasks including completed
cargo run -p todo-cli -- ls --all

# Complete a task (by ID prefix or title match)
cargo run -p todo-cli -- complete abc123

# Show statistics
cargo run -p todo-cli -- stats

# Initialize database
cargo run -p todo-cli -- init
```

## Development

### Prerequisites

- Rust 1.78+
- Node.js 20+
- pnpm 9+

### Building

```bash
# Build all Rust crates
cargo build --all

# Install frontend dependencies
cd apps/desktop-tauri && pnpm install

# Run desktop app in development
pnpm tauri dev

# Run tests
cargo test --all
```

## Next Steps (Sprint 2+)

- [ ] **Cloud Sync**: Supabase integration with opt-in sync
- [ ] **Retention**: Archive old items with zstd compression
- [ ] **VS Code Extension**: Sidebar tree and capture selection
- [ ] **Graph Visualization**: Force-layout canvas with Pixi.js
- [ ] **Semantic Search**: Embedding and vector search
- [ ] **Chat Integration**: Import ChatGPT conversations
- [ ] **Git Integration**: Link commits to tasks

## Data Model

All data is stored as immutable objects with append-only edges in both SQLite (local) and operation logs (.jsonl files):

```rust
pub enum Kind { Task, Chat, Doc, Commit }

pub struct ObjectV1 {
    pub id: Ulid,
    pub kind: Kind,
    pub payload: serde_json::Value,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}

pub struct EdgeV1 {
    pub id: Ulid,
    pub from: Ulid,
    pub to: Ulid,
    pub typ: String,         // e.g. "CREATES", "SPAWNS", "FULFILLS"
    pub created: DateTime<Utc>,
}
```

## Contributing

This is an early-stage project implementing the Graph-OS architecture. The CRDT append-only design makes it easy to add new features without breaking existing functionality.

---

Happy hacking! 🎉
