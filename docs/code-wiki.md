# 牛马信仰 — Code Wiki 文档

> 本文档提供项目代码层面的完整导航，包括项目结构、架构层次、模块职责、关键类型与函数、依赖关系及运行方式。适用于开发者快速上手和深入理解代码。

---

## 1. 项目概述

**牛马信仰** 是一个桌面端生产⼒工具，通过"信仰—等级—护甲"游戏化系统激励用户专注⼯作和学习。用户每⽇打卡记录⼯作/学习时长，完成任务获得信仰值奖励，累计信仰提升等级（Lv1~Lv15）并获得护甲保护。

| 项目 | 描述 |
|------|------|
| 项目名称 | 牛马信仰 (NiuMa Faith) |
| 技术栈 | Tauri v2 + Rust 后端 + Vue 3 + TypeScript + SQLite |
| 平台 | Windows (进程检测仅⽀持 Windows) |
| 构建工具 | Vite 5.2 + Rust 2021 edition |
| 开发模式 | 浏览器 Mock 模式 / Tauri 桌面模式 |

---

## 2. 项目目录结构

```
niuma-faith/
├── index.html                      # HTML 入口
├── package.json                    # Node.js 依赖与脚本
├── vite.config.ts                  # Vite 构建配置
├── tsconfig.json                   # TypeScript 配置 (@ -> frontend/src)
├── frontend/                       # 前端代码
│   └── src/
│       ├── main.ts                 # 应用入口
│       ├── App.vue                 # 根组件
│       ├── router.ts               # Hash 路由
│       ├── style.css               # 全局暗色主题
│       ├── types/
│       │   ├── index.ts            # 核心业务类型
│       │   └── kanban.ts           # 看板类型
│       ├── api/
│       │   ├── mock-invoke.ts      # 双模式 IPC (Mock/Tauri)
│       │   ├── task.ts             # 任务命令封装
│       │   └── tauri.ts            # 信仰/用户命令封装
│       ├── stores/
│       │   ├── faith.ts            # 信仰状态管理 (Pinia)
│       │   ├── task.ts             # 任务状态管理 (Pinia)
│       │   └── kanban.ts           # 看板状态管理 (Pinia)
│       ├── components/
│       │   ├── Dashboard.vue       # 仪表盘主页
│       │   ├── CalendarView.vue    # 日历组件
│       │   ├── FaithDashboard.vue  # 今日信仰汇总
│       │   ├── StatusPanel.vue     # 等级状态面板
│       │   ├── DailyGoalPanel.vue  # 每日目标进度
│       │   ├── TaskList.vue        # 任务列表
│       │   ├── TaskForm.vue        # 任务创建/编辑弹窗
│       │   ├── KanbanPage.vue      # 看板页面容器
│       │   ├── FloatingWidget.vue  # 悬浮窗
│       │   └── kanban/
│       │       ├── KanbanBoard.vue       # 看板主组件
│       │       ├── KanbanColumn.vue      # 看板列
│       │       ├── KanbanCard.vue        # 看板卡片
│       │       └── KanbanCardForm.vue    # 看板任务编辑表单
│       ├── services/
│       │   ├── kanban-api.ts       # 看板配置持久化
│       │   ├── process-detector.ts # 进程检测 (3s 轮询)
│       │   └── reminder-service.ts # 任务提醒 (1min)
│       └── utils/
│           └── format.ts           # 数字格式化
├── src-tauri/                      # Rust 后端
│   ├── Cargo.toml                  # Rust 依赖
│   ├── tauri.conf.json             # Tauri 窗口配置
│   └── src/
│       ├── main.rs                 # 应用入口 + Tauri Builder
│       ├── lib.rs                  # 库模块导出
│       ├── domain/                 # 领域层 (纯逻辑)
│       │   ├── mod.rs
│       │   ├── models.rs           # 核心数据模型
│       │   ├── faith.rs            # 信仰计算函数
│       │   ├── level.rs            # 等级阈值表
│       │   └── task.rs             # 任务模型与奖励计算
│       ├── data/                   # 数据持久化层
│       │   ├── mod.rs
│       │   ├── schema.rs           # SQLite DDL + 迁移
│       │   ├── repository.rs       # Repository Trait
│       │   └── sqlite.rs           # rusqlite 实现
│       ├── application/            # 业务编排层
│       │   ├── mod.rs
│       │   ├── faith_service.rs    # 信仰服务
│       │   ├── ledger_service.rs   # 记账服务
│       │   └── task_service.rs     # 任务服务
│       ├── tauri/                  # Tauri 适配层
│       │   ├── mod.rs
│       │   ├── state.rs            # AppState
│       │   └── commands.rs         # 25 个 #[tauri::command]
│       └── local_server.rs         # 本地 HTTP API
└── docs/                           # 设计文档
    ├── design-doc.md
    ├── api-contract.md
    ├── data-model.md
    ├── workflows.md
    └── ... (其他文档)
```

---

## 3. 系统架构

### 3.1 三层架构 (Rust 后端)

```
┌──────────────────────────────────────────────┐
│  Tauri 适配层 (tauri/)                       │
│  • state.rs: AppState 管理                  │
│  • commands.rs: 25 个 #[tauri::command]      │
├──────────────────────────────────────────────┤
│  应用服务层 (application/)                   │
│  • faith_service.rs: 信仰/打卡/状态          │
│  • ledger_service.rs: 记账/信仰更新          │
│  • task_service.rs: 任务生命周期             │
├──────────────────────────────────────────────┤
│  数据持久化层 (data/)                        │
│  • schema.rs: DDL + 增量迁移                │
│  • repository.rs: Trait 定义                │
│  • sqlite.rs: rusqlite 实现 (5 表)           │
├──────────────────────────────────────────────┤
│  领域层 (domain/) — 纯函数，零外部依赖        │
│  • models.rs: 领域模型                       │
│  • faith.rs: 信仰计算                        │
│  • level.rs: 等级阈值                        │
│  • task.rs: 任务枚举+奖励                    │
└──────────────────────────────────────────────┘
```

### 3.2 前端架构

```
Vue Component (Dashboard / KanbanPage / FloatingWidget)
    ↓
Pinia Store (faith / task / kanban)
    ↓
API Layer (mock-invoke.ts → safeInvoke)
    ├── Tauri 环境 → @tauri-apps/api/core invoke
    └── 浏览器环境 → localStorage Mock (完整业务逻辑)
    ↓
Tauri IPC (JSON) / Browser Mock
    ↓
Rust Backend Commands
```

### 3.3 数据流示例：任务计时

```
前端点击"开始" → safeInvoke("start_task", { id })
  → TaskService::start_task()
    ├─ materialize_if_virtual() — 虚拟任务物化
    ├─ tasks.status = Running, started_at = now
    └─ TaskSessionRepo::start_session() — 插入新 session
前端点击"暂停" → safeInvoke("pause_task", { id })
  → TaskService::pause_task()
    ├─ TaskSessionRepo::end_open_session() — 计算秒数
    ├─ tasks.duration_seconds += seconds
    ├─ 累加到 daily_records.work/study_minutes
    ├─ LedgerService 重新计算当日信仰
    └─ tasks.status = Paused
```

---

## 4. 后端模块详解

### 4.1 领域层 (`src-tauri/src/domain/`)

#### `models.rs` — 核心数据模型

| 结构体 | 字段 | 说明 |
|--------|------|------|
| `User` | id, nickname, cumulative_faith, current_level, armor_points, created_at, updated_at | 用户实体 |
| `FaithStatus` | user_id, cumulative_faith, current_level, level_title, progress_to_next, next_threshold, today, armor, total_armor | 前端获取的完整状态 |
| `DailyRecord` | 20+ 字段 (见 data-model.md) | 每日打卡记录 |
| `FaithTransaction` | id, user_id, ts, delta, armor_delta, kind, ref_id, message | 信仰变动流水 |
| `ProcessInfo` | pid, name, status | 进程信息 |

#### `faith.rs` — 信仰计算纯函数

| 函数 | 签名 | 说明 |
|------|------|------|
| `calc_survival` | `(minutes: i32) -> i32` | 生存信仰：120/240/360/480 分钟阶梯 → 0/100/200/300/400 |
| `calc_progress` | `(minutes: i32) -> i32` | 精进信仰：同上 |
| `calc_discipline` | `(input: DisciplineInput) -> (i32, i32, i32, i32)` | 戒律信仰：专注(80/40/0) + 离岗(60/30/0) + 闭环(60/0) |
| `calculate_daily` | `(work, study, discipline) -> FaithBreakdown` | 组装日信仰三部分 |
| `FaithBreakdown` | survival_faith, progress_faith, discipline_faith, total_faith | 信仰明细 |

**戒律计算逻辑**:
```
a (专注稳定) = break_count ≤2 → 80, ≤4 → 40, ≥5 → 0
b (离岗纪律) = leave_record ==0 → 60, ==1 → 30, 其他 → 0
c (记录闭环) = close_record ≥1 → 60, 其他 → 0
discipline_faith = a + b + c (0~200)
```

#### `level.rs` — 等级系统

15 级阈值表 (v2.0 版，全部 ×10):

| 等级 | 阈值信仰 | 称号 |
|------|----------|------|
| Lv1 | 0 | 见习牛马 |
| Lv2 | 15,000 | 工位信徒 |
| Lv3 | 40,000 | 初级供奉者 |
| Lv4 | 80,000 | 稳定产出者 |
| Lv5 | 135,000 | 自律门徒 |
| Lv6 | 205,000 | 双修学徒 |
| Lv7 | 290,000 | 工时祭司 |
| Lv8 | 395,000 | 苦修执行官 |
| Lv9 | 520,000 | 连轴修行者 |
| Lv10 | 665,000 | 钢铁牛马 |
| Lv11 | 825,000 | 卷力使徒 |
| Lv12 | 945,000 | 精进主教 |
| Lv13 | 1,025,000 | 福报传道者 |
| Lv14 | 1,070,000 | 31日苦修士 |
| Lv15 | 1,095,000 | 牛马圣徒 |

**关键函数**:
- `get_level(cumulative_faith) -> Level` — 从后向前扫描阈值表
- `progress_to_next(cumulative_faith) -> Option<i64>` — 距下一级差值
- `interval_to_next(cumulative_faith) -> Option<i64>` — 当前级距
- `calc_armor(current_level) -> i32` — 护甲值：Lv2~Lv5=2000, Lv6~Lv10=4000, Lv11~Lv15=6000, Lv1=0

#### `task.rs` — 任务模型

| 枚举 | 值 |
|------|----|
| `TaskCategory` | Work / Study / Other (序列化为小写) |
| `TaskStatus` | Running / Paused / Completed / Abandoned |
| `RecurrenceKind` | None / Daily |
| `TaskType` | Daily / Project |

**奖励函数**: `calc_task_bonus(category, actual_minutes) -> i32`
```
rate = Work/Study → 5 信仰/小时, Other → 2 信仰/小时
hours = max(1, actual_minutes / 60)  // 整数除法，最少 1 小时
return hours * rate
```

---

### 4.2 数据持久化层 (`src-tauri/src/data/`)

#### `schema.rs` — 数据库 Schema

5 张表：`users`, `daily_records`, `tasks`, `task_sessions`, `faith_transactions`

**初始化流程**:
1. `PRAGMA journal_mode=WAL`
2. `PRAGMA foreign_keys=ON`
3. 执行 CREATE TABLE IF NOT EXISTS (5 表)
4. 创建基础索引
5. 执行 `ensure_column()` 增量迁移 (添加新列 + 对应索引)

**增量迁移函数**: `ensure_column(conn, table, column, col_type)` — 通过 PRAGMA table_info 检查列是否存在，不存在则 ALTER TABLE ADD COLUMN

#### `repository.rs` — Repository Trait 定义

| Trait | 关键方法 |
|-------|----------|
| `UserRepo` | find, upsert, add_faith(delta → 同时更新 cumulative_faith 和 current_level) |
| `DailyRecordRepo` | find_by_user_date, upsert (ON CONFLICT DO UPDATE), find_by_user_date_range |
| `TaskRepo` | insert, find_by_id, find_by_user (支持 status 过滤), update, delete, find_templates, find_instances |
| `FaithTransactionRepo` | insert, find_by_user_since |
| `TaskSessionRepo` | start_session, end_open_session, find_open_session |

#### `sqlite.rs` — rusqlite 实现 (约 996 行)

`SqliteDb` 结构体：封装 `Mutex<rusqlite::Connection>`

**数据库路径策略**:
```
优先: dirs::data_local_dir()/牛马信仰/niuma_faith.db
回退: exe 所在目录/niuma_faith.db
```

---

### 4.3 应用服务层 (`src-tauri/src/application/`)

#### `faith_service.rs` — 信仰服务

| 方法 | 说明 |
|------|------|
| `get_status(user_id) -> FaithStatus` | 查询用户状态 + 今日记录，组装完整 FaithStatus |
| `check_in(user_id, work_minutes, study_minutes, discipline_input) -> FaithStatus` | 打卡：计算信仰 → 调用 LedgerService → 返回新状态 |
| `get_today_record(user_id) -> Option<DailyRecord>` | 仅查询今日记录 |
| `get_or_create_user() -> User` | 返回默认用户，不存在则创建 |

#### `ledger_service.rs` — 记账服务 (核心)

`upsert_daily_record(user_id, date, work_minutes, study_minutes, discipline, breakdown, now_ts)`:

1. 查询旧的 daily_record（若存在）
2. 计算 `new_total = survival + progress + discipline + task_bonus_work + task_bonus_study`
3. `delta = new_total - old_total`（旧记录不存在时 delta = new_total）
4. `INSERT ... ON CONFLICT(user_id, date) DO UPDATE` 写入 daily_records
5. `UserRepo::add_faith(user_id, delta)` — 更新累计信仰 + 自动重算等级
6. 插入 faith_transactions 流水记录

#### `task_service.rs` — 任务服务 (~1150 行)

完整任务生命周期管理：

| 方法 | 说明 |
|------|------|
| `create_task` | 创建任务，默认 Paused 状态 |
| `start_task` | 开始计时 → 物化虚拟任务 → 开 session |
| `pause_task` | 暂停 → 关 session → 累加时长到 daily_records → 重算信仰 |
| `resume_task` | 恢复 → 开新 session |
| `complete_task` | 完成 → calc_task_bonus → apply_task_bonus → 更新状态 |
| `abandon_task` | 放弃（无奖励） |
| `delete_task` | 删除（级联删除模板实例） |
| `get_tasks_by_date` | 查询 + 合成虚拟实例 |
| `set_task_recurrence` | 设置/取消每日重复 |
| `get_project_task` | 按 tool_session_id 查询项目任务 |
| `get_project_tasks` | 查询活跃项目任务 |
| `create_project_task` | 创建项目任务（工具推送） |

**虚拟实例机制**:
```
模板任务: recurrence_kind='daily', template_id IS NULL
实例任务: template_id = {模板id}, recurrence_kind='none'
虚拟实例: 查询时内存合成 daily:{template_id}:{date}
虚拟 ID 格式: "daily:{template_uuid}:{2026-05-05}"
首次操作(如 start_task)时物化为真实行
```

**历史日期保护**: `is_historical(date)` 阻止对过去日期的任务修改/完成/放弃/删除。

---

### 4.4 Tauri 适配层 (`src-tauri/src/tauri/`)

#### `state.rs` — AppState

```rust
pub struct AppState {
    pub faith_service: FaithService,
    pub task_service: TaskService,
}
```

`AppState::new(db)` 初始化流程:
1. 创建 `FaithService` → 内含 `LedgerService`
2. 创建 `TaskService`
3. 调用 `get_or_create_user()` 确保默认用户存在

#### `commands.rs` — 25 个 Tauri 命令

| 分组 | 命令 |
|------|------|
| 信仰 | get_status, check_in, get_today_record, get_or_create_user |
| 进程 | is_process_running, list_processes |
| 任务 | create_task, get_tasks_by_date, get_tasks, get_task, update_task, complete_task, abandon_task, delete_task, start_task, pause_task, resume_task, end_task, set_task_recurrence, get_project_task, get_project_tasks |
| 统计 | get_daily_stats |
| 窗口 | open_floating_widget, close_floating_widget, show_main_window |

#### `main.rs` — 应用入口

启动流程:
```
1. 初始化 tracing 日志
2. 确定数据库路径 → SqliteDb::open()
3. 创建 AppState
4. 配置系统托盘 (左键显示窗口, 右键菜单)
5. 启动本地 HTTP Server (后台线程)
6. 注册所有 #[tauri::command]
7. 构建并运行 Tauri App
```

### 4.5 本地 HTTP Server (`src-tauri/src/local_server.rs`)

开发工具推送接口，使用 `tiny_http` crate，仅监听 127.0.0.1。

**端口/Token 管理**:
- 端口写入 `{data_dir}/牛马信仰/http_port.txt`
- Token 写入 `{data_dir}/牛马信仰/http_token.txt`
- 每次启动刷新，退出时删除

**端点路由**:
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/health` | 健康检查 |
| POST | `/api/tasks` | 创建项目任务 |
| PUT | `/api/tasks/{session_id}` | 更新任务状态 |
| POST | `/api/tasks/{session_id}/complete` | 完成任务 + 结算 |
| POST | `/api/tasks/{session_id}/abandon` | 放弃任务 |
| GET | `/api/tasks/{session_id}` | 查询任务状态 |

**认证**: Header `Authorization: Bearer {token}`

---

## 5. 前端模块详解

### 5.1 路由 (`router.ts`)

Hash 模式，3 条路由:

| 路径 | 组件 | 说明 |
|------|------|------|
| `/` | Dashboard | 仪表盘主页 |
| `/kanban` | KanbanPage | 看板页 |
| `/floating` | FloatingWidget | 悬浮窗 (80×80 透明窗口) |

其余路径重定向到 `/`。

### 5.2 全局样式 (`style.css`)

暗色主题 CSS 变量系统:

```css
--color-bg: #1a1a24;           /* 深色背景 */
--color-surface: #222233;       /* 卡片表面 */
--color-primary: #ffd700;       /* 金色 - 主色调 */
--color-success: #4ade80;       /* 绿色 */
--color-danger: #ef4444;        /* 红色 */
```

### 5.3 Pinia Stores

#### `faith.ts` — 信仰 Store

| 字段/方法 | 类型 | 说明 |
|-----------|------|------|
| `faithStatus` | `FaithStatus \| null` | 完整信仰状态 |
| `user` | `User \| null` | 用户信息 |
| `todayRecord` | `DailyRecord \| null` | 今日打卡记录 |
| `loading` | `boolean` | 加载状态 |
| `init()` | async | 初始化：get_or_create_user → get_status |
| `checkIn(...)` | async | 执行打卡 |
| `refreshStatus()` | async | 重新获取状态 |

#### `task.ts` — 任务 Store

| 字段/方法 | 类型 | 说明 |
|-----------|------|------|
| `tasks` | `Task[]` | 任务列表 |
| `dailyStats` | `DailyStats \| null` | 日统计 |
| `filter` | `TaskStatus \| 'all'` | 筛选状态 |
| `filteredTasks` | computed | 筛选后的任务列表 |
| `loadTasks()` / `loadTasksByDate(date)` | async | 加载任务 |
| `createTask(...)` | async | 创建 |
| `completeTask(id, actualMinutes)` | async | 完成 |
| `startTask / pauseTask / resumeTask / endTask` | async | 计时 |
| `abandonTask / deleteTask` | async | 放弃/删除 |
| `setTaskRecurrence(id, kind)` | async | 设置重复 |

#### `kanban.ts` — 看板 Store

| 字段/方法 | 类型 | 说明 |
|-----------|------|------|
| `columns` | `KanbanColumn[]` | 看板列列表 |
| `cards` | `Map<string, KanbanCard>` | 看板卡片 Map |
| `activeTimers` | `Map<string, number>` | 前端计时器 Map |
| `isLoading` | `boolean` | 加载状态 |
| `loadBoard()` | async | 从 localStorage 读取列配置 + 从 task store 加载任务 → 合成 cards |
| `moveCard(cardId, targetColumnId, targetIndex)` | async | 拖拽移动卡片并持久化 |
| `addColumn(title)` / `removeColumn(id)` | async | 列管理 |
| `startTimer(cardId)` / `stopTimer(cardId)` | async | 前端实时计时 |

看板默认四列: **待办 → 进行中 → 暂停中 → 已完成**

### 5.4 双模式 IPC (`api/mock-invoke.ts`)

核心函数 `safeInvoke<T>(command, args)`:

```typescript
async function safeInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if ((window as any).__TAURI_INTERNALS__) {
    // Tauri 桌面模式 → 动态 import @tauri-apps/api/core invoke
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(command, args);
  } else {
    // 浏览器开发模式 → localStorage Mock handlers
    const handler = handlers[command];
    if (!handler) throw new Error(`Mock: unknown command ${command}`);
    return handler(args) as T;
  }
}
```

**Mock handlers** 实现了完整的业务逻辑，包括：
- 等级计算（与后端一致的阈值表）
- 信仰计算（survival/progress/discipline）
- 任务 CRUD（含虚拟实例合成）
- 历史日期保护

**Mock 存储 keys**:
| Key | 内容 |
|-----|------|
| `mock-tasks` | 任务数据 `MockTask[]` |
| `mock-faith` | 每日信仰记录 `MockFaithRecord[]` |
| `mock-user` | 用户状态 `MockUser` |
| `kanban-board-config` | 看板列配置 `BoardConfig` |

### 5.5 核心 Vue 组件

#### Dashboard.vue — 仪表盘主页
```
┌─────────────────────────────────┐
│    导航栏 (路由切换)              │
├────────────────┬────────────────┤
│   StatusPanel  │   TaskList     │
│   (等级状态)    │   (任务列表)    │
│                │                │
│   FaithDash    │   + TaskForm   │
│   (今日信仰)    │   (新建弹窗)    │
│                │                │
│   DailyGoal    │                │
│   (每日目标)    │                │
│                │                │
│   CalendarView │                │
│   (日历)       │                │
└────────────────┴────────────────┘
```

#### FloatingWidget.vue — 悬浮窗
- 80×80 圆形，渐变深色背景，金色边框
- **拖拽**: mousedown → mousemove 检测阈值(4px) → Tauri `startDragging()`
- **双击**: 调用 `show_main_window` 打开主窗口
- **轮询**: 每 10 秒刷新等级状态

#### Kanban 组件家族

| 组件 | 职责 |
|------|------|
| `KanbanBoard.vue` | 列容器，拖拽事件协调，添加/删除列 |
| `KanbanColumn.vue` | 单列渲染，拖拽接收区 (dragover/drop)，卡片排序 |
| `KanbanCard.vue` | 单卡片：标题/分类/计时/操作按钮 (开始/暂停/完成/放弃) |
| `KanbanCardForm.vue` | 编辑弹窗：标题/分类/预计时长/进程绑定 |

### 5.6 服务层

| 文件 | 功能 |
|------|------|
| `process-detector.ts` | 3 秒间隔轮询检测绑定进程，支持自动启停 |
| `kanban-api.ts` | 看板配置读写 localStorage |
| `reminder-service.ts` | 每分钟检查任务提醒 |

---

## 6. 数据库设计 (5 表)

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `users` | 用户 | cumulative_faith, current_level, armor_points |
| `daily_records` | 每日记录 | work_minutes, study_minutes, survival/progress/discipline_faith, total_faith |
| `tasks` | 任务 | status, category, estimated/actual_minutes, duration_seconds, recurrence_kind, template_id |
| `task_sessions` | 计时会话 | task_id, start_ts, end_ts, seconds |
| `faith_transactions` | 信仰流水 | user_id, delta, kind, ref_id |

详见 `docs/data-model.md`。

---

## 7. 依赖清单

### 7.1 Node.js 依赖 (package.json)

| 依赖 | 用途 |
|------|------|
| `vue` | 前端框架 |
| `vue-router` | 路由 |
| `pinia` | 状态管理 |
| `@tauri-apps/api` | Tauri IPC |
| `typescript` | 类型系统 |
| `vite` | 构建工具 |
| `vue-tsc` | Vue 类型检查 |

### 7.2 Rust 依赖 (Cargo.toml)

| Crate | 用途 |
|-------|------|
| `tauri` (v2) | 桌面应用框架 |
| `rusqlite` (bundled) | SQLite 数据库 |
| `serde` / `serde_json` | 序列化 |
| `chrono` | 日期时间处理 |
| `tracing` | 结构化日志 |
| `tiny_http` | 本地 HTTP 服务 |
| `uuid` | 任务 ID 生成 |

---

## 8. 项目运行方式

### 8.1 开发模式 (浏览器 Mock)

```bash
# 安装前端依赖
npm install

# 启动 Vite 开发服务器 (端口 5173)
npm run dev
```

- 在浏览器中运行，无 Tauri 后端
- 数据存储在 localStorage
- 进程检测 Mock 恒返回 false

### 8.2 桌面模式 (Tauri)

```bash
# 前置条件: Rust toolchain + Tauri CLI

# 安装依赖
npm install

# 开发运行 (热重载)
npm run tauri dev

# 构建生产版本
npm run tauri build
```

**系统要求**:
- Windows (进程检测仅支持 Windows)
- Rust 2021 edition
- Node.js 18+

### 8.3 可用 npm scripts

| 命令 | 说明 |
|------|------|
| `npm run dev` | 启动 Vite dev server (浏览器模式) |
| `npm run build` | 前端 TypeScript 检查 + Vite 构建 |
| `npm run typecheck` | `vue-tsc --noEmit` 类型检查 |
| `npm run tauri dev` | Tauri 开发模式 |
| `npm run tauri build` | Tauri 生产构建 |

### 8.4 窗口配置

| 窗口 | 标签 | 尺寸 | 特性 |
|------|------|------|------|
| 主窗口 | `main` | 900×700 | 普通窗口 |
| 悬浮窗 | `floating` | 80×80 | always_on_top, transparent, decorations=false, skip_taskbar |

---

## 9. 关键设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 状态管理 | Pinia | Vue 3 官方推荐，TypeScript 友好 |
| 路由模式 | Hash | Tauri 本地文件加载兼容性 |
| IPC 双模式 | Tauri invoke + localStorage Mock | 浏览器无 Tauri API，允许独立前端开发 |
| 数据库 | rusqlite bundled | 免安装，嵌入式 |
| 本地 HTTP Server | tiny_http | 轻量同步，无需 async runtime |
| 并发控制 | Mutex | SQLite 单写者模式，足够简单 |
| SDK 风格 | Repository Pattern | 隔离数据库实现 |
| 虚拟任务 | 内存合成 + 延迟物化 | 避免大量重复任务行 |
| 信仰计算 | 纯函数 (domain/) | 零外部依赖，可测试 |

---

## 10. 错误处理规范

所有 Tauri 命令返回 `Result<T, String>`。关键错误消息:

| 错误场景 | 消息 |
|----------|------|
| 非法 category | "category must be 'work', 'study', or 'other'" |
| 预计时间 ≤0 | "estimated_minutes must be > 0" |
| 实际时间 <0 | "actual_minutes must be >= 0" |
| 操作虚拟任务 | "cannot complete/abandon virtual task" |
| 实例晋升模板 | "cannot promote a materialized instance to a template" |
| 前端修改项目任务 | "project task cannot be modified via UI" |
| 非 Windows | "Unsupported platform" |

---

## 11. 开发工具集成 (本地 HTTP API)

牛马信仰启动时开启本地 HTTP 服务，供 Claude/Codex/OpenCode 等 AI 编码工具推送项目任务：

```
开发工具 → POST /api/tasks → 创建项目任务
         → PUT /api/tasks/{session_id} → 更新状态
         → POST /api/tasks/{session_id}/complete → 完成任务
```

项目任务 (task_type='project') 在 UI 中只读，不能从前端修改。

详见 `docs/api-contract.md` 第 7 节。
