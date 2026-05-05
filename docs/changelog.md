# 牛马信仰 — Changelog

> 本文档记录设计和实现层面的重要变化，服务于下一轮 AI 协作快速恢复上下文。细节契约仍以 `api-contract.md`、`data-model.md`、`workflows.md`、`ui-spec.md` 为准。

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

## Update Rules

- Record user-visible product changes, API changes, schema changes, workflow changes, and major documentation structure changes.
- Do not record every small refactor.
- If a change modifies a contract, update the contract document first and summarize the result here.
