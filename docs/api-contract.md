# 牛马信仰 — API 契约文档

> 本文档精确描述前端 ↔ 后端的全部通信接口。删除代码后，仅凭本文档可 100% 还原所有 Tauri 命令和前端调用。

## 1. 通信机制

- **协议**: Tauri IPC（JSON 序列化）
- **前端调用**: `import { invoke } from '@tauri-apps/api/core'; invoke(command, args)`
- **后端定义**: `#[tauri::command] async fn command_name(...) -> Result<T, String>`
- **命令注册**: `main.rs` 中通过 `tauri::generate_handler![...]` 注册全部命令
- **Mock 回退**: 浏览器环境下走 `frontend/src/api/mock-invoke.ts` 中的 `handlers`

## 2. 命令总览

共 **24 个 Tauri 命令**，按功能分组：

| # | 命令 | 分组 | 后端文件 |
|---|------|------|----------|
| 1 | `get_status` | 信仰 | commands.rs |
| 2 | `check_in` | 信仰 | commands.rs |
| 3 | `get_today_record` | 信仰 | commands.rs |
| 4 | `get_or_create_user` | 用户 | commands.rs |
| 5 | `is_process_running` | 进程 | commands.rs |
| 6 | `list_processes` | 进程 | main.rs |
| 7 | `create_task` | 任务 | commands.rs |
| 8 | `get_tasks_by_date` | 任务 | main.rs |
| 9 | `get_tasks` | 任务 | main.rs |
| 10 | `get_task` | 任务 | main.rs |
| 11 | `update_task` | 任务 | commands.rs |
| 12 | `complete_task` | 任务 | commands.rs |
| 13 | `abandon_task` | 任务 | commands.rs |
| 14 | `delete_task` | 任务 | commands.rs |
| 15 | `start_task` | 任务 | commands.rs |
| 16 | `pause_task` | 任务 | commands.rs |
| 17 | `resume_task` | 任务 | commands.rs |
| 18 | `end_task` | 任务 | main.rs |
| 19 | `set_task_recurrence` | 任务 | commands.rs |
| 20 | `get_daily_stats` | 统计 | commands.rs |
| 21 | `get_project_task` | 任务 | commands.rs |
| 22 | `get_project_tasks` | 任务 | commands.rs |
| 23 | `open_floating_widget` | 窗口 | main.rs |
| 24 | `close_floating_widget` | 窗口 | main.rs |
| 25 | `show_main_window` | 窗口 | main.rs |

## 3. 详细契约

### 3.1 信仰系统

---

#### `get_status`

获取用户累计信仰、等级、今日记录。

```typescript
// Request
invoke("get_status", { userId: string })

// Response: FaithStatus
interface FaithStatus {
  user_id: string;              // 用户 ID
  cumulative_faith: number;     // 累计信仰值 (i64)
  current_level: number;        // 当前等级 1-15
  level_title: string;          // 等级称号
  progress_to_next: number;     // 距下一级还需多少信仰
  next_threshold: number | null; // 下一级阈值，满级为 null
  today: DailyRecord | null;    // 今日记录，未打卡为 null
  armor: number;                // 当前护甲值
  total_armor: number;          // 当前等级段的总护甲值
}
```

**后端**: `FaithService::get_status(user_id)` → 查 users + daily_records(today) → 组装 FaithStatus

---

#### `check_in`

每日打卡/更新今日记录。同天重复打卡会覆盖旧值。

```typescript
// Request
invoke("check_in", {
  userId: string,
  workMinutes: number,      // i32, ≥0
  studyMinutes: number,     // i32, ≥0
  breakCount: number,       // i32, 中断次数
  leaveRecord: number,      // i32, 0/1/2
  closeRecord: number,      // i32, 0/1
})

// Response: FaithStatus（同 get_status）
```

**后端**: `FaithService::check_in(user_id, work_minutes, study_minutes, DisciplineInput)`
→ `LedgerService::upsert_daily_record()` → 计算 delta → 更新 daily_records + users + faith_transactions

---

#### `get_today_record`

仅获取今日 DailyRecord（不含累计信仰和等级信息）。

```typescript
// Request
invoke("get_today_record", { userId: string })

// Response: DailyRecord | null
interface DailyRecord {
  id: number | null;
  user_id: string;
  date: string;             // YYYY-MM-DD
  work_minutes: number;
  study_minutes: number;
  survival_faith: number;
  progress_faith: number;
  discipline_faith: number;
  total_faith: number;
  break_count: number;
  leave_record: number;
  close_record: number;
  discipline_a: number;
  discipline_b: number;
  discipline_c: number;
  tasks_completed: number;
  created_at: string;
  updated_at: string;
}
```

---

#### `get_or_create_user`

MVP 单用户应用，固定创建/返回 `default_user`。

```typescript
// Request
invoke("get_or_create_user")

// Response: User
interface User {
  id: string;
  nickname: string;
  cumulative_faith: number;
  current_level: number;
  armor: number;
  total_armor: number;
  created_at: string;
  updated_at: string;
}
```

**注意**: 前端 TypeScript 的 `User` 接口有 `armor` 和 `total_armor` 字段，但后端 Rust 的 `User` 结构体只有 `armor_points`。前端收到后做字段映射。

---

### 3.2 进程检测（Windows Only）

---

#### `is_process_running`

检测指定进程是否正在运行。

```typescript
// Request
invoke("is_process_running", { appName: string })  // 如 "notepad.exe"

// Response: boolean
```

**后端实现**: `tasklist /FI "IMAGENAME eq {app_name}" /NH` → 检查 stdout 是否包含进程名

---

#### `list_processes`

列出所有匹配名称的进程（大小写不敏感）。

```typescript
// Request
invoke("list_processes", { appName: string })

// Response: ProcessInfo[]
interface ProcessInfo {
  pid: number;      // u32
  name: string;     // 进程名
  status: string;   // 状态，如 "Running"
}
```

**后端实现**: `tasklist /FO CSV /NH` → CSV 解析 → 模糊匹配（`to_lowercase().contains()`）

---

### 3.3 任务管理

---

#### `create_task`

创建新任务。默认状态为 `Paused`。

```typescript
// Request
invoke("create_task", {
  userId: string,
  title: string,
  description: string,
  category: "work" | "study" | "other",
  estimatedMinutes: number,     // 必须 > 0
  date: string | null,          // YYYY-MM-DD，null 则使用今天
  recurrenceKind: "none" | "daily" | null,  // null 视为 "none"
})

// Response: Task
interface Task {
  id: string;
  user_id: string;
  date: string;
  title: string;
  description: string;
  category: "work" | "study" | "other";
  estimated_minutes: number;
  actual_minutes: number;
  status: "running" | "paused" | "completed" | "abandoned";
  notes: string;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  duration_seconds: number;
  ai_summary: string | null;
  updated_at: string;
  recurrence_kind?: "none" | "daily";
  template_id?: string | null;
}
```

**验证**:
- `category` 必须是 `"work"` / `"study"` / `"other"`，否则返回错误
- `estimated_minutes` 必须 > 0，否则返回错误

---

#### `get_tasks_by_date`

按日期获取任务列表。**包含虚拟实例合成**。

```typescript
// Request
invoke("get_tasks_by_date", {
  userId: string,
  date: string,           // YYYY-MM-DD
  status: "running" | "paused" | "completed" | "abandoned" | null,
})

// Response: Task[]
```

**虚拟实例合成逻辑**:
1. 查询当日真实任务（匹配 user_id + date + status_filter）
2. 若查询日期是未来（非历史日期）且 status_filter 为 null 或 `"paused"`：
   - 查询该用户的所有 `recurrence_kind = 'daily'` 模板
   - 对每个模板，若当天无实例，则合成虚拟任务：
     - `id = "daily:{template_id}:{date}"`
     - `status = "paused"`
     - `recurrence_kind = "none"`
     - `template_id = {模板id}`
3. 返回真实任务 + 虚拟任务

---

#### `get_tasks`

获取用户全部任务，可按状态过滤。不合成虚拟实例。

```typescript
// Request
invoke("get_tasks", {
  userId: string,
  status: "running" | "paused" | "completed" | "abandoned" | null,
})

// Response: Task[]
```

---

#### `get_task`

按 ID 获取单个任务。

```typescript
// Request
invoke("get_task", { id: string })

// Response: Task | null
```

---

#### `update_task`

更新任务字段。不能修改历史日期任务。

```typescript
// Request
invoke("update_task", {
  id: string,
  title: string | null,
  description: string | null,
  estimatedMinutes: number | null,   // 若提供必须 > 0
  actualMinutes: number | null,
  notes: string | null,
  status: "running" | "paused" | "completed" | "abandoned" | null,
})

// Response: Task
```

**注意**: main.rs 中的 `update_task` 只接受 `title, description, estimated_minutes, notes`，不接受 `actual_minutes` 和 `status`；commands.rs 中的版本接受全部 7 个参数。

---

#### `complete_task`

完成任务，计算并发放奖励信仰。

```typescript
// Request
invoke("complete_task", {
  id: string,
  actualMinutes: number,   // 必须 ≥ 0
})

// Response: TaskCompleteResult
interface TaskCompleteResult {
  task: Task;
  bonus_faith: number;       // 任务完成奖励信仰
  bonus_category: "work" | "study" | "other";
}
```

**后端逻辑**:
1. 验证任务存在且非历史日期
2. 若任务 Running，先关闭当前 session
3. `calc_task_bonus(category, actual_minutes)` → 计算奖励
4. 更新 task.status = Completed, actual_minutes, completed_at
5. `apply_task_bonus()` → 累加到当日 daily_records + users.cumulative_faith + 插入流水

---

#### `abandon_task`

放弃任务（无奖励）。

```typescript
// Request
invoke("abandon_task", { id: string })

// Response: Task
```

**后端逻辑**:
- 验证任务存在且非历史日期
- 若任务 Running，先关闭当前 session
- 更新 task.status = Abandoned

---

#### `delete_task`

永久删除任务。

```typescript
// Request
invoke("delete_task", { id: string })

// Response: boolean   // true = 删除成功
```

**后端逻辑**:
- 若删除的是模板任务（`recurrence_kind = 'daily', template_id IS NULL`），级联删除其所有实例
- 虚拟任务 ID → 直接返回 true（无实际删除）

---

#### `start_task`

开始计时任务。

```typescript
// Request
invoke("start_task", { id: string })

// Response: Task
```

**后端逻辑**:
- 若 ID 是虚拟任务 → `materialize_if_virtual()` 先物化为真实行
- 更新 task.status = Running, started_at = now
- `TaskSessionRepo::start_session(task_id, now_ts)` → 插入新 session

---

#### `pause_task`

暂停计时任务，关闭当前 session。

```typescript
// Request
invoke("pause_task", { id: string })

// Response: Task
```

**后端逻辑**:
- `TaskSessionRepo::end_open_session(task_id)` → 计算 seconds
- 更新 task.duration_seconds += seconds
- 将 seconds 换算为 minutes，累加到当日 daily_records.work/study_minutes
- `LedgerService` 重新计算当日信仰
- 更新 task.status = Paused

---

#### `resume_task`

恢复暂停的任务，开启新 session。

```typescript
// Request
invoke("resume_task", { id: string })

// Response: Task
```

**后端逻辑**:
- 更新 task.status = Running, started_at = now
- `TaskSessionRepo::start_session(task_id, now_ts)` → 插入新 session

---

#### `end_task`

结束任务（标记为 Completed）。**注意：与 complete_task 不同，end_task 不发放奖励信仰。**

```typescript
// Request
invoke("end_task", { id: string })

// Response: Task
```

**后端逻辑**:
- 类似 pause_task：关闭 session，累加时长到 daily_records
- 更新 task.status = Completed
- **不调用** `apply_task_bonus`

---

#### `set_task_recurrence`

设置/取消任务的每日重复。

```typescript
// Request
invoke("set_task_recurrence", {
  id: string,
  kind: "daily" | "none",
})

// Response: Task
```

**后端逻辑**:
- `kind = "daily"`：将普通任务升级为模板（`recurrence_kind = 'daily', template_id = NULL`）
- `kind = "none"`：将模板降级为普通任务
- 已物化的实例不能升级为模板

---

### 3.4 统计

---

#### `get_daily_stats`

获取指定日期的统计信息（含任务奖励拆解）。

```typescript
// Request
invoke("get_daily_stats", {
  userId: string,
  date: string,   // YYYY-MM-DD
})

// Response: DailyStats
interface DailyStats {
  date: string;
  work_minutes: number;
  study_minutes: number;
  survival_faith: number;
  progress_faith: number;
  discipline_faith: number;
  total_faith: number;
  task_bonus_work: number;     // 当日 work 类任务奖励总和
  task_bonus_study: number;    // 当日 study 类任务奖励总和
  tasks_completed: number;
  cumulative_faith: number;    // 当日结束时的累计信仰
}
```

---

### 3.5 窗口控制

---

#### `open_floating_widget`

打开/显示悬浮窗（80×80，置顶，无边框，透明）。

```typescript
// Request
invoke("open_floating_widget")

// Response: void
```

---

#### `close_floating_widget`

隐藏悬浮窗。

```typescript
// Request
invoke("close_floating_widget")

// Response: void
```

---

#### `show_main_window`

显示/还原主窗口。

```typescript
// Request
invoke("show_main_window")

// Response: void
```

## 4. 前端 API 封装层

前端不直接调用 `invoke`，而是通过 `frontend/src/api/` 下的封装函数：

### 4.1 `api/tauri.ts`

```typescript
invoke_get_status(): Promise<FaithStatus>
invoke_get_today_record(): Promise<DailyRecord | null>
invoke_get_or_create_user(): Promise<User>
```

所有函数内部使用 `DEFAULT_USER_ID = "default_user"`。

### 4.2 `api/task.ts`

```typescript
invoke_create_task(title, description, category, estimated_minutes, date?, recurrenceKind?): Promise<Task>
invoke_get_tasks(status?): Promise<Task[]>
invoke_get_tasks_by_date(date, status?): Promise<Task[]>
invoke_get_task(id): Promise<Task | null>
invoke_update_task(id, title?, description?, estimated_minutes?, actual_minutes?, notes?, status?): Promise<Task>
invoke_complete_task(id, actual_minutes): Promise<TaskCompleteResult>
invoke_start_task(id): Promise<Task>
invoke_pause_task(id): Promise<Task>
invoke_resume_task(id): Promise<Task>
invoke_end_task(id): Promise<Task>
invoke_abandon_task(id): Promise<Task>
invoke_delete_task(id): Promise<boolean>
invoke_set_task_recurrence(id, kind): Promise<Task>
invoke_get_daily_stats(date): Promise<DailyStats>
```

### 4.3 参数命名转换

Rust 后端使用 snake_case，前端调用使用 camelCase。Tauri 的 JSON 序列化自动处理字段名映射（通过 `serde`）。

但注意：前端 `api/task.ts` 中传给 `safeInvoke` 的参数名是 camelCase（如 `estimatedMinutes`），而后端 commands.rs 接收的也是 camelCase（因为 Tauri v2 的 invoke 会自动将 camelCase 转为 snake_case 匹配参数名）。

实际上，经代码确认：前端传 `{ estimatedMinutes: 60 }`，后端命令签名是 `estimated_minutes: i32`，Tauri v2 自动做了驼峰到下划线的映射。

## 5. 错误处理

所有命令返回 `Result<T, String>`。错误时前端收到字符串错误消息：

| 错误消息 | 触发命令 | 原因 |
|----------|----------|------|
| `"category must be 'work', 'study', or 'other'"` | create_task | 非法 category |
| `"estimated_minutes must be > 0"` | create_task, update_task | 预计时间 ≤ 0 |
| `"actual_minutes must be >= 0"` | complete_task | 实际时间 < 0 |
| `"invalid status"` | update_task | 非法 status 字符串 |
| `"cannot complete virtual task"` | complete_task | 试图完成虚拟任务 |
| `"cannot abandon virtual task"` | abandon_task | 试图放弃虚拟任务 |
| `"cannot set recurrence on virtual instance"` | set_task_recurrence | 对虚拟实例设置重复 |
| `"cannot promote a materialized instance to a template"` | set_task_recurrence | 实例晋升为模板 |
| `"project task cannot be modified via UI"` | update_task, complete_task, abandon_task, delete_task | 试图从前端修改项目任务 |
| `"Unsupported platform"` | is_process_running, list_processes | 非 Windows 平台 |
| 各类数据库错误 | 多个 | rusqlite 错误转字符串 |

---

## 6. 项目任务 Tauri 命令

### 6.1 `get_project_task`

按 tool_session_id 查询单个项目任务。

```typescript
// Request
invoke("get_project_task", { sessionId: string })

// Response: Task | null
// 返回匹配 tool_session_id 的任务，不存在返回 null
```

**后端**: `TaskService::get_project_task(session_id)` → 查询 `tasks WHERE tool_session_id=?`

---

### 6.2 `get_project_tasks`

获取当前所有活跃的项目任务。

```typescript
// Request
invoke("get_project_tasks", { userId: string })

// Response: Task[]
// 返回 task_type='project' 且 status IN ('running','paused') 的任务列表
```

**后端**: `TaskService::get_project_tasks(user_id)` → 查询活跃项目任务

---

## 7. 本地 HTTP API（开发工具推送接口）

牛马信仰启动时开启本地 HTTP Server（仅监听 127.0.0.1），供开发工具（Claude、Codex、OpenCode 等）主动推送任务。

### 7.1 端口发现

- 端口号写入 `{data_local_dir}/牛马信仰/http_port.txt`
- 认证 Token 写入 `{data_local_dir}/牛马信仰/http_token.txt`
- Token 格式: 16 字节 hex 随机字符串
- 每次启动刷新，退出时删除

### 7.2 API 端点

所有请求需携带 Header: `Authorization: Bearer {token}`

#### POST /api/tasks — 创建项目任务

```
Request Body:
{
  "action": "create",
  "tool_name": "claude",
  "session_id": "uuid-xxxx",
  "title": "重构用户认证模块",
  "description": ""
}

Response 201:
{
  "task_id": "uuid",
  "session_id": "uuid-xxxx",
  "status": "running",
  "created_at": "2026-05-05T14:30:00+08:00"
}
```

#### PUT /api/tasks/{session_id} — 更新任务状态

```
Request Body:
{
  "action": "update",
  "status": "paused",
  "title": "重构用户认证模块"
}

Response 200:
{
  "task_id": "uuid",
  "session_id": "uuid-xxxx",
  "status": "paused"
}
```

#### POST /api/tasks/{session_id}/complete — 完成任务

```
Request Body (可选):
{
  "title": "重构用户认证模块",
  "summary": "完成了JWT认证重构..."
}

Response 200:
{
  "task_id": "uuid",
  "session_id": "uuid-xxxx",
  "status": "completed",
  "duration_minutes": 95,
  "faith_contributed": 30
}
```

#### POST /api/tasks/{session_id}/abandon — 放弃任务

```
Response 200:
{
  "task_id": "uuid",
  "session_id": "uuid-xxxx",
  "status": "abandoned"
}
```

#### GET /api/tasks/{session_id} — 查询任务状态

```
Response 200:
{
  "task_id": "uuid",
  "session_id": "uuid-xxxx",
  "tool_name": "claude",
  "title": "重构用户认证模块",
  "status": "running",
  "duration_seconds": 1800,
  "created_at": "2026-05-05T14:30:00+08:00"
}
```

#### GET /api/health — 健康检查

```
Response 200:
{
  "status": "ok",
  "version": "2.0.0"
}
```

### 7.3 HTTP 错误码

| 状态码 | 含义 |
|--------|------|
| 200 | 成功 |
| 201 | 创建成功 |
| 400 | 请求体格式错误 |
| 401 | Token 缺失或无效 |
| 404 | 会话不存在 |
| 409 | 会话已存在（重复创建） |
