# 牛马信仰 — Changelog

> 本文档记录设计和实现层面的重要变化，服务于下一轮 AI 协作快速恢复上下文。细节契约仍以 `api-contract.md`、`data-model.md`、`workflows.md`、`design-doc.md` 为准。

## 2026-05-08

### Documentation

- **合并 `ui-spec.md` 入 `design-doc.md`**: `ui-spec.md` 已归档至 `docs/archive/ui-spec-2026-05-08.md`，其 CSS 变量、任务状态色、布局常量等内容合并到 `design-doc.md` §8，消除两个文档间 CSS 变量不一致（`design-doc.md` 原有过时的 v1 调色板）。
- 更新 `AGENTS.md`、`docs/AGENTS.md`、`ai-collaboration.md`、`tasks.md`、`changelog.md`、`code-wiki.md` 中 11 处 `ui-spec.md` 引用，指向 `design-doc.md`。

## 2026-05-06

### Features

- **看板泳道分组**: 每列内按 Task.category 自动分组渲染（工作/学习/其他），仅显示非空泳道。
  - 新增 `SwimlaneGroup` 类型 (`frontend/src/types/kanban.ts`)
  - 新增 `columnSwimlanes()`, `columnCards()`, `addCard()`, `resetToDefault()` 方法 (`frontend/src/stores/kanban.ts`)
  - 新增 `taskMap` computed 属性供组件查询任务详情
  - 修复 `KanbanBoard.vue` onMounted 加载、`KanbanCard.vue` 拖拽数据/timer 引用、`KanbanCardForm.vue` Map 查找

## 2026-05-05

### Documentation

- Added `docs/AGENTS.md` as the documents directory index and AI reading guide.
- Added `roadmap.md` for phase-level priorities.
- Added `tasks.md` for current implementation state and active queue.
- Added this `changelog.md` to prevent meaningful design and implementation changes from being lost between AI sessions.
- Updated `ai-collaboration.md` so new AI sessions read the lightweight current-state documents before editing code.
- Added `docs/testing/` with strategy, test cases, acceptance criteria, fixtures, bug template, and regression checklist.
- Updated `tasks.md` so active tasks include acceptance criteria and linked test cases.
- Updated `ai-collaboration.md` with explicit AI testing rules for code changes and bug fixes.

## 2026-05-11

### Documentation

- **更新 `tasks.md`**: P0 任务全部归档至 `docs/archive/tasks-2026-05-08.md`，文档精简为只保留活跃工作项。
- **更新 `build-guide.md`**: 同步 `package.json` 依赖变更（`niuma-faith: file:` 替换 `@tauri-apps/plugin-store`，`@playwright/test` 替换 `playwright`）。
- **更新 `design-doc.md`**: 修正看板组件描述，标记计时器和进程绑定为框架已实现、功能待补充。
- **更新测试用例**: `kanban-process-binding.md` 补充 TC-KANBAN-003/004/005 的实际状态说明。

## Update Rules

- Record user-visible product changes, API changes, schema changes, workflow changes, and major documentation structure changes.
- Do not record every small refactor.
- If a change modifies a contract, update the contract document first and summarize the result here.
