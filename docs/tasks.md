# 牛马信仰 — 当前任务与状态

> 本文档是 AI 协作时的当前状态板。它只记录短期有效的信息：正在做什么、下一步做什么、哪些能力只是计划、哪些已经实现或验证。

## 状态标签

| 标签 | 含义 |
|------|------|
| Planned | 已进入计划，但还未实现 |
| In Progress | 正在实现或本地有相关改动 |
| Implemented | 已实现，但仍需要验证或补文档 |
| Verified | 已实现并通过对应验证 |
| Rejected | 明确不做或已废弃 |

## 当前实现基线

| 能力 | 状态 | 事实来源 |
|------|------|----------|
| Tauri v2 桌面应用框架 | Implemented | `design-doc.md`, `README.md` |
| Vue 3 + TypeScript 前端 | Implemented | `design-doc.md`, `README.md` |
| SQLite 本地存储 | Implemented | `data-model.md`, `decisions.md` |
| 信仰积分与 15 级等级系统 | Implemented | `workflows.md`, `domain/level.rs` |
| 任务生命周期 | Implemented | `api-contract.md`, `workflows.md` |
| 历史日期保护 | Implemented | `workflows.md`, `ai-collaboration.md` |
| 日历月/周/日视图 | Implemented | `ui-spec.md`, `README.md` |
| 看板与任务卡片 | Implemented | `ui-spec.md`, `README.md` |
| Windows 进程绑定 | Implemented | `README.md`, `design-doc.md` |
| 浏览器 Mock 运行时 | Implemented | `ai-collaboration.md`, `frontend/src/api/mock-invoke.ts` |

## 当前优先队列

### P0

- Keep `mock-invoke.ts` aligned with `api-contract.md` whenever Tauri commands change.
- Keep task lifecycle behavior consistent across frontend, Tauri commands, application service, and local HTTP server.
- Verify historical-date protection after task lifecycle edits.

### P1

- Strengthen visual and workflow checks for calendar + task list interactions.
- Strengthen visual and workflow checks for Kanban drag/drop, timer, and process-binding behavior.
- Keep `ui-spec.md` synchronized when component layout or interaction behavior changes.

### P2

- Convert repeated manual verification into a smaller smoke checklist in `test-plan.md`.
- Document release and restore steps when packaging behavior changes.
- Move completed one-off task notes into `docs/archive/`.

## Planned But Not Implemented

- Cloud sync.
- Multi-user collaboration.
- Community/social features.
- Cross-device account system.

These remain out of scope until `vision.md`, `roadmap.md`, and the authoritative requirements document are updated.

## Rejected Or Deprecated

See `ai-collaboration.md` section 4 and `decisions.md`.

## Update Rules

- When code changes make a row inaccurate, update this file in the same change.
- Do not mark a task `Verified` without fresh test/build/manual verification evidence.
- Do not use this file as an archive. Move stale completed work to `docs/archive/` or summarize it in `changelog.md`.
