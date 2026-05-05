# 牛马信仰 — 系统设计文档

> 本文档是系统的「单一事实来源」。删除全部代码后，仅凭本文档 + data-model.md + api-contract.md + ui-spec.md 即可还原整个系统。

## 1. 系统全景

```
┌─────────────────────────────────────────────────────────────┐
│  前端 (Vue 3 + TypeScript + Pinia)                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │  仪表盘页    │  │  看板页      │  │  悬浮窗      │       │
│  │  Dashboard   │  │  KanbanPage  │  │  Floating    │       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘       │
│         │                 │                 │               │
│  ┌──────┴─────────────────┴─────────────────┴───────┐       │
│  │  Vue Router (Hash 模式) — 3 路由                  │       │
│  │  /  → Dashboard, /kanban → KanbanPage             │       │
│  │  /floating → FloatingWidget, 其余重定向到 /       │       │
│  └───────────────────────────────────────────────────┘       │
│         │                 │                 │               │
│  ┌──────┴─────────────────┴─────────────────┴───────┐       │
│  │  Pinia Stores: faith / task / kanban              │       │
│  └───────────────────────────────────────────────────┘       │
│         │                                                    │
│  ┌──────┴─────────────────────────────────────────────┐      │
│  │  API Layer: safeInvoke (双模式 IPC)                 │      │
│  │  Tauri 环境 → @tauri-apps/api/core invoke          │      │
│  │  浏览器环境 → localStorage Mock (完整业务逻辑)      │      │
│  └─────────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
                               │
                               │ Tauri IPC (JSON 序列化)
                               │
┌─────────────────────────────────────────────────────────────┐
│  后端 (Rust + Tauri v2 + SQLite)                            │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Tauri Commands (#[tauri::command]) — 25 个命令     │    │
│  │  main.rs 中注册，tauri/commands.rs 中复用于测试    │    │
│  └─────────────────────────────────────────────────────┘    │
│         │                                                    │
│  ┌──────┴─────────────────────────────────────────────┐     │
│  │  Application Services                               │     │
│  │  FaithService → LedgerService → User/Daily Repo     │     │
│  │  TaskService  → Task/Session Repo                    │     │
│  └─────────────────────────────────────────────────────┘     │
│         │                                                    │
│  ┌──────┴─────────────────────────────────────────────┐     │
│  │  Data Layer (SQLite via rusqlite, bundled)          │     │
│  │  5 张表 + WAL 模式 + Mutex 并发保护                 │     │
│  └─────────────────────────────────────────────────────┘     │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐    │
│  │  Local HTTP Server (tiny_http)                       │    │
│  │  127.0.0.1:{port} — 开发工具推送接口                  │    │
│  │  POST/PUT/GET → TaskService → 项目任务生命周期      │    │
│  └──────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                               ↑
                               │ HTTP POST/PUT (JSON)
                               │
┌──────────────────────────────┴──────────────────────────────┐
│  开发工具 (Claude / Codex / OpenCode / ...)                 │
│  通过本地 HTTP API 推送任务：创建 / 更新 / 完成 / 放弃      │
└─────────────────────────────────────────────────────────────┘
```

## 2. 技术栈

| 层级 | 技术 | 版本 | 说明 |
|------|------|------|------|
| 前端框架 | Vue | 3.4 | Composition API, `<script setup>` |
| 状态管理 | Pinia | 2.1 | 3 个 store |
| 路由 | Vue Router | 4.6 | Hash 模式 |
| 构建工具 | Vite | 5.2 | 端口 5173，别名 `@` → `frontend/src` |
| 类型系统 | TypeScript | 5.4 | `vue-tsc --noEmit` 类型检查 |
| 桌面壳 | Tauri | v2 | Rust 后端 + WebView 前端 |
| 后端语言 | Rust | 2021 edition | 分层架构 |
| 数据库 | SQLite | bundled | rusqlite 0.31，WAL 模式 |
| 日期库 | chrono | 0.4 | Rust 端日期处理 |
| 日志 | tracing | 0.1 | 结构化日志 |
| HTTP 服务 | tiny_http | 0.12 | 本地 HTTP API，工具推送接口 |

## 3. 前端架构

### 3.1 目录结构

```
frontend/src/
├── main.ts              # 入口：createApp → use(Pinia) → use(Router) → mount
├── App.vue              # 根组件：导航栏 + router-view，非悬浮窗时初始化 faith store
├── router.ts            # Hash 模式路由，3 条路径
├── style.css            # 全局暗色主题 CSS 变量
├── types/
│   ├── index.ts         # 核心业务类型：User, DailyRecord, FaithStatus, Task 等
│   └── kanban.ts        # 看板类型：KanbanColumn, KanbanCard, ProcessBinding 等
├── api/
│   ├── mock-invoke.ts   # 双模式 invoke 核心 + 浏览器 Mock 完整实现
│   ├── task.ts          # 任务命令封装（所有任务 CRUD / 生命周期）
│   └── tauri.ts         # 信仰/用户命令封装
├── stores/
│   ├── faith.ts         # 信仰状态：faithStatus / user / todayRecord / init() / checkIn()
│   ├── task.ts          # 任务状态：tasks[] / dailyStats / filter / 全部 CRUD actions
│   └── kanban.ts        # 看板状态：columns[] / cards Map / activeTimers / 拖拽同步
├── components/
│   ├── Dashboard.vue           # 仪表盘页面：左侧日历+信仰面板，右侧任务列表
│   ├── CalendarView.vue        # 日历组件：月/周/日三视图，日期选择，今日高亮
│   ├── FaithDashboard.vue      # 今日信仰汇总：survival/progress/discipline breakdown
│   ├── StatusPanel.vue         # 等级状态：Lv 徽章、升级进度条、护甲、今日明细
│   ├── DailyGoalPanel.vue      # 每日目标：工作/学习进度条、任务加成、信仰上限
│   ├── TaskList.vue            # 任务列表：状态筛选标签、操作按钮（开始/暂停/继续/结束/放弃/删除）
│   ├── TaskForm.vue            # 任务创建/编辑弹窗：标题/描述/分类/预计时长/所属列/每日执行
│   ├── KanbanPage.vue          # 看板页容器：加载任务 → 渲染 KanbanBoard
│   ├── FloatingWidget.vue      # 悬浮窗：圆形等级显示，支持拖拽，双击打开主窗口
│   └── kanban/
│       ├── KanbanBoard.vue     # 看板主组件：列管理、拖拽、进程绑定、计时器联动
│       ├── KanbanColumn.vue    # 看板列：拖拽接收区、卡片渲染、列增删
│       ├── KanbanCard.vue      # 看板卡片：任务信息、实时计时器、进程绑定 UI、操作按钮
│       └── KanbanCardForm.vue  # 看板任务表单：创建/编辑 + 进程绑定 + 提醒 + 所属列
├── services/
│   ├── kanban-api.ts         # 看板配置 localStorage 持久化（列配置、卡片位置）
│   ├── process-detector.ts   # 进程检测：轮询绑定进程，3 秒间隔，自动启停回调
│   └── reminder-service.ts   # 任务提醒：每分钟检查提醒时间，降级到浏览器通知
└── utils/
    └── format.ts             # 千分位数字格式化
```

### 3.2 Pinia Store 职责

**`faith.ts`**
- `state`: `faithStatus: FaithStatus | null`, `user: User | null`, `todayRecord: DailyRecord | null`, `loading: boolean`
- `init()`: 调用 `get_or_create_user` → `get_status`，初始化用户和信仰状态
- `checkIn(workMinutes, studyMinutes, discipline)`: 调用 `check_in` 命令，更新状态
- `refreshStatus()`: 重新获取 `get_status`

**`task.ts`**
- `state`: `tasks: Task[]`, `dailyStats: DailyStats | null`, `filter: TaskStatus | 'all'`
- `loadTasks()`, `loadTasksByDate(date)`, `createTask(...)`, `updateTask(...)`, `completeTask(...)`, `abandonTask(...)`, `deleteTask(...)`
- `startTask(id)`, `pauseTask(id)`, `resumeTask(id)`, `endTask(id)` — 计时生命周期
- `setTaskRecurrence(id, kind)` — 切换每日重复

**`kanban.ts`**
- `state`: `columns: KanbanColumn[]`, `cards: Map<string, KanbanCard>`, `activeTimers: Map<string, number>`, `isLoading`
- `loadBoard()`: 从 `kanban-api.ts` 读取列配置，从 `task.ts` 加载任务，合成 cards
- `moveCard(cardId, targetColumnId, targetIndex)`: 更新 cards 和 columns 的 taskIds，持久化
- `addColumn(title)`, `removeColumn(id)`: 列管理
- `startTimer(cardId)`, `stopTimer(cardId)`: 前端实时计时器（setInterval，每秒更新）

### 3.3 双模式 IPC 机制

前端通过 `safeInvoke(command, args)` 与后端通信：

1. **Tauri 模式**：检测 `(window as any).__TAURI_INTERNALS__`，动态 import `@tauri-apps/api/core` 的 `invoke`
2. **Mock 模式**：浏览器 dev 环境，走 `handlers[command]` 的 localStorage 实现

**Mock 实现的关键行为**：
- `STORAGE_TASKS` (JSON) → 存储所有 MockTask
- `STORAGE_FAITH` (JSON) → 存储每日 MockFaithRecord
- `STORAGE_USER` (JSON) → 存储用户状态
- 虚拟任务 ID 格式：`daily:{template_id}:{date}`
- 历史日期保护：`isHistoricalDate(date)` 阻止编辑
- 进程检测 mock 恒返回 false/空数组

## 4. 后端架构

### 4.1 目录结构

```
src-tauri/src/
├── main.rs                  # Tauri 应用入口：日志 → 数据库 → AppState → 窗口/托盘 → 命令注册
├── lib.rs                   # 库入口：公开 domain / data / application / tauri 模块
├── domain/                  # 纯逻辑层，零外部依赖（除 serde）
│   ├── mod.rs               # 导出所有子模块
│   ├── models.rs            # 核心领域模型：User, DailyRecord, FaithStatus, FaithTransaction, ProcessInfo
│   ├── faith.rs             # 纯函数：calc_survival, calc_progress, calc_discipline, calculate_daily
│   ├── level.rs             # 15 级等级阈值表 + 查询函数
│   └── task.rs              # Task 模型、枚举、calc_task_bonus
├── data/                    # 持久化层
│   ├── mod.rs               # 导出 schema / repository / sqlite
│   ├── schema.rs            # SQLite DDL + init_schema() + ensure_column() 增量迁移（索引紧跟列迁移）
│   ├── repository.rs        # Repository Trait 定义（UserRepo, DailyRecordRepo, TaskRepo, FaithTransactionRepo, TaskSessionRepo）
│   └── sqlite.rs            # rusqlite 实现：SqliteDb 结构体 + 全部 Repo 实现（~996 行）
├── application/             # 业务编排层
│   ├── mod.rs               # 导出三大服务
│   ├── faith_service.rs     # 信仰服务：check_in, get_status, get_today_record, get_or_create_user
│   ├── ledger_service.rs    # 记账服务：upsert_daily_record（核心：计算 delta → 更新 daily_records → add_faith → 插入流水）
│   └── task_service.rs      # 任务服务：~1150 行，完整的任务生命周期 + 虚拟实例 + 历史保护 + 项目任务
└── tauri/                   # UI 适配层
    ├── mod.rs               # 导出 state / commands
    ├── state.rs             # AppState：Arc<SqliteDb> → FaithService + TaskService
    └── commands.rs          # 全部 #[tauri::command] 定义，供 main.rs 注册和测试复用
└── local_server.rs          # 本地 HTTP Server：监听 127.0.0.1，处理工具推送请求
```

### 4.2 分层数据流

```
前端 Vue Component
    ↓
Pinia Store (faith / task / kanban)
    ↓
api/task.ts 或 api/tauri.ts (safeInvoke)
    ↓
Tauri IPC (JSON serialize/deserialize)
    ↓
main.rs #[tauri::command] handler
    ↓
AppState → FaithService / TaskService
    ↓
FaithLedgerService / TaskRepo / UserRepo
    ↓
SqliteDb (Mutex<rusqlite::Connection>)
    ↓
SQLite (WAL mode, bundled)
```

### 4.3 AppState 初始化

```rust
// main.rs
let db = Arc::new(SqliteDb::open(db_path)?);  // 自动 init_schema + WAL
let app_state = AppState::new(db);
```

`AppState::new(db)` 内部：
1. 创建 `FaithService::new(db)` → 同时创建 `FaithLedgerService`
2. 创建 `TaskService::new(db)`
3. `FaithService::get_or_create_user()` → 若 `users` 表无 `default_user` 记录则插入

### 4.4 Local HTTP Server

牛马信仰在启动时同时开启一个本地 HTTP 服务，供开发工具（Claude、Codex、OpenCode 等）主动推送项目任务。

**启动流程**：
```rust
// main.rs，在 AppState 创建后
let port = find_available_port();
let token = generate_random_token(16);

write_port_file(&data_dir, port)?;
write_token_file(&data_dir, &token)?;

let server = LocalHttpServer::new(app_state.clone(), port, token);
std::thread::spawn(move || server.run());
```

**依赖**：`tiny_http` crate（轻量级，无 async runtime）

**监听**：仅 `127.0.0.1`，不对外暴露

**端点路由**：
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/health` | 健康检查 |
| POST | `/api/tasks` | 创建项目任务 |
| PUT | `/api/tasks/{session_id}` | 更新任务状态 |
| POST | `/api/tasks/{session_id}/complete` | 完成任务 + 结算工时 |
| POST | `/api/tasks/{session_id}/abandon` | 放弃任务 |
| GET | `/api/tasks/{session_id}` | 查询任务状态 |

**认证**：每个请求 Header 需携带 `Authorization: Bearer {token}`

**端口/Token 文件**：
- `{data_local_dir}/牛马信仰/http_port.txt` — 当前端口
- `{data_local_dir}/牛马信仰/http_token.txt` — 当前 Token
- 应用退出时自动删除

**与 TaskService 的关系**：
```
HTTP Request → LocalHttpServer
  → 验证 Token
  → 解析 JSON body
  → 路由到 TaskService 对应方法:
      POST /api/tasks         → TaskService::create_project_task(...)
      PUT /api/tasks/{id}     → TaskService::pause_task / resume_task
      POST .../complete       → TaskService::end_task + 结算工时
      POST .../abandon        → TaskService::abandon_task
      GET /api/tasks/{id}     → TaskService::get_project_task
```

### 4.5 关键 Rust 模块

**`domain/faith.rs`**
- `calc_survival(minutes: i32) -> i32`：阶梯函数，120/240/360/480 分钟为阈值，分别返回 0/100/200/300/400
- `calc_progress(minutes: i32) -> i32`：同 survival
- `calc_discipline(input: DisciplineInput) -> (i32, i32, i32, i32)`：
  - break_count ≤2 → a=80, 3-4 → a=40, ≥5 → a=0
  - leave_record 0 → b=60, 1 → b=30, 其他 → b=0
  - close_record ≥1 → c=60, 其他 → c=0
  - 返回 (total, a, b, c)
- `calculate_daily(work, study, discipline) -> FaithBreakdown`：组装三部分

**`domain/level.rs`**
- 15 级阈值表（v2.0 全部 ×10）：Lv1=0, Lv2=15,000, ..., Lv15=1,095,000
- `get_level(cumulative_faith) -> Level`：从后向前扫描
- `progress_to_next(cumulative_faith) -> Option<i64>`
- `interval_to_next(cumulative_faith) -> Option<i64>`

**`domain/task.rs`**
- `TaskCategory`: Work / Study / Other（序列化为小写）
- `TaskStatus`: Running / Paused / Completed / Abandoned
- `RecurrenceKind`: None / Daily
- `calc_task_bonus(category, actual_minutes) -> i32`：Work/Study 每小时 5 分，Other 每小时 2 分，最少按 1 小时算

**`application/ledger_service.rs`**
- `upsert_daily_record(user_id, date, work_minutes, study_minutes, discipline, breakdown, now_ts)`：
  1. 计算新的 total_faith
  2. 计算 delta = new_total - old_total（旧记录存在时）
  3. `INSERT ... ON CONFLICT(user_id, date) DO UPDATE` 写入 daily_records
  4. `UserRepo::add_faith(user_id, delta)` → 自动重新计算 current_level
  5. 插入 `faith_transactions` 流水记录

**`application/task_service.rs`**
- 核心设计：
  - **虚拟实例**：`recurrence_kind = 'daily'` 的模板行，在 `get_tasks_by_date` 查询未来日期时，内存中合成 `daily:{template_id}:{date}` 虚拟任务
  - **物化**：首次对虚拟任务执行 `start/pause/complete` 等操作时，通过 `materialize_if_virtual()` 写入真实行
  - **历史保护**：`is_historical(date)` 阻止对过去日期任务的修改/完成/放弃/删除
  - **计时联动**：`pause_task` / `end_task` 自动关闭当前 session，计算秒数，累加到当日 `daily_records` 的 work/study_minutes，并通过 ledger 重新计算日信仰
  - **任务奖励**：`complete_task` 调用 `calc_task_bonus`，通过 `apply_task_bonus` 累加到日记录与累计信仰
  - **项目任务保护**：`task_type = 'project'` 的任务在前端只读，不可通过 Tauri 命令 edit/complete/abandon/delete
  - **项目任务查询**：`get_project_task(session_id)` 按 tool_session_id 查找

### 4.6 数据库路径策略

```
优先级 1: dirs::data_local_dir()/牛马信仰/niuma_faith.db  (若已存在)
优先级 2: exe 所在目录 / niuma_faith.db                    (开发兼容)
```

### 4.7 系统托盘

- 左键单击托盘图标 → 显示主窗口
- 右键菜单：显示主窗口 / 打开悬浮窗 / 退出
- `show_menu_on_left_click(false)` — 左键不弹菜单，只触发显示窗口

### 4.8 窗口配置

| 窗口 | 标签 | 尺寸 | 特性 |
|------|------|------|------|
| 主窗口 | `main` | 900×700 | 普通窗口 |
| 悬浮窗 | `floating` | 80×80 | always_on_top, decorations=false, skip_taskbar, transparent, shadow=false |

## 5. 数据流与状态同步

### 5.1 信仰计算数据流

```
用户操作（打卡/任务完成/计时暂停）
    ↓
FaithService.check_in() / TaskService.pause_task() / complete_task()
    ↓
LedgerService.upsert_daily_record()
    ↓
  ├─ 写入/更新 daily_records（当日信仰明细）
  ├─ UserRepo.add_faith(delta) → 更新 users.cumulative_faith + current_level
  └─ 插入 faith_transactions（流水）
    ↓
前端调用 get_status → 返回 FaithStatus（含最新累计信仰、等级、今日记录）
```

### 5.2 任务计时数据流

```
前端点击「开始」→ start_task(id)
    ↓
TaskService.start_task()
    ├─ 更新 tasks.status = Running, started_at = now
    └─ TaskSessionRepo.start_session(task_id, start_ts)

前端点击「暂停」→ pause_task(id)
    ↓
TaskService.pause_task()
    ├─ TaskSessionRepo.end_open_session(task_id) → 计算 seconds
    ├─ 更新 tasks.duration_seconds += seconds
    ├─ 将 seconds 换算为 minutes，累加到当日 daily_records.work/study_minutes
    ├─ LedgerService 重新计算当日信仰
    └─ 更新 tasks.status = Paused
```

### 5.3 看板数据流

```
KanbanPage 加载
    ↓
kanban store.loadBoard()
    ├─ kanban-api.ts 读取 localStorage「kanban-board-config」→ BoardConfig（列配置）
    ├─ 若无配置，创建默认四列：待办/进行中/暂停中/已完成
    ├─ task store 加载当日任务
    └─ 将任务按 status 映射到默认列，合成 KanbanCard Map

用户拖拽卡片
    ↓
KanbanBoard @drop 事件
    ↓
kanban store.moveCard(cardId, targetColumnId, targetIndex)
    ├─ 更新 cards Map（columnId, orderInColumn）
    ├─ 更新 columns[] 的 taskIds 顺序
    └─ kanban-api.ts 持久化 BoardConfig 到 localStorage
```

## 6. 核心算法规范

### 6.1 信仰计算（每日）

```
survival_faith  = calc_survival(work_minutes)    // 0~400, 阶梯
progress_faith  = calc_progress(study_minutes)   // 0~400, 阶梯
discipline_faith = a + b + c                     // 0~200
  a (专注稳定) = break_count ≤2 ? 80 : (≤4 ? 40 : 0)
  b (离岗纪律) = leave_record ==0 ? 60 : (==1 ? 30 : 0)
  c (记录闭环) = close_record ≥1 ? 60 : 0
daily_record.total_faith = survival + progress + discipline + task_bonus_work + task_bonus_study
// 三信仰支柱上限 1000，任务奖励可叠加超出
```

### 6.2 等级系统

```
Lv1:  0           见习牛马
Lv2:  15,000      工位信徒
Lv3:  40,000      初级供奉者
Lv4:  80,000      稳定产出者
Lv5:  135,000     自律门徒
Lv6:  205,000     双修学徒
Lv7:  290,000     工时祭司
Lv8:  395,000     苦修执行官
Lv9:  520,000     连轴修行者
Lv10: 665,000     钢铁牛马
Lv11: 825,000     卷力使徒
Lv12: 945,000     精进主教
Lv13: 1,025,000   福报传道者
Lv14: 1,070,000   31日苦修士
Lv15: 1,095,000   牛马圣徒

progress_to_next = next_threshold - cumulative_faith (若未满级)
interval_to_next = next_threshold - current_threshold
```

### 6.3 护甲系统（v2.0）

```
calc_armor(current_level):
  Lv2-Lv5:   2,000
  Lv6-Lv10:  4,000
  Lv11-Lv15: 6,000
  Lv1:       0

扣分规则：先扣 armor_points，armor 为 0 后才扣 cumulative_faith
升级时自动按 calc_armor 设置 armor_points
```

### 6.4 任务奖励

```
calc_task_bonus(category, actual_minutes):
  rate = Work/Study ? 5 : 2
  hours = max(1, actual_minutes / 60)  // 整数除法，最少 1 小时
  return hours * rate
```

## 7. 异常与边界

| 场景 | 行为 |
|------|------|
| 历史日期任务 | `is_historical(date)` → true 时禁止 edit/complete/abandon/delete |
| 虚拟任务完成/放弃 | 抛出错误 "cannot complete/abandon virtual task" |
| 虚拟任务设置重复 | 抛出错误 "cannot set recurrence on virtual instance" |
| 实例晋升模板 | 抛出错误 "cannot promote a materialized instance to a template" |
| estimated_minutes ≤ 0 | create_task / update_task 返回错误 |
| actual_minutes < 0 | complete_task 返回错误 |
| 不存在的任务 | start/pause/resume/complete/abandon/delete 返回错误 |
| 非 Windows 进程检测 | 返回错误 "Unsupported platform" |
| 同天重复打卡 | `ON CONFLICT DO UPDATE` 覆盖，delta 按新旧差值计算 |
| 并发数据库访问 | Mutex 保护，串行化 |

## 8. 配置与常量

### 8.1 CSS 变量（style.css）

```css
:root {
  --color-bg: #1a1a24;
  --color-surface: #222233;
  --color-surface-hover: #2a2a3e;
  --color-border: #333344;
  --color-text: #e0e0e0;
  --color-text-muted: #888899;
  --color-primary: #ffd700;    /* 金色 */
  --color-primary-dim: #b8860b;
  --color-success: #4ade80;
  --color-danger: #ef4444;
}
```

### 8.2 进程检测轮询间隔

```typescript
// process-detector.ts
const POLL_INTERVAL_MS = 3000;  // 3 秒
```

### 8.3 提醒检查间隔

```typescript
// reminder-service.ts
const REMINDER_INTERVAL_MS = 60000;  // 1 分钟
```

---

> **文档关联**: data-model.md ← 数据库完整 Schema | api-contract.md ← 全部命令签名 | ui-spec.md ← 组件与交互细节 | workflows.md ← 业务流程 | build-guide.md ← 构建还原步骤
