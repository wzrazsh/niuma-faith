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

- [ ] 保持 Mock 与 Tauri 命令契约一致 [Planned]
  - 验收：`frontend/src/api/mock-invoke.ts` 的返回字段、错误语义与 `api-contract.md` 一致。
  - 验收：等级阈值使用 v2.0 x10 表。
  - 关联测试用例：`docs/testing/test-cases/task-lifecycle.md#tc-task-006-mock-与-tauri-任务契约一致`
  - 关联验收标准：`docs/testing/acceptance-criteria/task-lifecycle.md#ac-task-005-mock-一致性`
- [ ] 加固任务生命周期一致性 [Planned]
  - 验收：frontend、Tauri commands、application service、local HTTP server 的状态迁移一致。
  - 验收：终态任务不会重复开始、继续、完成或重复加分。
  - 关联测试用例：`docs/testing/test-cases/task-lifecycle.md`
  - 关联验收标准：`docs/testing/acceptance-criteria/task-lifecycle.md`
- [ ] 验证历史日期保护 [Planned]
  - 验收：历史日期任务在 UI 和后端均只读。
  - 验收：通过 IPC 或 Mock 直接写历史任务时也被拒绝。
  - 关联测试用例：`docs/testing/test-cases/task-lifecycle.md#tc-task-005-历史日期保护`
  - 关联验收标准：`docs/testing/acceptance-criteria/task-lifecycle.md#ac-task-004-历史保护`

### P1

- [ ] 补强日历 + 任务列表交互检查 [Planned]
  - 验收：月/周/日切换、日期选择、任务只读状态有测试用例或手动验收记录。
  - 关联测试用例：`docs/test-plan.md`
- [ ] 补强看板拖拽、计时器、进程绑定检查 [Planned]
  - 验收：看板拖拽、计时器、绑定/解绑进程均有测试用例或手动 smoke。
  - 关联测试用例：`docs/testing/test-cases/kanban-process-binding.md`
- [ ] UI 行为变更同步文档 [Planned]
  - 验收：组件布局或交互变化同步更新 `ui-spec.md` 和必要测试用例。

### P2

- [ ] 将重复手动验证收敛为 smoke 清单 [Implemented]
  - 验收：`docs/testing/regression-checklist.md` 可作为提交/发布前检查入口。
- [ ] 打包行为变化时更新发布和恢复步骤 [Planned]
  - 验收：`build-guide.md` 与实际命令一致。
- [ ] 已完成一次性任务归档 [Planned]
  - 验收：过时任务从本文档移入 `docs/archive/` 或总结到 `changelog.md`。

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
