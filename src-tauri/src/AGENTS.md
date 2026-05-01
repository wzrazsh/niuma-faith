<!-- Parent: ../../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# src-tauri/src

## Purpose
Rust backend for 牛马信仰 (Niuma Faith). Implements domain models, business logic (faith calculation, task service), data persistence (SQLite), and Tauri command handlers.

## Key Files
| File | Description |
|------|-------------|
| `main.rs` | Application entry, command registration, process detection commands |
| `lib.rs` | Library entry point (used by `main.rs`) |

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `application/` | Business logic services (task_service, faith_service, ledger_service) |
| `data/` | Data access layer (SQLite implementation, repository traits) |
| `domain/` | Domain models (Task, Faith, Level) and business rules |
| `tauri/` | Tauri command handlers, application state |

## For AI Agents

### Working In This Directory
- Use `cargo build` for compilation, `cargo check` for type checking
- Run `npm run tauri dev` or `npm run tauri build` from project root
- Commands registered in `main.rs` via Tauri builder pattern
- Process detection commands: `is_process_running`, `list_processes`

### Testing Requirements
- Use `cargo test` for unit tests
- Integration tests use temporary SQLite databases

### Common Patterns
- `Result<T, Error>` pattern for fallible operations
- `thiserror` for error definitions
- `serde` derive macros for serialization
- `tracing` for structured logging

## Dependencies

### Internal
- `domain/` - Domain models and business rules
- `data/` - Repository traits and SQLite implementation
- `application/` - Service layer

### External
- `tauri` 2 - Desktop app framework
- `rusqlite` 0.31 - SQLite binding
- `chrono` 0.4 - Date/time handling
- `serde` 1.0 - Serialization
- `tracing` 0.1 - Logging

<!-- MANUAL: -->
