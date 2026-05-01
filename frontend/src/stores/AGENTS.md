<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# frontend/src/stores

## Purpose
Pinia stores for managing application state: tasks, faith points, and kanban board.

## Key Files
| File | Description |
|------|-------------|
| `task.ts` | Task state store (tasks array, selectedDate, CRUD actions) |
| `faith.ts` | Faith state store (user faith points, level, statistics) |
| `kanban.ts` | Kanban state store (columns, cards, drag-drop state) |

## For AI Agents

### Working In This Directory
- Use `defineStore` from Pinia with composition API style
- `task.ts` manages `tasks` array and `selectedDate` for calendar
- `faith.ts` computes faith points from task completions
- `kanban.ts` manages kanban board state with columns and cards

### Testing Requirements
- State changes tested via Playwright screenshots
- No formal unit tests

### Common Patterns
- Stores use `ref`/`computed` from Vue composition API
- Actions dispatch Tauri commands via `invoke`
- State persisted to SQLite via Rust backend

## Dependencies

### Internal
- `../api/task.ts` - Tauri task command wrappers
- `../types/index.ts` - TypeScript interfaces

### External
- `pinia` 2.1 - State management

<!-- MANUAL: -->
