# 牛马信仰 — 数据模型文档

> 本文档精确描述 SQLite 数据库的完整 Schema、字段含义、约束、索引和业务关系。删除代码后，仅凭本文档可 100% 还原数据库结构。

## 1. 数据库配置

| 配置项 | 值 | 说明 |
|--------|-----|------|
| 引擎 | SQLite 3 | rusqlite bundled |
| 日志模式 | WAL | `PRAGMA journal_mode=WAL` |
| 外键 | ON | `PRAGMA foreign_keys=ON` |
| 并发 | Mutex | `Mutex<rusqlite::Connection>` 单写 |
| 路径 | `data_local_dir/牛马信仰/niuma_faith.db` 或 `exe_dir/niuma_faith.db` |

## 2. 表结构

### 2.1 users — 用户表

```sql
CREATE TABLE IF NOT EXISTS users (
    id               TEXT PRIMARY KEY,
    nickname         TEXT NOT NULL DEFAULT '',
    cumulative_faith INTEGER NOT NULL DEFAULT 0,
    current_level    INTEGER NOT NULL DEFAULT 1,
    armor_points     INTEGER NOT NULL DEFAULT 0,  -- 增量迁移添加
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);
```

| 字段 | Rust 类型 | TypeScript 类型 | 说明 |
|------|-----------|-----------------|------|
| id | `String` | `string` | 主键，MVP 固定为 `"default_user"` |
| nickname | `String` | `string` | 用户昵称 |
| cumulative_faith | `i64` | `number` | 累计信仰值，v2.0 单位 |
| current_level | `i32` | `number` | 当前等级 1-15 |
| armor_points | `i32` | `number` | 当前护甲值 |
| created_at | `String` (ISO-8601) | `string` | 创建时间戳 |
| updated_at | `String` (ISO-8601) | `string` | 更新时间戳 |

**业务规则**:
- `UserRepo::add_faith(delta)` 会自动重新计算 `current_level`（基于 `domain/level.rs` 阈值表）
- 护甲在升级时按 `calc_armor(current_level)` 设置

---

### 2.2 daily_records — 每日打卡记录表

```sql
CREATE TABLE IF NOT EXISTS daily_records (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id          TEXT NOT NULL,
    date             TEXT NOT NULL,           -- YYYY-MM-DD
    work_minutes     INTEGER NOT NULL DEFAULT 0,
    study_minutes    INTEGER NOT NULL DEFAULT 0,
    survival_faith   INTEGER NOT NULL DEFAULT 0,
    progress_faith   INTEGER NOT NULL DEFAULT 0,
    discipline_faith INTEGER NOT NULL DEFAULT 0,
    total_faith      INTEGER NOT NULL DEFAULT 0,
    task_bonus_work  INTEGER NOT NULL DEFAULT 0,   -- 增量迁移添加
    task_bonus_study INTEGER NOT NULL DEFAULT 0,   -- 增量迁移添加
    break_count      INTEGER NOT NULL DEFAULT 0,
    leave_record     INTEGER NOT NULL DEFAULT 0,
    close_record     INTEGER NOT NULL DEFAULT 0,
    discipline_a     INTEGER NOT NULL DEFAULT 0,
    discipline_b     INTEGER NOT NULL DEFAULT 0,
    discipline_c     INTEGER NOT NULL DEFAULT 0,
    tasks_completed  INTEGER NOT NULL DEFAULT 0,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    UNIQUE(user_id, date)
);

CREATE INDEX IF NOT EXISTS idx_daily_user_date ON daily_records(user_id, date);
```

| 字段 | Rust 类型 | TypeScript 类型 | 说明 |
|------|-----------|-----------------|------|
| id | `Option<i64>` | `number \| null` | 自增主键 |
| user_id | `String` | `string` | 关联用户 |
| date | `String` | `string` | 日期，格式 `YYYY-MM-DD` |
| work_minutes | `i32` | `number` | 工作分钟数（来自计时 session + 手动打卡） |
| study_minutes | `i32` | `number` | 学习分钟数 |
| survival_faith | `i32` | `number` | 当日生存信仰（calc_survival(work_minutes)） |
| progress_faith | `i32` | `number` | 当日精进信仰（calc_progress(study_minutes)） |
| discipline_faith | `i32` | `number` | 当日戒律信仰（a+b+c） |
| total_faith | `i32` | `number` | 当日总信仰 = survival + progress + discipline + task_bonus_work + task_bonus_study |
| task_bonus_work | `i32` | `number` | 当日 work 类任务完成奖励总和 |
| task_bonus_study | `i32` | `number` | 当日 study 类任务完成奖励总和 |
| break_count | `i32` | `number` | 休息/中断次数 |
| leave_record | `i32` | `number` | 离岗记录：0=无/已解释, 1=已解释, 2=未解释 |
| close_record | `i32` | `number` | 记录闭环：0=未完成, ≥1=已完成 |
| discipline_a | `i32` | `number` | 专注稳定得分 (0/40/80) |
| discipline_b | `i32` | `number` | 离岗纪律得分 (0/30/60) |
| discipline_c | `i32` | `number` | 记录闭环得分 (0/60) |
| tasks_completed | `i32` | `number` | 当日完成任务数 |
| created_at | `String` | `string` | 创建时间戳 |
| updated_at | `String` | `string` | 更新时间戳 |

**约束**: `UNIQUE(user_id, date)` — 每个用户每天只有一条记录

**写入方式**: `INSERT ... ON CONFLICT(user_id, date) DO UPDATE` — 同天打卡覆盖旧值

---

### 2.3 tasks — 任务表

```sql
CREATE TABLE IF NOT EXISTS tasks (
    id                TEXT PRIMARY KEY,
    user_id           TEXT NOT NULL,
    date              TEXT NOT NULL DEFAULT '',   -- 增量迁移
    title             TEXT NOT NULL,
    description       TEXT NOT NULL DEFAULT '',
    category          TEXT NOT NULL,              -- 'work' | 'study' | 'other'
    estimated_minutes INTEGER NOT NULL DEFAULT 0,
    actual_minutes    INTEGER NOT NULL DEFAULT 0,
    status            TEXT NOT NULL,              -- 'running' | 'paused' | 'completed' | 'abandoned'
    notes             TEXT NOT NULL DEFAULT '',
    created_at        TEXT NOT NULL,
    started_at        TEXT,                       -- 增量迁移
    completed_at      TEXT,
    duration_seconds  INTEGER NOT NULL DEFAULT 0, -- 增量迁移，累计计时秒数
    ai_summary        TEXT,                       -- 增量迁移
    updated_at        TEXT NOT NULL,
    recurrence_kind   TEXT NOT NULL DEFAULT 'none', -- 增量迁移：'none' | 'daily'
    template_id       TEXT,                       -- 增量迁移：指向模板任务的自引用
    task_type        TEXT NOT NULL DEFAULT 'daily', -- 增量迁移：'daily' | 'project'
    source_tool      TEXT,                        -- 增量迁移：来源工具名(project任务)
    tool_session_id  TEXT                         -- 增量迁移：工具侧唯一会话ID
);

CREATE INDEX IF NOT EXISTS idx_tasks_user_status ON tasks(user_id, status);
CREATE INDEX IF NOT EXISTS idx_tasks_user_recurrence_kind
    ON tasks(user_id, recurrence_kind) WHERE recurrence_kind != 'none';
CREATE INDEX IF NOT EXISTS idx_tasks_template_id_date
    ON tasks(template_id, date) WHERE template_id IS NOT NULL;
```

| 字段 | Rust 类型 | TypeScript 类型 | 说明 |
|------|-----------|-----------------|------|
| id | `String` | `string` | 主键，UUID 或 `daily:{template_id}:{date}` 虚拟 ID |
| user_id | `String` | `string` | 所属用户 |
| date | `String` | `string` | 任务日期 `YYYY-MM-DD`，空字符串为无日期 |
| title | `String` | `string` | 任务标题 |
| description | `String` | `string` | 任务描述 |
| category | `TaskCategory` | `TaskCategory` | work / study / other |
| estimated_minutes | `i32` | `number` | 预计用时（分钟），必须 > 0 |
| actual_minutes | `i32` | `number` | 实际用时（分钟） |
| status | `TaskStatus` | `TaskStatus` | running / paused / completed / abandoned |
| notes | `String` | `string` | 备注 |
| created_at | `String` | `string` | 创建时间戳 |
| started_at | `Option<String>` | `string \| null` | 最近一次开始计时时间 |
| completed_at | `Option<String>` | `string \| null` | 完成时间戳 |
| duration_seconds | `i64` | `number` | 累计计时秒数（各 session 之和） |
| ai_summary | `Option<String>` | `string \| null` | AI 生成的任务摘要 |
| updated_at | `String` | `string` | 更新时间戳 |
| recurrence_kind | `RecurrenceKind` | `RecurrenceKind` | none / daily |
| template_id | `Option<String>` | `string \| null` | 指向同表模板任务的 id（自引用） |
| task_type | `TaskType` | `TaskType` | daily / project |
| source_tool | `Option<String>` | `string \| null` | 来源工具名（如 claude/codex/opencode），project 任务专用 |
| tool_session_id | `Option<String>` | `string \| null` | 工具侧唯一会话 ID，用于去重和后续更新 |

**业务关系**:
- **模板任务**: `recurrence_kind = 'daily'` 且 `template_id IS NULL` → 这是每日重复模板
- **实例任务**: `template_id = {模板id}` 且 `recurrence_kind = 'none'` → 某一天的实例
- **虚拟实例**: 查询时内存合成的 `daily:{template_id}:{date}`，首次操作才物化为真实行
- **日常任务**: `task_type = 'daily'` → 用户手动创建和管理
- **项目任务**: `task_type = 'project'` → 由开发工具远程推送创建，用户只读
- **去重**: `tool_session_id` 用于项目任务去重，同一 session_id 不可重复创建

---

### 2.4 task_sessions — 任务计时会话表

```sql
CREATE TABLE IF NOT EXISTS task_sessions (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id   TEXT NOT NULL,
    start_ts  TEXT NOT NULL,    -- ISO-8601
    end_ts    TEXT,             -- ISO-8601, NULL 表示未结束
    seconds   INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_task_sessions_task_id ON task_sessions(task_id);
```

| 字段 | Rust 类型 | 说明 |
|------|-----------|------|
| id | `Option<i64>` | 自增主键 |
| task_id | `String` | 关联任务 ID |
| start_ts | `String` | 开始时间戳 ISO-8601 |
| end_ts | `Option<String>` | 结束时间戳，Running 时为 NULL |
| seconds | `i32` | 持续秒数，end 时计算 |

**生命周期**:
1. `start_task` → `TaskSessionRepo::start_session(task_id, now_ts)` → 插入新行（end_ts=NULL, seconds=0）
2. `pause_task` / `end_task` → `TaskSessionRepo::end_open_session(task_id)` → 查找 end_ts=NULL 的行，计算 seconds，更新 end_ts

---

### 2.5 faith_transactions — 信仰变动流水表

```sql
CREATE TABLE IF NOT EXISTS faith_transactions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     TEXT NOT NULL,
    ts          TEXT NOT NULL,              -- ISO-8601
    delta       INTEGER NOT NULL,           -- 信仰变化值（可为负）
    armor_delta INTEGER NOT NULL DEFAULT 0, -- 护甲变化值
    kind        TEXT NOT NULL,              -- 类型标记：'check_in' / 'task_bonus' / 'session' 等
    ref_id      TEXT,                       -- 关联业务 ID（如 task_id）
    message     TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_faith_tx_user_ts ON faith_transactions(user_id, ts);
```

| 字段 | Rust 类型 | 说明 |
|------|-----------|------|
| id | `Option<i64>` | 自增主键 |
| user_id | `String` | 用户 ID |
| ts | `String` | 变动时间戳 |
| delta | `i32` | 信仰变化值（正=增加，负=扣除） |
| armor_delta | `i32` | 护甲变化值 |
| kind | `String` | 业务类型标记 |
| ref_id | `Option<String>` | 关联业务 ID |
| message | `String` | 描述消息 |

## 3. Schema 迁移策略

### 3.1 初始化流程

```rust
// SqliteDb::open(path)
1. 打开数据库连接
2. PRAGMA journal_mode=WAL
3. PRAGMA foreign_keys=ON
4. init_schema(&conn) — 执行 SCHEMA_SQL
5. ensure_column 增量迁移（向后兼容）
```

### 3.2 增量迁移字段清单

以下字段通过 `ensure_column()` 在启动时自动添加（兼容旧数据库）：

| 表 | 字段 | 类型 | 添加版本 |
|----|------|------|----------|
| users | armor_points | `INTEGER NOT NULL DEFAULT 0` | v2.0 |
| tasks | started_at | `TEXT` | — |
| tasks | duration_seconds | `INTEGER NOT NULL DEFAULT 0` | — |
| tasks | ai_summary | `TEXT` | — |
| tasks | date | `TEXT NOT NULL DEFAULT ''` | — |
| tasks | recurrence_kind | `TEXT NOT NULL DEFAULT 'none'` | — |
| tasks | template_id | `TEXT` | — |
| tasks | task_type | `TEXT NOT NULL DEFAULT 'daily'` | v2.1 |
| tasks | source_tool | `TEXT` | v2.1 |
| tasks | tool_session_id | `TEXT` | v2.1 |
| daily_records | task_bonus_work | `INTEGER NOT NULL DEFAULT 0` | v2.1 |
| daily_records | task_bonus_study | `INTEGER NOT NULL DEFAULT 0` | v2.1 |

### 3.3 索引与迁移顺序

**关键约束**：引用增量迁移字段的索引必须在对应 `ensure_column()` 之后创建，否则旧数据库迁移时列不存在导致初始化崩溃。

**基础索引**（依赖 CREATE TABLE 已定义的列，在 init_schema 中直接创建）：

```sql
CREATE INDEX IF NOT EXISTS idx_daily_user_date ON daily_records(user_id, date);
CREATE INDEX IF NOT EXISTS idx_tasks_user_status ON tasks(user_id, status);
CREATE INDEX IF NOT EXISTS idx_task_sessions_task_id ON task_sessions(task_id);
CREATE INDEX IF NOT EXISTS idx_faith_tx_user_ts ON faith_transactions(user_id, ts);
```

**增量索引**（紧跟对应 `ensure_column()` 之后创建）：

| 索引名 | 依赖列 | 创建位置 |
|--------|--------|----------|
| `idx_tasks_user_recurrence` | `tasks.recurrence_kind` | `ensure_column("recurrence_kind")` 之后 |
| `idx_tasks_template_id_date` | `tasks.template_id` | `ensure_column("template_id")` 之后 |
| `idx_tasks_task_type` | `tasks.task_type` | `ensure_column("task_type")` 之后 |
| `idx_tasks_tool_session` | `tasks.tool_session_id` | `ensure_column("tool_session_id")` 之后 |

## 4. Rust ↔ TypeScript 类型映射

| Rust 类型 | TypeScript 类型 | 序列化说明 |
|-----------|-----------------|------------|
| `String` | `string` | 直接 |
| `i32` | `number` | 直接 |
| `i64` | `number` | 直接（JS 安全整数范围内） |
| `Option<T>` | `T \| null` | `Some` → 值, `None` → `null` |
| `TaskCategory` | `"work" \| "study" \| "other"` | `#[serde(rename_all = "lowercase")]` |
| `TaskStatus` | `"running" \| "paused" \| "completed" \| "abandoned"` | 同上 |
| `RecurrenceKind` | `"none" \| "daily"` | 同上 |
| `TaskType` | `"daily" \| "project"` | 同上 |

## 5. 前端 localStorage Mock Schema

浏览器 dev 模式下，Mock 使用以下 localStorage keys：

| Key | 内容 | 类型 |
|-----|------|------|
| `mock-tasks` | `MockTask[]` JSON | 任务数据 |
| `mock-faith` | `MockFaithRecord[]` JSON | 每日信仰记录 |
| `mock-user` | `MockUser` JSON | 用户状态 |
| `kanban-board-config` | `BoardConfig` JSON | 看板列配置 |

**Mock 与真实后端的差异**（代码生成时必须消除以下差异）：
- Mock 的等级阈值必须与后端一致（使用 v2.0 ×10 阈值表）
- Mock 的 `check_in` 应真正计算信仰（非仅标记 check_in_done）
- Mock 的 `get_daily_stats` 应计算任务奖励
- Mock 的进程检测恒返回 false（开发环境正常行为）
