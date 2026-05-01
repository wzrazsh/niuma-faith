<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# frontend/src/types

## Purpose
TypeScript type definitions shared across the frontend application.

## Key Files
| File | Description |
|------|-------------|
| `index.ts` | Core type definitions (Task, TaskCategory, TaskStatus, FaithState, etc.) |
| `kanban.ts` | Kanban-specific types (KanbanColumn, KanbanCard, KanbanConfig) |

## For AI Agents

### Working In This Directory
- All types are exported from `index.ts` as barrel
- `kanban.ts` types are separate to keep kanban-specific interfaces organized
- Types must match Rust structs in `src-tauri/src/domain/`

### Testing Requirements
- Types verified by `vue-tsc --noEmit`

### Common Patterns
- Use `interface` for object shapes, `type` for unions/enums
- Nullable fields use `| null` syntax
- Date fields stored as ISO strings

## Dependencies

### Internal
- Types used by stores, components, and API layers

### External
- `typescript` 5.4 - Type system

<!-- MANUAL: -->
