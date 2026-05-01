<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# src-tauri/src/data

## Purpose
Data access layer implementing persistence for tasks and faith data using SQLite (rusqlite).

## Key Files
| File | Description |
|------|-------------|
| `mod.rs` | Module declarations |
| `sqlite.rs` | SQLite connection and migration management |
| `schema.rs` | Database schema definitions |
| `repository.rs` | Repository trait definitions |

## For AI Agents

### Working In This Directory
- `sqlite.rs` handles connection pooling and schema migrations
- `schema.rs` defines SQL for creating tables
- Repository traits define data access contracts implemented by services

### Testing Requirements
- Use `tempfile` crate for test databases
- Each test gets isolated database

### Common Patterns
- `rusqlite` with `bundled` feature for embedded SQLite
- Migrations run on database open
- Connection passed to repositories

## Dependencies

### Internal
- `../domain/` - Domain models persisted to DB

### External
- `rusqlite` 0.31 - SQLite binding
- `chrono` 0.4 - Date/time serialization

<!-- MANUAL: -->
