# 牛马信仰 (Niuma Faith)

A gamified productivity desktop app built with Tauri + Vue 3. Users complete tasks to accumulate faith points and level up through 15 ranks from 见习牛马 (Lv1) to 牛马圣徒 (Lv15).

## Features

- **单页应用** — 日历视图 + 任务管理融为一体，无需页面切换
- **任务积分** — 信仰积分从完成任务自动计算，无手动录入
- **日历模式** — 月/周/日三种视图，点击日期管理任务
- **历史保护** — 今日之前任务和积分锁定，不可修改
- **等级系统** — 15 级信仰等级，见习牛马 → 牛马圣徒

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Pinia + Vue Router + Vite
- **Backend**: Rust (Tauri v2) with SQLite (rusqlite)
- **Build**: `npm run build` runs both `vue-tsc --noEmit` (type check) and `vite build`

## Getting Started

```bash
# Install dependencies
npm install

# Start Vite dev server (frontend only)
npm run dev

# Type-check + production build
npm run build

# Start full Tauri app (dev)
npm run tauri dev

# Build Tauri executable
npm run tauri build
```

## Architecture

### Frontend (`frontend/src/`)

| Path | Purpose |
|------|---------|
| `components/Dashboard.vue` | 单页主布局：侧边日历 + 右侧任务区 |
| `components/CalendarView.vue` | 日历组件：月/周/日视图 |
| `components/TaskList.vue` | 任务列表（含只读模式） |
| `components/TaskForm.vue` | 任务创建/编辑表单 |
| `components/FaithDashboard.vue` | 当日信仰积分统计 |
| `stores/task.ts` | Pinia store：任务状态 + selectedDate |
| `stores/faith.ts` | Pinia store：用户信仰状态 |
| `api/task.ts` | Tauri 任务命令封装 |
| `types/index.ts` | TypeScript 类型定义 |

### Backend (`src-tauri/src/`)

| Layer | File | Purpose |
|-------|------|---------|
| **Domain** | `domain/task.rs` | Task 模型 + calc_task_bonus |
| **Domain** | `domain/faith.rs` | 信仰计算：survival/progress |
| **Domain** | `domain/level.rs` | 15 级等级系统 |
| **Application** | `application/task_service.rs` | 任务业务逻辑 |
| **Data** | `data/sqlite.rs` | SQLite 实现 |
| **Data** | `data/repository.rs` | Repo traits |
| **Tauri** | `main.rs` | 命令定义和注册 |

## 信仰计算

- **Survival faith** — 从 Work 任务实际时长计算（每 2h 一档，上限 40）
- **Progress faith** — 从 Study 任务实际时长计算（每 2h 一档，上限 40）
- **Daily max** — 100 pts
- **discipline_faith** — 已从任务系统移除

## 任务系统

- `TaskCategory`: Work / Study / Other
- `TaskStatus`: Active / Completed / Abandoned
- 完成任务获得 bonus faith（Work/Study +5/hr，Other +2/hr）
- 历史任务不可编辑/删除/完成
