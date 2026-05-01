<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# src-tauri/src/tauri

## Purpose
Tauri command handlers and application state management. Exposes Rust functions to the Vue frontend via `invoke`.

## Key Files
| File | Description |
|------|-------------|
| `mod.rs` | Module declarations |
| `commands.rs` | Tauri command definitions (task CRUD, process detection) |
| `state.rs` | Application state (database connection, etc.) |

## For AI Agents

### Working In This Directory
- Commands registered with Tauri builder in `main.rs`
- Process detection commands use `tauri-plugin-shell` to run `tasklist`
- All commands return `Result<T, Error>` serialized as JSON

### Testing Requirements
- Commands tested via integration tests with Tauri runtime

### Common Patterns
- `#[tauri::command]` attribute macro for commands
- `invoke<T>` from `@tauri-apps/api` on frontend
- State managed via `AppHandle` or `State<T>`

## Dependencies

### Internal
- `../application/` - Service layer
- `../data/` - Repository layer

### External
- `tauri` 2 - Command framework
- `tauri-plugin-shell` 2 - Process commands

<!-- MANUAL: -->
