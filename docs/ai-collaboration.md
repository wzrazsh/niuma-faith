# 牛马信仰 — AI 协作规则

> 本文档是 AI 修改代码前必须读取的上下文控制文档。删除全部代码后，仅凭本文档 + design-doc.md + api-contract.md 可还原完整的协作边界。

## 1. 核心原则

### 1.1 文档优先

任何代码修改前，AI 必须先确认：
1. 相关文档是否已更新
2. 修改是否会破坏已有功能（参考 `api-contract.md` 的命令契约）
3. 是否涉及废弃设计（见本文档第 4 节）

### 1.2 双模式理解

项目有 **两套运行时**：
- **Tauri 模式**：真实桌面应用，Rust 后端 + SQLite
- **浏览器模式**：Vite dev server，localStorage Mock

修改前端 API 层时，必须同步更新 `mock-invoke.ts`（`frontend/src/api/mock-invoke.ts`），否则浏览器模式会断裂。

### 1.3 前后端边界

```
前端 (Vue 3 + TypeScript)
    ↓ safeInvoke() — 唯一出口
    ↓ Tauri IPC 或 Mock
后端 (Rust + Tauri v2)
```

**禁止**：
- 前端直接 import Rust 模块
- 后端直接操作 DOM
- 前端绕过 `safeInvoke` 直接调用 `invoke`

## 2. 文件读取优先级

修改代码前，AI 必须按以下顺序读取相关文档：

| 修改类型 | 必须读取的文档 |
|----------|---------------|
| 修改 UI 组件 | `ui-spec.md` → `design-doc.md` |
| 修改 API / 命令 | `api-contract.md` → `design-doc.md` |
| 修改数据库 | `data-model.md` → `design-doc.md` |
| 修改业务流程 | `workflows.md` → `design-doc.md` |
| 任何修改 | `docs/AGENTS.md`（文档索引） |

## 3. 开发命令规范

### 3.1 前端

```bash
# 开发
npm run dev

# 类型检查（必须通过才能 commit）
npm run build

# 生产构建
npm run tauri build
```

### 3.2 后端

```bash
# 类型检查
cargo check

# 测试
cargo test

# 构建
cargo build --release
```

### 3.3 全量测试

```bash
npm run tauri build
vue-tsc --noEmit
cargo test
```

## 4. 废弃设计（禁止复活）

以下设计已废弃，代码中不得恢复：

### 4.1 已废弃的命令参数

| 命令 | 废弃参数 | 原因 |
|------|----------|------|
| `update_task` (main.rs) | 不接受 `actual_minutes` 和 `status` | commands.rs 版本才是完整版 |
| `start_task` | 禁止对虚拟任务 ID（`daily:*`）直接操作 | 必须先物化 |

### 4.2 Mock 等级表说明

Mock 的 `LEVEL_THRESHOLDS` 在首次代码生成时必须使用 v2.0 ×10 阈值表（与后端一致）：
```
正确值: [0, 15000, 40000, 80000, 135000, 205000, 290000, 395000, 520000, 665000, 825000, 945000, 1025000, 1070000, 1095000]
```
已废弃的 v1 旧值 `[0, 100, 300, 600, ...]` 不得出现在 Mock 实现中。

### 4.3 已废弃的目录约定

- `frontend/src/components/` 下的组件直接放置，不使用子目录分组（除了 `kanban/`）
- 所有组件使用 `<script setup lang="ts">` 语法，禁止 Options API

## 5. 代码修改红线

### 5.1 禁止行为

- **禁止** 在 `main.rs` 和 `commands.rs` 之间制造功能差异——两者命令集必须一致
- **禁止** 修改 `domain/` 层的纯函数签名（这是无依赖的核心逻辑）
- **禁止** 在 `data/sqlite.rs` 之外的地方直接操作数据库连接
- **禁止** 修改 `design-doc.md` 中描述的表结构而不更新 `data-model.md`

### 5.2 必须同步的修改

任何以下修改必须同步更新对应文档：

| 修改内容 | 必须同步的文档 |
|----------|---------------|
| 新增 Tauri 命令 | `api-contract.md` |
| 修改数据库表/字段 | `data-model.md` |
| 新增/修改 UI 组件 | `ui-spec.md` |
| 修改业务流程 | `workflows.md` |
| 修改 API 契约 | `api-contract.md` |
| 修改构建流程 | `build-guide.md` |

## 6. 常见坑与注意事项

### 6.1 Tauri v2 特性

- 命令参数自动将 camelCase 转为 snake_case（`estimatedMinutes` → `estimated_minutes`）
- `tauri::generate_handler!` 中注册的命令才会暴露给前端
- 窗口标签必须与 `tauri.conf.json` 中的一致

### 6.2 SQLite 并发

- 所有数据库访问通过 `SqliteDb`（`Arc<Mutex<Connection>>`）进行
- 不要在 `MutexGuard` 之外持有数据库连接
- WAL 模式下读操作不阻塞写操作

### 6.3 虚拟任务系统

- 模板任务 `recurrence_kind = 'daily'` 且 `template_id IS NULL`
- 虚拟任务 ID 格式：`daily:{template_id}:{date}`
- 只有 `start_task` 会触发物化，其他操作（complete/abandon/delete）对虚拟任务直接返回错误

### 6.4 历史日期保护

- `is_historical(date)` = `date < today`
- 历史日期任务只能读取，不能修改/完成/放弃/删除

## 7. 新 AI 会话初始化

启动新的 AI 协作会话时，必须读取以下文件建立上下文：

```text
必读：
  1. docs/AGENTS.md           ← 文档索引，了解有哪些文档
  2. docs/vision.md            ← 产品愿景，保持方向一致
  3. docs/design-doc.md        ← 系统设计，单一事实来源
  4. docs/ai-collaboration.md  ← 本文档

按需读取：
  - api-contract.md            ← 修改命令时
  - data-model.md             ← 修改数据库时
  - ui-spec.md                ← 修改 UI 时
  - workflows.md              ← 修改流程时
```

## 8. 版本与更新

- 本文档版本：v1.0
- 最后更新：2026-05-05
- 更新时同步修改 `docs/AGENTS.md` 的 Key Files 表格