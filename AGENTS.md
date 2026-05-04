<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# docs

## Purpose
Design specifications, requirements documents, and task documentation for the 牛马信仰 project.

## Key Files
| File | Description |
|------|-------------|
| `vision.md` | Product vision and design principles |
| `design-doc.md` | System architecture and module design (single source of truth) |
| `data-model.md` | Complete database schema (5 tables, DDL, migrations) |
| `api-contract.md` | Frontend↔Backend API contracts (25 Tauri commands) |
| `workflows.md` | Core business processes (check-in, task lifecycle, etc.) |
| `ui-spec.md` | UI components and interaction specifications |
| `build-guide.md` | Build and restore guide |
| `decisions.md` | Architecture Decision Records (ADR-001 ~ ADR-010) |
| `ai-collaboration.md` | AI collaboration rules and red lines |
| `test-plan.md` | Testing plan and verification checklist |
| `牛马信仰-需求文档.md` | Chinese requirements document (authoritative) |

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `archive/` | Closed-out specs and one-off task lists kept for history |

## For AI Agents

### Working In This Directory
- Documents contain design context and requirements
- Closed-out implementation worklogs and completed designs live in `archive/` (read-only history)
- Chinese documents (`牛马信仰-需求文档.md`) are authoritative requirements

### Testing Requirements
- Test plan used for visual verification via Playwright

### Common Patterns
- Markdown format for all documents
- Screenshots in `test-output/` for visual verification

## Dependencies

### Internal
- Project root `AGENTS.md` - Overall project structure

### External
- None

<!-- MANUAL: -->
