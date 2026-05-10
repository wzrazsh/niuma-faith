# 牛马信仰 — 当前任务与状态

> 本文档是 AI 协作时的当前状态板。它只记录短期有效的信息：正在做什么、下一步做什么、哪些能力只是计划、哪些已经实现或验证。
>
> **最后更新**: 2026-05-11

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
| 日历月/周/日视图 | Implemented | `design-doc.md`, `README.md` |
| 看板与任务卡片 | Implemented | `design-doc.md`, `README.md` |
| Windows 进程绑定 | Implemented | `README.md`, `design-doc.md` |
| 浏览器 Mock 运行时 | Implemented | `ai-collaboration.md`, `frontend/src/api/mock-invoke.ts` |

## 当前优先队列

### P0

P0 任务已全部完成并归档。详见 [`docs/archive/tasks-2026-05-08.md`](archive/tasks-2026-05-08.md)。

- [x] 看板拖拽状态同步 [Verified] — 2026-05-06
- [x] 保持 Mock 与 Tauri 命令契约一致 [Verified] — 2026-05-06
- [x] 加固任务生命周期一致性 [Verified] — 2026-05-05
- [x] 验证历史日期保护 [Verified] — 2026-05-05
- [x] Playwright 交互测试 — 任务生命周期端到端 [Verified] — 2026-05-05

### P1

- [x] 补强日历 + 任务列表交互检查 [Verified] — 2026-05-05
  - 关联测试用例: [`docs/test-plan.md`](test-plan.md)
- [ ] 补强看板拖拽、计时器、进程绑定检查 [Planned]
  - 关联测试用例: [`docs/testing/test-cases/kanban-process-binding.md`](testing/test-cases/kanban-process-binding.md)
- [ ] UI 行为变更同步文档 [Planned]

#### P1 已知问题（已修复）

- ~~Dashboard 不自动加载任务~~ **[已修复 2026-05-08]**
  - 修复: `Dashboard.vue` `onMounted` 中已添加 `task.loadTasksByDate()` 调用
  - 验证: 代码审查确认 `onDateSelect` 和 `onMounted` 均触发加载

### P2

- [x] 将重复手动验证收敛为 smoke 清单 [Verified] — 2026-05-05
  - 关联: [`docs/testing/regression-checklist.md`](testing/regression-checklist.md)
- [ ] 打包行为变化时更新发布和恢复步骤 [Planned]
  - 关联: [`docs/build-guide.md`](build-guide.md)
- [ ] 已完成一次性任务归档 [Planned]
  - 关联: [`docs/changelog.md`](changelog.md)

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
- Behavior-changing tasks must include acceptance criteria and a linked test case.
- Do not use this file as an archive. Move stale completed work to `docs/archive/` or summarize it in `changelog.md`.
