# 牛马信仰 — 核心业务流程

> 本文档描述系统的关键业务流程，按步骤详细说明。删除代码后，仅凭本文档 + design-doc.md 可还原业务逻辑。

## 1. 每日打卡流程

### 触发
用户在前端 Dashboard 页面填写今日工作/学习时长和纪律表现，点击「打卡」。

### 流程

```
[前端] Dashboard.vue / DailyGoalPanel.vue
    ↓
[Store] faith.ts checkIn(workMinutes, studyMinutes, discipline)
    ↓
[API] tauri.ts invoke_check_in(workMinutes, studyMinutes, breakCount, leaveRecord, closeRecord)
    ↓  Tauri IPC
[后端] commands.rs check_in()
    ↓
[Service] faith_service.rs check_in(user_id, work, study, discipline)
    ↓
[Domain] calculate_daily(work, study, discipline) → FaithBreakdown
    ├─ calc_survival(work) → survival_faith (0/100/200/300/400)
    ├─ calc_progress(study) → progress_faith (同上)
    └─ calc_discipline(discipline) → discipline_faith + a/b/c
    ↓
[Service] ledger_service.rs upsert_daily_record(..., breakdown, now_ts)
    1. 查旧记录（同 user_id + date）
    2. 计算新 total_faith = breakdown.total()
    3. delta = new_total - old_total（旧记录存在时），否则 = new_total
    4. INSERT ... ON CONFLICT(user_id, date) DO UPDATE
       写入 daily_records（覆盖 work_minutes, study_minutes, 各 faith 字段, discipline 子项）
    5. UserRepo::add_faith(user_id, delta)
       ├─ UPDATE users SET cumulative_faith += delta
       ├─ 重新查询 cumulative_faith → get_level() → 更新 current_level
       └─ 若升级，按 calc_armor(new_level) 更新 armor_points
    6. INSERT faith_transactions (delta, kind='check_in', ts=now)
    ↓
[后端] 返回 FaithStatus
    ↓
[前端] faith.ts 更新 state.faithStatus / state.todayRecord
    ↓
[前端] StatusPanel.vue / FaithDashboard.vue 自动刷新显示
```

### 边界
- **重复打卡**: 同天第二次打卡 → `ON CONFLICT DO UPDATE` 覆盖，delta 按差值计算
- **负 delta**: 若新打卡总信仰低于旧值 → delta 为负，累计信仰扣减（但 armor 优先抵扣）
- **升级触发**: `UserRepo::add_faith` 内部自动检测并更新 `current_level`

---

## 2. 任务创建流程

### 触发
用户在 TaskList 点击「新建任务」，或在 KanbanBoard 点击列头的「+」。

### 流程

```
[前端] TaskForm.vue / KanbanCardForm.vue
    用户输入：标题、描述、分类(work/study/other)、预计时长、日期、是否每日重复
    ↓
[Store] task.ts createTask(payload)
    ↓
[API] task.ts invoke_create_task(title, desc, category, estimatedMinutes, date, recurrenceKind)
    ↓  Tauri IPC
[后端] commands.rs create_task()
    验证：category ∈ {work,study,other}，estimated_minutes > 0
    ↓
[Service] task_service.rs create_task(user_id, title, desc, cat, estimated, date, rec)
    1. 生成 UUID 作为 id
    2. date = date.unwrap_or(今天)
    3. status = Paused（默认）
    4. INSERT tasks
    ↓
[后端] 返回 Task
    ↓
[前端] task.ts 将新任务加入 tasks 数组
    ↓
[前端] TaskList.vue / KanbanBoard.vue 渲染新任务
```

### 边界
- `estimated_minutes` 必须 > 0，否则报错
- `date` 为空则使用今天日期
- 新任务默认状态为 `Paused`

---

## 3. 任务计时生命周期

### 3.1 开始计时

```
[前端] TaskList.vue / KanbanCard.vue 点击「开始」
    ↓
[Store] task.ts startTask(id)
    ↓
[API] task.ts invoke_start_task(id)
    ↓  Tauri IPC
[后端] commands.rs start_task()
    ↓
[Service] task_service.rs start_task(id)
    1. 若 id 以 "daily:" 开头 → materialize_if_virtual(id)
       ├─ 解析 template_id 和 date
       ├─ 检查是否已有实例（同 template_id + date）
       ├─ 有 → 返回实例 id；无 → INSERT 真实行，返回新 id
       └─ 后续操作使用真实 id
    2. 验证任务状态（不能是 Completed/Abandoned）
    3. UPDATE tasks SET status=Running, started_at=now
    4. TaskSessionRepo::start_session(task_id, now_ts)
       └─ INSERT task_sessions (task_id, start_ts, end_ts=NULL, seconds=0)
    ↓
[后端] 返回更新后的 Task
    ↓
[前端] task.ts 更新任务状态为 running
[前端] kanban.ts startTimer(id) → setInterval 每秒更新 UI 计时显示
```

### 3.2 暂停计时

```
[前端] 点击「暂停」
    ↓
[API] invoke_pause_task(id)
    ↓  Tauri IPC
[后端] task_service.rs pause_task(id)
    1. TaskSessionRepo::end_open_session(task_id)
       ├─ SELECT * FROM task_sessions WHERE task_id=? AND end_ts IS NULL
       ├─ 计算 seconds = now - start_ts 的秒数差
       └─ UPDATE SET end_ts=now, seconds=?
    2. UPDATE tasks SET duration_seconds += seconds
    3. 将 seconds 换算为 minutes（seconds / 60，整数）
    4. 根据 task.category：
       ├─ Work → daily_records.work_minutes += minutes
       └─ Study → daily_records.study_minutes += minutes
    5. LedgerService::upsert_daily_record() → 重新计算当日信仰
       ├─ 查旧记录 → 加 minutes → calculate_daily → 新 breakdown
       ├─ delta = new_total - old_total
       ├─ UPDATE daily_records
       ├─ UserRepo::add_faith(delta) → 更新累计信仰/等级
       └─ INSERT faith_transactions
    6. UPDATE tasks SET status=Paused
    ↓
[后端] 返回 Task
    ↓
[前端] kanban.ts stopTimer(id) → 清除 setInterval
[前端] 刷新 FaithStatus 显示最新信仰
```

### 3.3 恢复计时

```
[前端] 点击「继续」
    ↓
[API] invoke_resume_task(id)
    ↓  Tauri IPC
[后端] task_service.rs resume_task(id)
    1. UPDATE tasks SET status=Running, started_at=now
    2. TaskSessionRepo::start_session(task_id, now_ts) → 插入新 session
    ↓
[前端] kanban.ts startTimer(id) → 重新启动 setInterval
```

### 3.4 结束任务（无奖励）

```
[前端] 点击「结束」
    ↓
[API] invoke_end_task(id)
    ↓  Tauri IPC
[后端] task_service.rs end_task(id)
    同 pause_task 步骤 1-5（关闭 session，累加时长，更新日信仰）
    6. UPDATE tasks SET status=Completed
    （注意：不调用 apply_task_bonus，不发任务奖励）
    ↓
[前端] 刷新状态
```

### 3.5 完成任务（有奖励）

```
[前端] 点击「完成」→ 弹窗确认实际用时 → 确认
    ↓
[API] invoke_complete_task(id, actualMinutes)
    ↓  Tauri IPC
[后端] task_service.rs complete_task(id, actual_minutes)
    1. 验证任务非历史日期
    2. 若 Running → 先关闭 session（同 pause 步骤 1-5）
    3. calc_task_bonus(category, actual_minutes) → bonus
       ├─ Work/Study: 每小时 5 分，最少 1 小时
       └─ Other: 每小时 2 分，最少 1 小时
    4. UPDATE tasks SET status=Completed, actual_minutes, completed_at=now
    5. apply_task_bonus(user_id, date, category, bonus, now_ts)
       ├─ 查 daily_records
       ├─ 更新 task_bonus_work 或 task_bonus_study
       ├─ 新 total = survival + progress + discipline + task_bonus
       ├─ delta = task_bonus（因为前面 pause 已经更新了基础信仰）
       ├─ UPDATE daily_records（tasks_completed += 1）
       ├─ UserRepo::add_faith(delta) → 更新累计信仰
       └─ INSERT faith_transactions (delta, kind='task_bonus')
    ↓
[后端] 返回 TaskCompleteResult { task, bonus_faith, bonus_category }
    ↓
[前端] 显示奖励弹窗（+{bonus_faith} 信仰）
[前端] 刷新 FaithStatus
```

---

## 4. 每日重复任务流程

### 4.1 创建模板

```
[前端] TaskForm.vue 勾选「每日执行」
    ↓
[API] invoke_create_task(..., recurrenceKind: 'daily')
    ↓
[后端] task_service.rs create_task(..., RecurrenceKind::Daily)
    INSERT tasks (..., recurrence_kind='daily', template_id=NULL)
```

### 4.2 查询合成虚拟实例

```
[前端] CalendarView.vue 选择未来某一天 / Dashboard 加载当日
    ↓
[API] invoke_get_tasks_by_date(date, status=null)
    ↓
[后端] task_service.rs get_tasks_by_date(user_id, date, status_filter)
    1. 查询当日真实任务
    2. IF date >= 今天 AND (status_filter IS NULL OR status_filter=Paused):
       查询该用户的所有 daily 模板
       FOR each template:
         IF template.date == date: SKIP（模板自身已在这天）
         IF 已存在实例（同 template_id + date）: SKIP
         合成虚拟任务:
           id = "daily:{template_id}:{date}"
           status = Paused, recurrence_kind = None
           template_id = {template_id}
    3. 返回 [真实任务] + [虚拟任务]
```

### 4.3 虚拟任务物化

```
[前端] 对虚拟任务执行「开始」「暂停」「完成」等操作
    ↓
[后端] task_service.rs start_task("daily:tpl-xxx:2026-05-10")
    1. materialize_if_virtual(id)
       ├─ 解析 template_id = "tpl-xxx", date = "2026-05-10"
       ├─ 查是否已有实例：SELECT * FROM tasks WHERE template_id=? AND date=?
       ├─ 有 → 返回实例 id
       └─ 无 → INSERT 真实行（复制模板字段，id=新UUID, template_id=tpl-xxx, date=2026-05-10, recurrence_kind=None）
    2. 用真实 id 继续后续操作
```

### 4.4 取消重复

```
[前端] 编辑任务，取消「每日执行」
    ↓
[API] invoke_set_task_recurrence(id, 'none')
    ↓
[后端] task_service.rs set_task_recurrence(id, None)
    UPDATE tasks SET recurrence_kind='none' WHERE id=?
    （模板变为普通任务，不再生成虚拟实例）
```

---

## 5. 看板操作流程

### 5.1 加载看板

```
[前端] KanbanBoard.vue onMounted
    ↓
[Store] kanban.ts loadBoard()
    1. kanban-api.ts loadConfig() → localStorage.getItem('kanban-board-config')
       ├─ 有 → JSON.parse 为 BoardConfig
       └─ 无 → 创建默认列：
           col1: {id:'todo', title:'待办', order:0, taskIds:[], isCustom:false}
           col2: {id:'inprogress', title:'进行中', order:1, ...}
           col3: {id:'paused', title:'暂停中', order:2, ...}
           col4: {id:'done', title:'已完成', order:3, ...}
    2. task.ts loadTasksByDate(今天)
    3. 将 Task[] 按 status 映射到列：
       ├─ Paused → 待办（若未指定列）
       ├─ Running → 进行中
       ├─ Paused（之前移过来的）→ 暂停中
       └─ Completed/Abandoned → 已完成
    4. 为每个任务创建 KanbanCard，存入 cards Map
    5. 恢复 activeTimers（Running 任务启动 setInterval）
[UI] KanbanColumn 按 category 分组渲染泳道
    columnSwimlanes(columnId) → SwimlaneGroup[]
    └─ 每列内按 work/study/other 显示 工作/学习/其他 分组，空组自动隐藏
```

### 5.2 拖拽移动卡片

```
[前端] KanbanCard.vue dragstart → 记录 draggedCardId
[前端] KanbanColumn.vue @drop
    ↓
[Store] kanban.ts moveCard(cardId, targetColumnId, targetIndex)
    1. 查卡片旧列
    2. 从旧列 taskIds 中移除
    3. 插入新列 taskIds 的 targetIndex 位置
    4. 更新 card.columnId = targetColumnId
    5. 更新 card.orderInColumn（重新计算该列所有卡片顺序）
    6. kanban-api.ts saveConfig({columns}) → localStorage
    ↓
[前端] 重新渲染看板
```

### 5.3 进程绑定自动启停

```
[前端] KanbanCard.vue / KanbanCardForm.vue 设置进程绑定
    输入应用名（如 "notepad.exe"），勾选 autoStart / autoPause
    ↓
[Service] process-detector.ts
    1. 将绑定信息存入 KanbanCard.processBinding
    2. kanban-api.ts 持久化（随 BoardConfig 一起存）
    3. 启动轮询：setInterval(3000ms)
       ├─ 调用 invoke_is_process_running(appName)
       ├─ IF 进程存在 AND 之前不存在 AND autoStart:
       │   → kanban.ts startTask(card.task.id)
       └─ IF 进程不存在 AND 之前存在 AND autoPause:
           → kanban.ts pauseTask(card.task.id)
```

---

## 6. 升级流程

### 触发
累计信仰达到下一级阈值时自动升级。

### 流程

```
[后端] UserRepo::add_faith(user_id, delta)
    1. BEGIN TRANSACTION
    2. UPDATE users SET cumulative_faith += delta WHERE id=?
    3. SELECT cumulative_faith FROM users WHERE id=?
    4. new_level = get_level(cumulative_faith).level
    5. IF new_level > old_level:
         UPDATE users SET current_level = new_level
         UPDATE users SET armor_points = calc_armor(new_level)
    6. COMMIT
```

### 护甲规则

```
Lv1:  0
Lv2-Lv5:   2,000
Lv6-Lv10:  4,000
Lv11-Lv15: 6,000
```

---

## 7. 降级流程（v2.0）

### 触发
违规扣分导致累计信仰低于当前等级阈值。

### 流程

```
[后端] UserRepo::deduct_faith(user_id, penalty)  // 假设实现
    1. IF armor_points >= penalty:
         armor_points -= penalty
       ELSE:
         remaining = penalty - armor_points
         armor_points = 0
         cumulative_faith -= remaining
    2. 重新 get_level(cumulative_faith)
    3. IF new_level < current_level:
         current_level = new_level
         // 降级不回收已获得的护甲（或按设计决定）
```

**注意**: 当前代码中降级逻辑在 `UserRepo::add_faith` 中已支持（delta 可为负），但完整的即时扣分/日结算扣分/周期性扣分机制尚未完全实现（见需求文档）。

---

## 8. 悬浮窗流程

### 8.1 打开悬浮窗

```
[触发] 系统托盘菜单「打开悬浮窗」/ 双击托盘图标
    ↓
[后端] open_floating_widget()
    IF floating 窗口已存在:
        window.show() + set_focus()
    ELSE:
        WebviewWindowBuilder::new("floating", WebviewUrl::App("/?f=1".into()))
            .inner_size(80.0, 80.0)
            .always_on_top(true)
            .decorations(false)
            .skip_taskbar(true)
            .transparent(true)
            .shadow(false)
            .build()
        window.set_size(PhysicalSize::new(80, 80))
    ↓
[前端] FloatingWidget.vue 加载
    显示圆形等级徽章，支持鼠标拖拽
    双击 → invoke_show_main_window()
```

> **注意**: Tauri 使用 `/?f=1` 查询参数而非 `/#/floating` hash 路由创建悬浮窗，因 Tauri WebView 不完全支持 URL 中的 `#` 字符。`index.html` 中的内联脚本在 Vue 启动前自动将 `?f=1` 转换为 `#/floating`。

### 8.2 关闭悬浮窗

```
[触发] 托盘菜单「退出」/ 主窗口关闭时
    ↓
[后端] close_floating_widget() → window.hide()
```

---

## 9. 历史保护流程

### 规则
过去日期的任务为只读，禁止以下操作：
- update_task（修改字段）
- complete_task（完成）
- abandon_task（放弃）
- delete_task（删除）

### 实现

```rust
// task_service.rs
fn is_historical(date: &str) -> bool {
    date < today_str()   // 字符串比较，YYYY-MM-DD 格式可直接比较
}

// 在 update/complete/abandon/delete 前检查
if is_historical(&task.date) {
    return Err("cannot modify historical task".into());
}
```

---

## 10. 项目任务远程推送流程

项目任务由开发工具通过本地 HTTP API 主动推送，不是用户手动创建。

### 10.1 创建项目任务

```
[开发工具] Claude/Codex/OpenCode 启动工作会话
    1. 读取 {data_dir}/牛马信仰/http_port.txt 获取端口
    2. 读取 {data_dir}/牛马信仰/http_token.txt 获取 Token
    3. 生成 session_id (UUID)
    4. 根据上下文生成 title
    ↓
[HTTP] POST http://127.0.0.1:{port}/api/tasks
    Headers: Authorization: Bearer {token}
    Body: { action:"create", tool_name:"claude", session_id:"uuid", title:"重构用户认证模块" }
    ↓
[后端] local_server.rs 路由 → TaskService
    1. 验证 Token
    2. 解析 JSON body
    3. check session_id 去重：SELECT * FROM tasks WHERE tool_session_id=?
       ├─ 已存在 → 返回 409 { error: "session already exists", task_id: "..." }
       └─ 不存在 → 继续
    4. 生成 UUID 作为 task_id
    5. INSERT tasks (id, user_id, date=今天, title, category='work',
       task_type='project', source_tool=tool_name,
       tool_session_id=session_id, status='running', started_at=now)
    6. INSERT task_sessions (task_id, start_ts=now, end_ts=NULL, seconds=0)
    7. 返回 201 { task_id, session_id, status: "running", created_at }
    ↓
[前端] 自动（非本次请求触发）：看板下次刷新时显示新的项目任务
    项目任务在 UI 中只读，不可编辑/删除/手动完成
```

### 10.2 更新项目任务状态

```
[开发工具] 用户暂停工具 / 恢复工作
    ↓
[HTTP] PUT http://127.0.0.1:{port}/api/tasks/{session_id}
    Body: { action:"update", status:"paused" }
    ↓
[后端] local_server.rs → TaskService
    1. 查 task：SELECT * FROM tasks WHERE tool_session_id=?
    2. 验证 task_type = 'project'
    3. 验证当前 status 为 running 或 paused
    4. IF status == "paused":
         ├─ TaskSessionRepo::end_open_session(task_id) → 计算 seconds
         ├─ UPDATE tasks SET duration_seconds += seconds, status='paused'
         ├─ 将 minutes 累加到 daily_records.work_minutes
         └─ LedgerService::upsert_daily_record() → 重新计算当日信仰
       IF status == "running":
         ├─ UPDATE tasks SET status='running', started_at=now
         └─ TaskSessionRepo::start_session(task_id, now_ts)
    5. 返回 200 { task_id, session_id, status }
```

### 10.3 完成项目任务

```
[开发工具] 工作会话结束
    ↓
[HTTP] POST http://127.0.0.1:{port}/api/tasks/{session_id}/complete
    Body: { title:"...", summary:"完成了JWT认证重构" }
    ↓
[后端] local_server.rs → TaskService
    1. 查 task：SELECT * FROM tasks WHERE tool_session_id=?
    2. 验证 task_type = 'project' 且 status IN ('running', 'paused')
    3. 若 status=running → 关闭当前 session（同 pause 逻辑）
    4. 计算总 duration_seconds（所有 sessions 之和）
    5. work_minutes = duration_seconds / 60
    6. 更新 daily_records.work_minutes += work_minutes
    7. LedgerService::upsert_daily_record() → 重新计算当日信仰
       ├─ survival_faith 可能因 work_minutes 达到阶梯（120→100, 240→200, 360→300, 480→400）
       ├─ delta = new_total - old_total
       ├─ UserRepo::add_faith(delta)
       └─ INSERT faith_transactions (delta, kind='project_task', ref_id=task_id)
    8. UPDATE tasks SET status='completed', completed_at=now,
       actual_minutes=work_minutes, ai_summary=summary（若有）, updated_at=now
    9. 返回 200 { task_id, session_id, status:"completed",
       duration_minutes: work_minutes, faith_contributed: delta }
```

### 10.4 放弃项目任务

```
[开发工具] 会话异常终止
    ↓
[HTTP] POST .../api/tasks/{session_id}/abandon
    ↓
[后端]
    1. 若 status=running → 关闭 session，结算已工作时长（同 pause）
    2. UPDATE tasks SET status='abandoned'
    3. 已结算的 minutes 不回收
```

### 10.5 项目任务保护规则

```
[前端] 用户尝试编辑/完成/放弃/删除项目任务
    ↓
[API] update_task / complete_task / abandon_task / delete_task
    ↓
[后端] TaskService 检查：
    IF task.task_type == 'project':
        RETURN Err("project task cannot be modified via UI")
```

项目任务仅能通过 HTTP API（来自其 source_tool）修改状态和内容。

---

## 11. 首屏初始化流程

```
[应用启动]
    ↓
[后端] main()
    1. 初始化 tracing 日志
    2. 打开 SQLite 数据库（自动 init_schema + 迁移）
    3. 创建 AppState → FaithService + TaskService
    4. FaithService::get_or_create_user() → 确保 default_user 存在
    5. 分配随机端口，启动 Local HTTP Server (127.0.0.1:{port})
    6. 写入 http_port.txt 和 http_token.txt 到 data 目录
    7. 创建主窗口（900×700）
    8. 创建系统托盘
    ↓
[前端] App.vue mounted
    IF 非悬浮窗路由:
        faith.ts init()
        ├─ invoke_get_or_create_user()
        ├─ invoke_get_status() → 更新 faithStatus
        └─ invoke_get_today_record() → 更新 todayRecord
        task.ts loadTasksByDate(今天)
        kanban.ts loadBoard()
    ↓
[前端] 渲染 Dashboard / KanbanPage
```
