# 牛马信仰 — 测试计划

> 版本：2.1 | 更新日期：2026-05-02 | 测试范围：全栈

---

## 一、测试概览

| 层次 | 框架 | 状态 | 覆盖范围 |
|------|------|------|----------|
| Rust 单元测试 | `#[cfg(test)]` in-module | ✅ 141 passed | domain / data / application / tauri |
| 前端类型检查 | `vue-tsc --noEmit` | ✅ 0 errors | 类型安全 |
| 前端构建 | `vite build` | ✅ 通过 | 模块打包 |
| E2E 测试 | Playwright | 🔲 未配置 | 端到端 |
| 集成测试 | Tauri test binary | ⚠️ Windows 需 WebView2 | IPC 调用链 |
| 手动验收 | Chrome DevTools | 🔲 待执行 | UI / 交互 / 数据流 |

---

## 二、Rust 后端测试（当前 141 tests）

### 2.1 Domain 层 — 纯函数单元测试

| 模块 | 测试数 | 覆盖点 |
|------|--------|--------|
| `domain::faith` | 42 | `calc_survival` 9 档位边界；`calc_progress` 5 档位边界；`calc_discipline` 10 种组合；`calculate_daily` 7 种场景（空日/8h工作/8h+4h/满负载/边界） |
| `domain::level` | 17 | `get_level` 各阈值/超界；`progress_to_next` 各级；`interval_to_next`；满级；全称号验证 |
| `domain::task` | 9 | `calc_task_bonus` work/study/other 各分钟段（30/60/90/120min） |
| `domain::models` | 2 | 模型结构序列化等 |

**需补充的测试：**

- [ ] `domain::faith::calc_survival` — 负数输入处理
- [ ] `domain::faith::calc_discipline` — `leave_record` 非法值（如 99）
- [ ] `domain::level::LEVELS` — 确认 15 级 2.0 阈值全部 ×10 无误
- [ ] `domain::models::ProcessInfo` — JSON 序列化/反序列化往返

### 2.2 Data 层 — SQLite 持久化（共 15 项）

| 测试 | 覆盖点 |
|------|--------|
| `upsert_user` | 用户 upsert + 读取 |
| `upsert_daily_record_last_write_wins` | 同日多次写入取最新 |
| `cross_day_separate_records` | 跨天记录隔离 |
| `add_faith_updates_level` | 增加累计信仰触发等级更新 |
| ……(其余 11 项见 `src-tauri/src/data/sqlite.rs::tests`) | armor 字段、faith_transactions、task_sessions 等 |

**需补充的测试：**

- [ ] `armor` 字段读写 — upsert user 后验证 `armor` / `total_armor` 正确持久化
- [ ] `faith_transactions` 表 — 查询/插入交易记录
- [ ] `task_sessions` 表 — 会话创建/更新/完成生命周期
- [ ] 并发安全 — 多线程同时写入（Mutex 锁验证）
- [ ] Schema 迁移 — `ensure_column` 对 `armr` 列的幂等性

### 2.3 Application 层 — 业务逻辑（共 29 项 = faith_service 9 + ledger_service 5 + task_service 15）

| 测试 | 覆盖点 |
|------|--------|
| `upsert_daily_record_applies_delta` | 账本服务增量计算 |
| `pause_task_accumulates_minutes_to_daily_record` | 暂停任务累计分钟到日记录 |
| `check_in_*`(3 项) | 签到流程：无用户报错 / 新用户建档 / 同日重复覆盖 |
| `build_status_*`(3 项) | FaithStatus 构建：含 armor 字段 / 零 armor / progress_to_next 正确 |
| ……(其余 19 项分散于各 service) | 任务生命周期、信仰增量、跨天逻辑等 |

**需补充的测试：**

- [x] `faith_service::check_in` — 完整签到流程（空用户 / 已有记录 / 跨天）
- [x] `faith_service::build_status` — FaithStatus 构建（armor 字段 / level 正确）
- [x] `task_service::complete_task` — 完成任务触发 bonus faith
- [x] `task_service::abandon_task` — 放弃任务不触发 bonus
- [x] `task_service::delete_task` — 删除后的级联清理
- [ ] `task_service::is_historical` — 历史日期任务保护
- [ ] `ledger_service::upsert_daily_record` — 重复 upsert 幂等；减少时为 0；负值保护

### 2.4 Tauri 命令层（共 27 项）

| 测试 | 覆盖点 |
|------|--------|
| `get_or_create_user` | 新建用户 |
| `get_status_new_user` | 新用户状态查询 |
| `create_task` / `create_task_invalid_category` / `create_task_zero_estimated_minutes` / `create_task_with_no_user` | 创建任务的正常 + 错误路径 |
| `start_task` / `pause_task` / `resume_task` | 任务启停恢复 |
| `task_lifecycle_workflow` | 完整生命周期 |
| `complete_task_command` / `abandon_task_command` / `delete_task_command` | 完成 / 放弃 / 删除 |
| `start_nonexistent_task` / `complete_nonexistent_task` / `abandon_nonexistent_task` / `delete_nonexistent_task` | 不存在任务的错误路径 |
| `update_task_title` / `update_task_invalid_status` | 更新字段与非法 status |
| `check_in_normal` / `check_in_duplicate_overwrites` | 签到 + 重复签到覆盖 |
| `get_daily_stats_command` | 日统计 |
| `is_process_running_returns_bool` / `is_process_running_non_windows` | 进程检查跨平台 |
| `list_processes_returns_vec` / `list_processes_case_insensitive` / `list_processes_no_match` | 进程列表 |

**需补充的测试：**

- [x] `check_in` 命令 — 正常签到 / 重复签到
- [x] `get_daily_stats` 命令 — 日统计
- [x] `update_task` — 更新标题/描述/预估时间
- [x] `complete_task` / `abandon_task` / `delete_task`
- [x] 主要命令的错误路径 — 无效参数 / 不存在的任务 / 不存在的用户
- [x] `is_process_running` — 非 Windows 平台错误处理

---

## 三、前端测试

### 3.1 组件渲染测试（Playwright）

需为以下组件编写基本渲染验证：

| 组件 | 验证点 |
|------|--------|
| `Dashboard.vue` | 日历侧栏 + 任务区正确渲染；StatusPanel / FaithDashboard / DailyGoalPanel 子组件都加载 |
| `CalendarView.vue` | 月视图默认显示；点击日期切换；任务 dot 指示器 |
| `TaskList.vue` | 任务列表渲染；创建按钮可用 |
| `TaskForm.vue` | 表单输入/提交；验证错误提示 |
| `FaithDashboard.vue` | 信仰值正确显示；每日上限进度条渲染；"暂无记录"空态 |
| `StatusPanel.vue` | 等级徽章显示；等级进度条正确；护甲进度条显示 |
| `DailyGoalPanel.vue` | 工作/学习进度条正确；任务加成标签 |
| `KanbanBoard.vue` | 四列默认渲染；创建任务/添加列按钮；拖拽区域 |
| `KanbanCard.vue` | 标题/分类/预估时间显示；操作按钮（开始/暂停/完成/删除）；进程绑定表单 |
| `KanbanColumn.vue` | 列标题 + 计数；空态 "暂无任务" |
| `KanbanCardForm.vue` | 编辑/新建表单 |
| `KanbanPage.vue` | 完整看板页面加载 |
| `FloatingWidget.vue` | 悬浮窗渲染（Tauri 环境） |

### 3.2 前端交互测试（Playwright）

| 场景 | 操作 | 预期 |
|------|------|------|
| 任务 CRUD | 创建 → 编辑 → 完成 → 删除 | 列表实时更新 |
| 看板拖拽 | 将卡片从"待办"拖到"进行中" | 卡片移动且持久化 |
| 看板计时器 | 开始任务 → 等待 3 秒 → 查看计时器数值 | 计时器递增 |
| 进程绑定 | 打开绑定表单 → 输入进程名 → 勾选自动开始 → 确定 | 绑定 UI 更新 |
| 进程解绑 | 点击解绑按钮 | 绑定 UI 消失 |
| 日历切换 | 月→周→日 视图切换 | 视图正确渲染 |
| 历史保护 | 选择历史日期 → 尝试编辑/删除任务 | 操作被阻止或按钮禁用 |
| 信仰仪表盘 | 提交任务后查看信仰值变化 | 值正确更新 |

### 3.3 前端状态管理测试

| Store | 验证点 |
|-------|--------|
| `faith.ts` | `init()` 获取用户和状态；`fetchStatus()` 更新 FaithStatus；`todayFaith` 计算正确；`currentLevel` / `progressToNext` 派生值 |
| `task.ts` | `fetchTasksByDate` 按日期过滤；`createTask` 后列表更新；`updateTask` 部分字段更新；`completeTask` 状态变更 |
| `kanban.ts` | `loadBoardConfig` 默认四列；`moveCard` 列间移动；`startTimer` / `stopTimer` 计时；`addColumn` 自定义列 |

### 3.4 前端服务层测试

| 服务 | 验证点 |
|------|--------|
| `kanban-api.ts` | `getBoardConfig` 默认配置；`saveBoardConfig` 持久化；`moveCard` 列间移动算法；`bindProcess` / `unbindProcess` |
| `process-detector.ts` | `isRunning` mock 返回 false；`listProcesses` mock 返回 []；`startPolling` 注册/清理 interval |
| `reminder-service.ts` | 添加/移除提醒；未触发逻辑验证 |

### 3.5 前端类型问题（已清零）

`vue-tsc --noEmit` 当前 0 errors。历史的 4 类问题（mock-invoke 未使用变量、task.ts invoke 导入、activeTasks、KanbanBoard/Card 的 `'active'` 字面量）均已修复或显式 `as TaskStatus` 收编。后续如新增类型问题需在此节登记。

---

## 四、集成测试场景

### 4.1 信仰计算端到端

```
初始化 → 创建 Work 任务 480min → 创建 Study 任务 240min
→ 完成任务 → 验证 FaithStatus.total_faith = 800
→ 验证 armor 字段存在且 >= 0
→ 验证 level 正确计算
```

### 4.2 等级升级端到端

```
初始 Lv1 (cumulative=0) → 多次完成满负载日
→ 累计信仰达到 Lv2 阈值 (15,000) → 验证自动升级到 Lv2
→ 验证 progress_to_next 更新
→ 验证 level_title 更新为 "工位信徒"
```

### 4.3 看板进程绑定端到端（Windows 环境）

```
创建任务 → 添加到看板 → 绑定进程 "notepad.exe"
→ 启动记事本 → 验证 3 秒内任务自动开始
→ 关闭记事本 → 验证 3 秒内任务自动暂停
→ 解绑进程 → 验证不再自动控制
```

### 4.4 历史保护端到端

```
选择昨天日期 → 查看昨天创建的任务
→ 尝试编辑 → 操作被阻止
→ 尝试删除 → 操作被阻止
→ 尝试完成 → 操作被阻止
```

### 4.5 多日连续性

```
Day 1: 签到 → 完成 8h work → 累计信仰 +600
Day 2: 签到 → 完成 8h work + 8h study → 累计信仰 +1000
→ 验证累计信仰 = 1600（跨日累加正确）
→ 验证 DailyRecord 按日隔离
```

---

## 五、性能测试

| 场景 | 指标 | 目标 |
|------|------|------|
| 页面首次加载 | FCP < 1s | 桌面应用标准 |
| 日历月视图切换 | < 100ms | 即时响应 |
| 看板拖拽操作 | < 50ms | 即拖即放 |
| 进程轮询 | 3s 间隔 CPU < 1% | 后台静默 |
| SQLite 写入 | < 10ms | 单条记录 |
| 大量任务列表 | 1000 任务渲染 < 500ms | 长列表优化 |

---

## 六、安全测试

| 场景 | 验证点 |
|------|--------|
| SQL 注入 | 任务标题/描述包含 SQL 关键词 → rusqlite 参数化查询 |
| XSS | 任务标题包含 `<script>` → Vue 模板自动转义 |
| 输入验证 | 非法 TaskCategory → 后端拒绝 |
| 数值溢出 | 累计信仰 > i64::MAX → 数据库约束 |
| 历史篡改 | 通过 IPC 修改历史日期的 DailyRecord → 后端日期保护 |

---

## 七、平台兼容性

| 平台 | 状态 | 备注 |
|------|------|------|
| Windows 10/11 | ✅ 主要目标 | 进程检测仅 Windows 支持 |
| macOS | ⚠️ 未测试 | 进程检测返回错误（预期行为） |
| Linux | ⚠️ 未测试 | 进程检测返回错误（预期行为） |

---

## 八、回归测试清单（每次发布前执行）

- [ ] `cargo test` — 所有 141 项 Rust 测试通过
- [ ] `npx vite build` — 前端构建成功
- [ ] 手动启动 Tauri — 主窗口 + 悬浮窗 + 系统托盘正常
- [ ] 创建 Work 任务 480min → 完成 → 验证生存信仰 = 400
- [ ] 创建 Study 任务 480min → 完成 → 验证精进信仰 = 400
- [ ] 验证等级进度条正确更新
- [ ] 验证护甲显示正确
- [ ] 看板拖拽卡片正常
- [ ] 看板计时器正常
- [ ] 进程绑定功能（Windows）
- [ ] 日历月/周/日视图切换
- [ ] 历史日期任务只读
- [ ] 应用正常退出

---

## 九、测试优先级排序

### P0 — 发布阻塞（每次提交前）
1. `cargo test` 全部通过
2. `vite build` 成功
3. 信仰计算值正确（manual smoke: survival/progress/discipline）

### P1 — 功能完整性
1. 新需求测试覆盖（armor 字段、进程绑定）
2. 边界条件测试（零值、负值、超限）
3. 错误路径测试

### P2 — 质量保证
1. Playwright 组件渲染测试
2. Playwright 交互流程测试
3. 性能测试

### P3 — 长期维护
1. 前端类型错误修复
2. 平台兼容性测试（macOS / Linux）
3. 压力测试（大量数据 / 长时间运行）
