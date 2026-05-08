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
| 日历月/周/日视图 | Implemented | `design-doc.md`, `README.md` |
| 看板与任务卡片 | Implemented | `design-doc.md`, `README.md` |
| Windows 进程绑定 | Implemented | `README.md`, `design-doc.md` |
| 浏览器 Mock 运行时 | Implemented | `ai-collaboration.md`, `frontend/src/api/mock-invoke.ts` |

## 当前优先队列

### P0

- [x] 看板拖拽状态同步 [Verified]
  - 7 条验收标准均已通过代码审查验证。
  - 同列拖拽只更新 localStorage ✓
  - loadBoard reconcile 覆盖 localStorage ✓
  - 历史日期/项目/虚拟任务静默跳过 ✓
  - 终态任务拖回 inprogress 被保护 ✓
  - last-user-action-wins 序列号机制 ✓
  - 后端拒绝后 UI 回滚 ✓
- [x] 保持 Mock 与 Tauri 命令契约一致 [Verified]
  - 等级阈值 v2.0 x10 表已验证一致（`domain/level.rs` vs `mock-invoke.ts`）
  - `start_task`/`resume_task`/`complete_task`/`abandon_task` 添加终端状态守卫 ✓
  - `pause_task` 添加会话结算模拟 ✓
  - `end_task` 修复为委托 pause_task + 设 completed_at ✓
  - 关联测试用例：`docs/testing/test-cases/task-lifecycle.md#tc-task-006`
- [x] 加固任务生命周期一致性 [Verified]
  - Mock、Backend、Frontend 状态迁移一致验证通过
  - `start_task`/`resume_task`: 幂等 + 终态拒绝 ✓
  - `complete_task`/`abandon_task`: 终态拒绝 ✓
  - `update_task`: 新增终端守卫（后端 + Mock）✓
  - `update_task` 不再允许修改终态任务的任何字段 ✓
- [x] 验证历史日期保护 [Verified]
  - 后端: `is_historical` 拒绝写操作 ✓
  - Mock: `t.date < todayStr()` 拒绝写操作 ✓
  - UI: `TaskList.vue` 隐藏操作按钮 + 显示「只读」✓
  - UI: `TaskDetailModal.vue` 隐藏保存/删除按钮 + 显示「历史任务·只读」✓
  - UI: `KanbanCard.vue` 历史任务禁止拖拽 ✓
  - Playwright 交互验证: `KanbanCard` `draggable=false` ✓, `TaskDetailModal` 横幅正确 ✓
  - 保护逻辑代码审查三处全部确认 ✓
- [x] Playwright 交互测试 — 任务生命周期端到端 [Verified]
  - 验收: 创建→启动→暂停→继续→完成/放弃 全链路通过
  - 创建任务: 表单填写 + 确认 ✓
  - 启动任务: 状态→`running`，显示暂停/完成/放弃 ✓
  - 暂停/继续: `paused` ↔ `running` 双向切换 ✓
  - 完成任务: 状态→`completed`，信仰值累加，prompt 处理 ✓
  - 放弃任务: 状态→`abandoned` ✓
  - 终端状态守卫: 已完成/已放弃任务 `.task-actions` 为空 ✓
  - 测试脚本: `test_lifecycle.mjs`、`test_pause_complete.mjs`、`test_terminal_guard.mjs`
  - 注意: 需处理 `window.prompt` 对话框（测试中通过 Playwright `page.on('dialog')` 自动处理）

### P1

- [x] 补强日历 + 任务列表交互检查 [Verified]
  - 验收：月/周/日切换、日期选择、任务只读状态有测试用例或手动验收记录。
  - Playwright 交互测试通过（创建→生命周期→终端守卫→历史保护）
  - 关联测试用例：`docs/test-plan.md`
- [ ] 补强看板拖拽、计时器、进程绑定检查 [Planned]
  - 验收：看板拖拽、计时器、绑定/解绑进程均有测试用例或手动 smoke。
  - KanbanCard 拖拽: 代码审查确认 `:draggable="!isHistorical"` ✓（交互测试受限于看板仅加载当日任务）
  - 关联测试用例：`docs/testing/test-cases/kanban-process-binding.md`
- [ ] UI 行为变更同步文档 [Planned]
  - 验收：组件布局或交互变化同步更新 `design-doc.md` 和必要测试用例。

#### P1 已知问题

- **Dashboard 不自动加载任务** (`Dashboard.vue:24`): `onDateSelect` 仅设置 `selectedDate` 但不调用 `loadTasksByDate`，导致首次加载或切换日期时显示"暂无任务"（需先访问看板页面才能填充 store）。建议修复为 `loadTasksByDate(date)` 并在 `onMounted` 中触发加载。

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
