<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# frontend/src/services

## Purpose
Frontend services for process detection, kanban configuration persistence, and reminder scheduling.

## Key Files
| File | Description |
|------|-------------|
| `process-detector.ts` | Windows process detection via tasklist parsing |
| `kanban-api.ts` | Kanban column/card config persistence (localStorage) |
| `reminder-service.ts` | Reminder scheduling service |

## For AI Agents

### Working In This Directory
- `process-detector.ts` uses `tauri-plugin-shell` to run `tasklist` command
- `kanban-api.ts` uses localStorage for persisting kanban layout
- Reminder service integrates with Tauri notification system

### Testing Requirements
- Process detection only works on Windows
- localStorage persistence tested manually

### Common Patterns
- Services are plain TypeScript modules (no Vue composition)
- Use `invoke` from `@tauri-apps/api` for Rust communication

## Dependencies

### Internal
- `../api/tauri.ts` - Tauri invoke wrapper

### External
- `@tauri-apps/api` 2.0 - Tauri invoke
- `@tauri-apps/plugin-store` 2.0 - Persistent storage

<!-- MANUAL: -->
