<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-05 -->

# docs

## Purpose

Design specifications, requirements documents, and task/state documentation for the 牛马信仰 project.

This directory is the context-control layer for AI-assisted development. Keep documents short enough to refresh often, current enough to trust, and explicit enough to prevent stale design from coming back.

## Key Files

| File | Description |
|------|-------------|
| `vision.md` | Product vision, target users, values, and non-goals |
| `roadmap.md` | Current phase, priorities, and milestone direction |
| `tasks.md` | Active work queue and current implementation state |
| `design-doc.md` | System architecture and module design; single source of truth for structure |
| `data-model.md` | Complete database schema, DDL, and migration rules |
| `api-contract.md` | Frontend/backend API contracts and Tauri commands |
| `workflows.md` | Core business processes, task lifecycle, check-in, and scoring flows |
| `ui-spec.md` | UI components and interaction specifications |
| `build-guide.md` | Build, restore, and local verification guide |
| `decisions.md` | Architecture Decision Records; accepted and rejected design choices |
| `ai-collaboration.md` | AI collaboration rules, red lines, and required reading order |
| `test-plan.md` | Testing plan and verification checklist |
| `changelog.md` | Human-readable design and implementation change log |
| `牛马信仰-需求文档.md` | Chinese requirements document; authoritative product requirements |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `archive/` | Closed-out specs and one-off task lists kept for history |

## For AI Agents

### Required Reading

Before changing code, read:

1. `docs/AGENTS.md`
2. `docs/vision.md`
3. `docs/ai-collaboration.md`
4. `docs/tasks.md`
5. The domain-specific document for the change:
   - API/IPC: `api-contract.md`
   - Database: `data-model.md`
   - UI: `ui-spec.md`
   - Business flow: `workflows.md`
   - Architecture: `design-doc.md`

### Working In This Directory

- Documents contain design context, requirements, and current state.
- Closed-out implementation worklogs and completed designs live in `archive/` as read-only history.
- Chinese documents are authoritative when product intent or naming is ambiguous.
- Update `tasks.md` when a planned capability moves from idea to implementation or from implementation to verified.
- Update `decisions.md` when a design choice would otherwise be re-litigated.
- Update `changelog.md` when a meaningful product, API, schema, or workflow change lands.

### Testing Requirements

- Use `test-plan.md` for verification scope.
- Use Playwright screenshots for visual verification when UI behavior or layout changes.
- For API, data, or workflow changes, pair implementation updates with the relevant contract document.

### Common Patterns

- Markdown format for all documents.
- Keep current-state files direct and editable; avoid large speculative sections.
- Prefer explicit "planned / implemented / verified / rejected" wording over vague descriptions.

## Dependencies

### Internal

- Project root `AGENTS.md` - Workspace-level agent guidance.
- `docs/ai-collaboration.md` - AI collaboration red lines and read order.

### External

- None.

<!-- MANUAL: -->
