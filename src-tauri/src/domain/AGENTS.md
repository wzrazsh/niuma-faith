<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# src-tauri/src/domain

## Purpose
Domain models and business rules for 牛马信仰. Contains Task, Faith, and Level entities with their calculation logic.

## Key Files
| File | Description |
|------|-------------|
| `mod.rs` | Module declarations |
| `task.rs` | Task model (TaskCategory, TaskStatus) and `calc_task_bonus` |
| `faith.rs` | Faith calculation (survival_faith, progress_faith) |
| `level.rs` | 15-level faith rank system (见习牛马 Lv1 → 牛马圣徒 Lv15) |
| `models.rs` | Shared model types |

## For AI Agents

### Working In This Directory
- Domain models are plain Rust structs with `serde` derive
- Business rules implemented as methods/functions (e.g., `calc_task_bonus`)
- No direct database or I/O dependencies
- Faith calculation:
  - Survival faith from Work tasks: 2h per tier, max 40
  - Progress faith from Study tasks: 2h per tier, max 40
  - Daily max total: 100

### Testing Requirements
- Unit tests for calculation logic
- Property-based tests for edge cases

### Common Patterns
- `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]`
- `thiserror` for domain errors
- `chrono::DateTime` for timestamps

## Dependencies

### Internal
- None (pure domain layer)

### External
- `chrono` 0.4 - Timestamp types
- `serde` 1.0 - Serialization

<!-- MANUAL: -->
