# 牛马信仰 — Changelog

> 本文档记录设计和实现层面的重要变化，服务于下一轮 AI 协作快速恢复上下文。细节契约仍以 `api-contract.md`、`data-model.md`、`workflows.md`、`ui-spec.md` 为准。

## 2026-05-05

### Documentation

- Added `docs/AGENTS.md` as the documents directory index and AI reading guide.
- Added `roadmap.md` for phase-level priorities.
- Added `tasks.md` for current implementation state and active queue.
- Added this `changelog.md` to prevent meaningful design and implementation changes from being lost between AI sessions.
- Updated `ai-collaboration.md` so new AI sessions read the lightweight current-state documents before editing code.

## Update Rules

- Record user-visible product changes, API changes, schema changes, workflow changes, and major documentation structure changes.
- Do not record every small refactor.
- If a change modifies a contract, update the contract document first and summarize the result here.
