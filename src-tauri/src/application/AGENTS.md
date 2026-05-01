<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# src-tauri/src/application

## Purpose
Application layer containing business logic services for tasks, faith calculation, and ledger operations.

## Key Files
| File | Description |
|------|-------------|
| `mod.rs` | Module declarations |
| `task_service.rs` | Task business logic (create, complete, delete with faith bonuses) |
| `faith_service.rs` | Faith calculation (survival faith, progress faith, daily max 100) |
| `ledger_service.rs` | Ledger/日志 service for recording daily activities |

## For AI Agents

### Working In This Directory
- Services use domain models from `../domain/`
- Services interact with data layer via repository traits
- `faith_service.rs` computes faith from Work/Study task durations (2h per tier, max 40 each)
- Task bonus: Work/Study +5 faith/hr, Other +2 faith/hr

### Testing Requirements
- Unit tests in each module
- Use temporary SQLite db for integration tests

### Common Patterns
- `Result<T, ServiceError>` error handling
- Services are stateless (依赖注入 via repository)
- chrono for date/time arithmetic

## Dependencies

### Internal
- `../domain/` - Domain models (Task, Faith, Level)
- `../data/` - Repository traits and implementations

### External
- `chrono` 0.4 - Date/time
- `thiserror` 1 - Error handling

<!-- MANUAL: -->
