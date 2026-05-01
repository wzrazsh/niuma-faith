<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# frontend/src/api

## Purpose
Tauri command wrappers for invoking Rust backend from Vue frontend.

## Key Files
| File | Description |
|------|-------------|
| `tauri.ts` | Core Tauri invoke wrapper |
| `task.ts` | Task CRUD commands (get_tasks, save_task, delete_task) |
| `mock-invoke.ts` | Mock invoke for testing without Tauri |

## For AI Agents

### Working In This Directory
- All backend commands accessed via `invoke` from `@tauri-apps/api`
- Task commands defined in Rust `src-tauri/src/tauri/commands.rs`
- Fallback to mock via `mock-invoke.ts` when Tauri not available

### Testing Requirements
- Mock invoke allows frontend testing without backend

### Common Patterns
- `invoke<T>('plugin:tauri|command_name', { args })` pattern
- Result type wrapped in Rust `Result<T, Error>`

## Dependencies

### Internal
- `src-tauri/src/tauri/commands.rs` - Backend command definitions

### External
- `@tauri-apps/api` 2.0 - Tauri invoke API

<!-- MANUAL: -->
