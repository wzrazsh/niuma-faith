<!-- Parent: ../../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# frontend/src/components/kanban

## Purpose
Kanban board components featuring drag-and-drop task cards, process binding to Windows processes, and per-card timers.

## Key Files
| File | Description |
|------|-------------|
| `KanbanBoard.vue` | Main kanban container (process polling, drag-drop zones) |
| `KanbanColumn.vue` | Kanban column (To Do / In Progress / Paused / Done + custom columns) |
| `KanbanCard.vue` | Kanban card (timer, process binding, drag handle) |
| `KanbanCardForm.vue` | Form for creating/editing kanban cards |

## For AI Agents

### Working In This Directory
- KanbanBoard manages column state and drag-drop logic
- KanbanCard supports binding to Windows processes via `services/process-detector.ts`
- Timer runs on card when bound process is detected as running

### Testing Requirements
- Visual verification via screenshots in `test-output/`
- Process binding tested on Windows

### Common Patterns
- Uses `../stores/kanban.ts` for kanban state
- Uses `../../services/process-detector.ts` for process detection
- Drag-and-drop via Vue event handlers (no external library visible)

## Dependencies

### Internal
- `../../stores/kanban.ts` - Kanban Pinia store
- `../../services/process-detector.ts` - Windows process detection

### External
- `@tauri-apps/api` 2.0 - Tauri invoke commands

<!-- MANUAL: -->
