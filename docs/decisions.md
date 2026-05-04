# 牛马信仰 — 架构决策记录 (ADR)

> 本文档记录关键架构决策及其理由。删除全部代码后，仅凭本文档 + design-doc.md + data-model.md 可理解为何如此设计。

## 决策索引

| ID | 标题 | 状态 | 日期 |
|----|------|------|------|
| ADR-001 | 使用 Tauri v2 作为桌面应用框架 | 已采纳 | 2024 |
| ADR-002 | 使用 SQLite（bundled rusqlite）作为本地数据库 | 已采纳 | 2024 |
| ADR-003 | 领域驱动分层架构（domain/data/application/tauri） | 已采纳 | 2024 |
| ADR-004 | 虚拟任务实例系统（Daily 模板 → 虚拟实例） | 已采纳 | 2024 |
| ADR-005 | 双模式 IPC（Tauri 真实调用 + 浏览器 Mock） | 已采纳 | 2024 |
| ADR-006 | 信仰阈值 v2.0 升级（全部 ×10） | 已采纳 | 2025-05 |
| ADR-007 | 护甲系统 v2（分等级段设定护甲值） | 已采纳 | 2025-05 |
| ADR-008 | 进程绑定实现任务自动启停 | 已采纳 | 2024 |
| ADR-009 | 数据库路径优先级策略 | 已采纳 | 2025-05 |
| ADR-010 | 主窗口 + 悬浮窗双窗口模式 | 已采纳 | 2024 |

---

## ADR-001: 使用 Tauri v2 作为桌面应用框架

### 状态
已采纳

### 背景
需要一个跨平台桌面应用，同时嵌入 Web 前端（Vue）和 Rust 后端。

### 决策
使用 Tauri v2 桌面框架：
- 后端：Rust（二进制，打包小）
- 前端：嵌入 WebView（HTML/CSS/JS）
- 通信：Tauri IPC（JSON 序列化）

### 理由
- **轻量**：比 Electron 小很多（无 Node.js 运行时）
- **安全**：Rust 内存安全，默认沙盒
- **体验**：原生窗口、菜单、托盘
- **技术栈匹配**：前端 Vue + 后端 Rust 的组合适合本项目需求

### 后果
- 前端必须通过 `invoke()` 与后端通信，不能直接调用 Rust
- 调试时可分为 Vite dev server（浏览器模式）和 Tauri 桌面应用（真实模式）

---

## ADR-002: 使用 SQLite（bundled rusqlite）作为本地数据库

### 状态
已采纳

### 背景
需要一个零配置的本地持久化存储，支持多表关联、事务、并发安全。

### 决策
使用 `rusqlite` 的 `bundled` 特性，SQLite 数据库文件：
- 路径：见 ADR-009
- 模式：WAL（Write-Ahead Logging）
- 并发保护：`Arc<Mutex<Connection>>`
- 迁移：增量列迁移（`ensure_column`）

### 理由
- **零配置**：SQLite 无需独立服务器进程
- **嵌入式**：打包到可执行文件，用户无感知
- **跨平台**：Windows/macOS/Linux 一致
- **WAL 模式**：读不阻塞写，写不阻塞读
- **bundled**：编译器内置 SQLite，无需用户安装

### 约束
- 所有数据库操作必须通过 `SqliteDb`（不允许直接持有 `Connection`）
- 迁移只能添加列，不能删除或修改现有列

---

## ADR-003: 领域驱动分层架构

### 状态
已采纳

### 决策

```
src-tauri/src/
├── domain/          # 纯逻辑，零外部依赖
│   ├── models.rs    # 领域模型（User, Task, DailyRecord, FaithStatus）
│   ├── faith.rs     # 信仰计算纯函数
│   ├── level.rs     # 等级阈值表
│   └── task.rs      # 任务模型、枚举、奖励计算
├── data/            # 持久化层
│   ├── schema.rs    # DDL + 增量迁移
│   ├── repository.rs # Repository Trait
│   └── sqlite.rs    # rusqlite 实现
├── application/     # 业务编排层
│   ├── faith_service.rs   # 信仰/打卡业务
│   ├── ledger_service.rs  # 每日记账
│   └── task_service.rs    # 任务生命周期
└── tauri/           # UI 适配层
    ├── state.rs     # AppState（依赖注入容器）
    └── commands.rs  # Tauri 命令
```

### 理由
- **可测试性**：domain 层无任何外部依赖，可直接单元测试
- **职责清晰**：每层只做一件事
- **可替换性**：data 层可以换成别的数据库实现而不影响业务逻辑
- **可读性**：新人可以按层次理解代码

### 约束
- `domain/` 层禁止引入 `rusqlite`、`tauri` 等外部依赖
- `data/` 层通过 Trait 定义接口，`application/` 层依赖 Trait 而非具体实现

---

## ADR-004: 虚拟任务实例系统

### 状态
已采纳

### 背景
需要支持「每日重复」任务：用户创建一个模板，每天自动出现一个实例。

### 决策

**模板行**：`recurrence_kind = 'daily'` 且 `template_id IS NULL`
- 创建时即是一个真实的 `tasks` 表记录
- 可被 `start/pause/complete`

**虚拟实例**：按需合成，不写入数据库
- 触发条件：`get_tasks_by_date` 查询未来日期时
- 合成规则：查找该用户所有模板行，若当天无实例，合成虚拟任务
- 虚拟 ID 格式：`daily:{template_id}:{date}`
- 虚拟任务状态：`status = 'paused'`, `recurrence_kind = 'none'`

**物化**：首次对虚拟任务执行写操作时
- `start_task` 检测虚拟 ID → 调用 `materialize_if_virtual()` → 插入真实行 → 后续操作针对真实行

**历史保护**：
- 历史日期（`date < today`）：不合成虚拟实例
- 历史日期任务：只读，禁止 edit/complete/abandon/delete

### 理由
- **查询效率**：虚拟实例不落盘，按需合成，数据库无膨胀
- **一致性**：所有真实实例共享模板的 `title`、`category`、`estimated_minutes`
- **可逆性**：物化后变为普通任务，可以独立操作而不影响模板
- **简洁**：不需要额外的 `task_templates` 表

### 约束
- 模板设置 `recurrence_kind = 'daily'` 后，`template_id` 必须为 `NULL`
- 已物化的实例不能再晋升为模板（`cannot promote a materialized instance to a template`）
- 虚拟任务不能直接完成/放弃（`cannot complete/abandon virtual task`）

---

## ADR-005: 双模式 IPC（Tauri 真实调用 + 浏览器 Mock）

### 状态
已采纳

### 背景
前端开发时不想每次都启动 Tauri 桌面应用，需要 Vite dev server 直接可跑。

### 决策

**`safeInvoke(command, args)`** — 唯一调用入口：

```typescript
// mock-invoke.ts
export async function safeInvoke<T>(command: string, args = {}): Promise<T> {
  if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
    // Tauri 模式：真实调用
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke(command, args);
  }
  // 浏览器模式：Mock 处理
  return handlers[command](args) as T;
}
```

**Mock 实现**（`mock-invoke.ts` 中 `handlers` 对象）：
- `STORAGE_TASKS`：localStorage 中的 MockTask JSON
- `STORAGE_FAITH`：localStorage 中的 MockFaithRecord JSON
- `STORAGE_USER`：localStorage 中的 MockUser JSON
- 虚拟任务 ID：`daily:{template_id}:{date}`
- 进程检测 Mock：恒返回 `false` / `[]`

### 理由
- **开发效率**：前端可以独立开发，不需要每次启动 Tauri
- **测试友好**：Playwright 可以直接浏览器模式截图
- **一致性**：Mock 实现了和 Tauri 后端相同的业务逻辑（信仰计算、任务 CRUD）

### 已知不一致
Mock 的 `LEVEL_THRESHOLDS` 使用旧的非 ×10 表，浏览器模式下等级与 Tauri 模式不一致。**不要修复**，优先级问题。

### 约束
- 前端必须通过 `safeInvoke` 通信，禁止直接调用 `invoke`
- 新增命令必须同时在 `commands.rs` 和 `mock-invoke.ts` 中实现

---

## ADR-006: 信仰阈值 v2.0 升级（全部 ×10）

### 状态
已采纳 | 2025-05-02

### 背景
v1 的信仰阈值太小（Lv15 = 109,500），用户积累过快，等级感不足。

### 决策

v2.0 阈值全部乘以 10：

| Lv | v1 阈值 | v2 阈值（×10） | 称号 |
|----|---------|---------------|------|
| 1 | 0 | 0 | 见习牛马 |
| 2 | 1,500 | 15,000 | 工位信徒 |
| 3 | 4,000 | 40,000 | 初级供奉者 |
| ... | ... | ... | ... |
| 15 | 109,500 | 1,095,000 | 牛马圣徒 |

### 理由
- **等级感**：从 Lv1 到 Lv15 需要真实积累，避免快速满级
- **仪式感**：更大的数字让「牛马圣徒」更具成就感
- **兼容性**：数据库存储的是累计值，不存在迁移问题，按需重新计算即可

### 约束
- 代码中的阈值常量必须是 ×10 后的值（`15_000`, `40_000`, `1_095_000`）
- 测试用例中必须验证 ×10 关系（`level.rs` 中 `all_thresholds_are_10x_v1` 测试）

---

## ADR-007: 护甲系统 v2（分等级段设定护甲值）

### 状态
已采纳 | 2025-05-02

### 背景
v1 护甲系统设计过于简单，需要更精细的保护机制。

### 决策

护甲计算规则（`calc_armor(current_level)`）：
```
Lv1:  0
Lv2-Lv5:   2,000
Lv6-Lv10:  4,000
Lv11-Lv15: 6,000
```

扣分规则：
1. 护甲优先：扣分先扣 `armor_points`
2. 护甲耗尽：才扣 `cumulative_faith`（可能导致降级）
3. 升级时：`armor_points` 重置为当前等级段的护甲值

### 理由
- **保护**：防止连续失误直接掉级，提供缓冲
- **进阶感**：等级越高，护甲越厚（Lv11+ 需要更多积累）
- **惩罚感**：护甲归零后仍继续扣分，形成真实压力

### 约束
- `armor_points` 存在 `users` 表，增量迁移添加（v2 前版本需要迁移）
- 护甲不跨等级继承，每次晋升重新计算

---

## ADR-008: 进程绑定实现任务自动启停

### 状态
已采纳

### 背景
用户希望绑定任务到特定应用（如 IDE），应用运行时任务自动开始，退出时自动暂停。

### 决策

**后端命令**：
- `is_process_running(app_name)` → `bool`
- `list_processes(app_name)` → `Vec<ProcessInfo>`（Windows `tasklist` 实现）

**前端实现**（`process-detector.ts`）：
```typescript
const POLL_INTERVAL_MS = 3000; // 3 秒轮询
// 对每个绑定了进程的任务：
//   - 检测到进程启动 → 自动 start_task
//   - 检测到进程关闭 → 自动 pause_task
```

### 理由
- **自动化**：减少手动操作，让任务计时更自然
- **准确性**：基于真实的应用使用时间来记录工作/学习时长
- **Windows 专用**：使用 `tasklist` 命令，无跨平台计划

### 约束
- 仅支持 Windows（`tasklist` 命令）
- 非 Windows 调用返回 `"Unsupported platform"` 错误

---

## ADR-009: 数据库路径优先级策略

### 状态
已采纳 | 2025-05-02

### 背景
需要同时兼容开发环境（exe 同目录）和生产环境（用户数据目录）。

### 决策

```
优先级 1: dirs::data_local_dir()/牛马信仰/niuma_faith.db  (若已存在)
优先级 2: exe 所在目录/niuma_faith.db                       (开发兼容)
```

### 实现逻辑

```rust
// main.rs
let db_path = if user_db_exists {
    user_data_dir / "niuma_faith.db"
} else {
    exe_dir / "niuma_faith.db"
};
```

### 理由
- **开发友好**：开发时数据库在 exe 旁，不需要清理用户目录
- **生产规范**：生产环境使用标准用户数据目录，符合操作系统约定
- **自动选择**：代码自动检测，无需配置

### 约束
- 用户数据目录优先：若用户目录下已有数据库，优先使用（保护用户数据）
- 首次启动时用户目录可能没有数据库，此时回退到 exe 目录

---

## ADR-010: 主窗口 + 悬浮窗双窗口模式

### 状态
已采纳

### 背景
用户需要实时看到当前等级/信仰，但又不想占用太多屏幕空间。

### 决策

| 窗口 | 标签 | 尺寸 | 特性 |
|------|------|------|------|
| 主窗口 | `main` | 900×700 | 普通窗口，可关闭/最小化 |
| 悬浮窗 | `floating` | 80×80 | always_on_top, decorations=false, skip_taskbar, transparent |

**托盘**：
- 左键单击：显示主窗口
- 右键菜单：显示主窗口 / 打开悬浮窗 / 退出

### 理由
- **轻量**：悬浮窗只显示圆形等级徽章，不干扰工作
- **双击**：悬浮窗双击打开主窗口，方便全功能操作
- **可隐藏**：主窗口关闭时最小化到托盘，悬浮窗仍在

### 约束
- 悬浮窗标签必须与 `tauri.conf.json` 中的 `floating` 窗口标签一致
- `open_floating_widget` / `close_floating_widget` 命令控制悬浮窗显隐

---

## 废弃决策

### 已废弃：单窗口模式（v1 之前）
原因：用户需要实时等级显示，单窗口无法满足。

### 已废弃：v1 信仰阈值（未 ×10）
原因：等级感不足，用户积累过快。

### 已废弃：v1 护甲（固定值）
原因：高级别保护不足，进阶感弱。

---

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2025-05-02 | v1.0 | 初始版本，ADR-001 ~ ADR-010 |