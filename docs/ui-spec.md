# 牛马信仰 — UI 组件与交互规范

> 本文档精确描述前端所有组件、页面布局、交互行为和样式规范。删除代码后，仅凭本文档 + design-doc.md 可还原全部 Vue 组件。

## 1. 页面路由

```
/           → Dashboard.vue      (主仪表盘)
/kanban     → KanbanPage.vue     (任务看板)
/floating   → FloatingWidget.vue (悬浮窗，无边框)
*           → 重定向到 /
```

路由模式：Hash 模式 (`createWebHashHistory`)

## 2. 全局布局

### 2.1 App.vue（根组件）

```
┌─────────────────────────────────────────┐
│  <nav> 仪表盘 | 任务看板                 │  ← 仅在非 /floating 路由显示
├─────────────────────────────────────────┤
│                                         │
│           <router-view />               │
│                                         │
└─────────────────────────────────────────┘
```

**导航栏样式**:
- 背景: `var(--color-surface)` (#222233)
- 底部边框: `1px solid var(--color-border)`
- padding: `8px 16px`
- 链接间距: `16px`
- 链接样式: 圆角 6px, 字号 0.875rem, 颜色 `var(--color-text-muted)`
- 悬停: 背景 `var(--color-bg)`, 颜色 `var(--color-text)`
- 激活: 背景 `var(--color-primary)` (#ffd700), 颜色 `#1a1a24`, 字重 600

### 2.2 CSS 主题变量（style.css）

```css
:root {
  --color-bg: #1a1a24;
  --color-surface: #222233;
  --color-surface-hover: #2a2a3e;
  --color-border: #333344;
  --color-text: #e0e0e0;
  --color-text-muted: #888899;
  --color-primary: #ffd700;        /* 金色 — 等级/信仰主题色 */
  --color-primary-dim: #b8860b;
  --color-success: #4ade80;        /* 绿色 — 成功/完成 */
  --color-danger: #ef4444;         /* 红色 — 删除/危险 */
}
```

全局背景: `#1a1a24`, 文字: `#e0e0e0`, 字体栈: 系统默认无衬线

---

## 3. 仪表盘页面（Dashboard.vue）

### 3.1 布局

```
┌──────────────────────────────────────────────────────────────┐
│  Dashboard                                                   │
│  ┌─────────────────┬───────────────────────────────────────┐ │
│  │                 │                                       │ │
│  │  CalendarView   │         TaskList                      │ │
│  │  (日历)         │         (任务列表 + 筛选 + 操作)      │ │
│  │                 │                                       │ │
│  ├─────────────────┤                                       │ │
│  │  FaithDashboard │                                       │ │
│  │  (今日信仰汇总) │                                       │ │
│  ├─────────────────┤                                       │ │
│  │  StatusPanel    │                                       │ │
│  │  (等级/护甲)    │                                       │ │
│  ├─────────────────┤                                       │ │
│  │  DailyGoalPanel │                                       │ │
│  │  (每日目标)     │                                       │ │
│  └─────────────────┴───────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

- 左侧边栏: 约 320px，包含日历 + 信仰面板
- 右侧主区: 弹性填充，任务列表

### 3.2 CalendarView.vue

**功能**: 月视图日历（支持周/日切换），日期选择，今日高亮

**Props**: 无（内部使用 `new Date()`）

**状态**:
- `currentMonth: Date` — 当前显示的月份
- `selectedDate: string | null` — 选中的日期 `YYYY-MM-DD`

**事件**:
- `@select(date: string)` — 用户点击某一天，触发 task store 加载该日任务

**样式**:
- 背景: `var(--color-surface)`
- 今日高亮: 边框或背景使用 `var(--color-primary)`
- 选中日期: 背景 `var(--color-surface-hover)`

### 3.3 FaithDashboard.vue

**功能**: 展示今日信仰的 survival / progress / discipline breakdown

**数据**: `faithStatus.today` (DailyRecord)

**展示**:
```
今日信仰: {total_faith} / 1000
├─ 生存信仰 {survival_faith}  (工作 {work_minutes} 分钟)
├─ 精进信仰 {progress_faith}  (学习 {study_minutes} 分钟)
└─ 戒律信仰 {discipline_faith} (专注{discipline_a}/离岗{discipline_b}/闭环{discipline_c})

已完成任务: {tasks_completed}
```

### 3.4 StatusPanel.vue

**功能**: 等级状态总览

**展示**:
```
┌─────────────┐
│   Lv{level} │   ← 圆形徽章
│  {title}    │
└─────────────┘

升级进度: {progress_to_next} / {interval}
[==========>    ] 进度条

护甲: {armor} / {total_armor}
[████████░░░░░░] 护甲条

今日明细:
- 生存: {survival_faith}
- 精进: {progress_faith}
- 戒律: {discipline_faith}
```

**满级特殊处理**:
- `next_threshold === null` 时显示「已达最高等级」
- 进度条显示 100%

### 3.5 DailyGoalPanel.vue

**功能**: 每日目标追踪

**展示**:
```
每日目标
工作进度: {work_minutes} / 480 分钟
[████████░░░░░░]  (满 480 分钟 = 400 生存信仰)

学习进度: {study_minutes} / 480 分钟
[████░░░░░░░░░░]  (满 480 分钟 = 400 精进信仰)

信仰上限: 1000
任务加成: +{task_bonus} (来自 completed 任务)
```

### 3.6 TaskList.vue

**功能**: 任务列表 + 状态筛选 + 操作

**布局**:
```
┌─────────────────────────────────────────┐
│  [全部] [进行中] [暂停] [已完成] [已放弃] │  ← 筛选标签
├─────────────────────────────────────────┤
│  □ 任务标题          [开始] [编辑] [删除]│  ← 每行一个任务
│  📁 work | 预计 60分钟 | 已用 0分钟       │
├─────────────────────────────────────────┤
│  □ 另一个任务        [暂停] [完成] [放弃]│
│  📚 study | 预计 90分钟 | 已用 45分钟    │
└─────────────────────────────────────────┘
```

**筛选标签**:
- 全部 / running / paused / completed / abandoned
- 点击切换 filter，触发 task store 重新过滤

**任务操作按钮**（根据状态动态显示）:
- `Paused` → [开始] [编辑] [删除]
- `Running` → [暂停] [完成] [放弃]
- `Completed` → [删除]（灰色，不可编辑）
- `Abandoned` → [删除]

**新建任务按钮**: 底部或顶部「+ 新建任务」→ 打开 TaskForm 弹窗

---

## 4. 任务表单（TaskForm.vue）

### 4.1 弹窗内容

```
┌────────────────────────────┐
│  新建任务 / 编辑任务    [×] │
├────────────────────────────┤
│  标题: [________________]  │
│  描述: [________________]  │
│  分类: ○ work  ○ study  ○ other
│  预计时长: [____] 分钟     │
│  日期: [2026-05-05]        │
│  [☑] 每日执行              │
│  所属列: [下拉选择看板列___]│  ← 仅看板模式下
├────────────────────────────┤
│        [取消]  [保存]      │
└────────────────────────────┘
```

**字段验证**:
- 标题: 必填
- 预计时长: 必填，> 0
- 日期: 默认为今天

---

## 5. 看板页面（KanbanPage.vue）

### 5.1 布局

```
┌──────────────────────────────────────────────────────────────┐
│  KanbanPage                                                  │
│  ┌──────────┬──────────┬──────────┬──────────┐               │
│  │  待办    │  进行中  │  暂停中  │  已完成  │  [+ 添加列]  │
│  │  (+)     │  (+)     │          │          │               │
│  ├──────────┼──────────┼──────────┼──────────┤               │
│  │ ┌──────┐ │ ┌──────┐ │ ┌──────┐ │ ┌──────┐ │               │
│  │ │Task 1│ │ │Task 3│ │ │Task 4│ │ │Task 2│ │               │
│  │ │⏱ 5:23│ │ │⏱ 1:12│ │ │      │ │ │✓     │ │               │
│  │ │📎进程│ │ │      │ │ │      │ │ │      │ │               │
│  │ └──────┘ │ └──────┘ │ └──────┘ │ └──────┘ │               │
│  │ ┌──────┐ │          │          │          │               │
│  │ │Task 5│ │          │          │          │               │
│  │ └──────┘ │          │          │          │               │
│  └──────────┴──────────┴──────────┴──────────┘               │
└──────────────────────────────────────────────────────────────┘
```

- 水平排列的列，每列固定宽度（如 280px），可横向滚动
- 每列顶部：标题 + 任务数量 + [+] 新建任务按钮
- 列内垂直排列 KanbanCard，支持拖拽排序

### 5.2 KanbanColumn.vue

**Props**:
```typescript
interface Props {
  column: KanbanColumn;
  cards: KanbanCard[];  // 该列的卡片，按 orderInColumn 排序
}
```

**事件**:
- `@drop(cardId, targetIndex)` — 拖拽释放时触发
- `@addCard(columnId)` — 点击 + 按钮
- `@deleteColumn(columnId)` — 删除列（仅自定义列）

**样式**:
- 背景: `var(--color-bg)` 或略深的表面色
- 边框: `1px solid var(--color-border)`
- 圆角: `8px`
- padding: `12px`
- 拖拽悬停: 边框高亮 `var(--color-primary)`

### 5.3 KanbanCard.vue

**Props**:
```typescript
interface Props {
  card: KanbanCard;
  isDragging?: boolean;
}
```

**展示内容**:
```
┌─────────────────────────────┐
│  任务标题                    │
│  📁 work | ⏱ 预计 60分钟    │
│  ─────────────────────────  │
│  ⏱ 已用: 00:05:23  (实时)  │  ← Running 时显示计时器
│  [开始] [暂停] [完成] [×]   │  ← 操作按钮
│  📎 notepad.exe (自动)      │  ← 进程绑定显示
└─────────────────────────────┘
```

**交互**:
- 拖拽: `draggable="true"`，`@dragstart` 设置数据，`@dragend` 清理
- 双击: 打开编辑弹窗（KanbanCardForm）
- 计时器: Running 状态时，前端 `setInterval(1000)` 实时更新 `duration_seconds` 显示

**进程绑定 UI**:
- 显示绑定应用名（如 "notepad.exe"）
- 绿色圆点: 进程正在运行
- 灰色圆点: 进程未运行

### 5.4 KanbanBoard.vue

**职责**:
- 管理列数组和卡片 Map
- 处理跨列拖拽逻辑
- 进程绑定轮询协调
- 计时器管理

**拖拽逻辑**:
```typescript
function onDrop(cardId: string, targetColumnId: string, targetIndex: number) {
    const card = cards.get(cardId);
    const sourceColumnId = card.columnId;
    
    // 1. 从源列移除
    const sourceCol = columns.find(c => c.id === sourceColumnId);
    sourceCol.taskIds = sourceCol.taskIds.filter(id => id !== cardId);
    
    // 2. 插入目标列
    const targetCol = columns.find(c => c.id === targetColumnId);
    targetCol.taskIds.splice(targetIndex, 0, cardId);
    
    // 3. 更新卡片
    card.columnId = targetColumnId;
    
    // 4. 若跨列且目标列是"进行中" → 自动 start_task
    // 5. 若跨列且目标列是"暂停中" → 自动 pause_task
    // 6. 若跨列且目标列是"已完成" → 自动 complete_task
    
    // 7. 持久化
    kanbanApi.saveConfig({ columns });
}
```

### 5.5 KanbanCardForm.vue

**功能**: 在看板内直接创建/编辑任务，比 TaskForm 多了进程绑定和提醒设置

**额外字段**:
```
进程绑定:
  应用名: [____________] [搜索进程]
  [☑] 进程启动时自动开始
  [☑] 进程结束时自动暂停

提醒:
  时间: [__:__] (HH:mm)
  [☑] 启用提醒

所属列: [下拉: 待办/进行中/暂停中/已完成/自定义列]
```

---

## 6. 悬浮窗（FloatingWidget.vue）

### 6.1 布局

```
┌────────┐
│  Lv5   │   ← 圆形，直径约 60px
│ 自律门徒│
└────────┘
```

- 圆形等级徽章，金色边框
- 中心显示等级数字
- 底部小字显示称号

### 6.2 交互

- **拖拽**: 鼠标按住可拖动位置
- **双击**: 调用 `invoke_show_main_window()` 打开主窗口
- **右键**: 可显示菜单（可选）

### 6.3 窗口特性

```
size: 80×80
always_on_top: true
decorations: false      // 无边框
skip_taskbar: true      // 不显示在任务栏
transparent: true       // 透明背景
shadow: false           // 无阴影
resizable: false
```

---

## 7. 通知与提醒

### 7.1 任务提醒

```
[Service] reminder-service.ts

每 60 秒检查一次:
  FOR each 启用了提醒的任务:
    IF 当前时间 == 提醒时间 AND 任务状态 == Paused:
      发送通知:
        Tauri 环境 → Tauri 原生通知
        浏览器环境 → new Notification(title, { body })
```

### 7.2 进程检测通知

```
[Service] process-detector.ts

每 3 秒检查一次:
  FOR each 有进程绑定的卡片:
    running = await invoke_is_process_running(appName)
    IF running AND !wasRunning AND autoStart:
        startTask(card.task.id)
        // 可选：发送通知 "{appName} 已启动，任务自动开始"
    IF !running AND wasRunning AND autoPause:
        pauseTask(card.task.id)
        // 可选：发送通知 "{appName} 已关闭，任务已暂停"
    wasRunning = running
```

---

## 8. 响应式与状态管理

### 8.1 Store 初始化顺序

```
App.vue onMounted (非悬浮窗)
  ├─ faith.ts init()
  │   ├─ get_or_create_user() → user
  │   └─ get_status() → faithStatus
  ├─ task.ts loadTasksByDate(today)
  │   └─ get_tasks_by_date(today)
  └─ kanban.ts loadBoard()
      ├─ 读取 localStorage 列配置
      └─ 合成 cards Map
```

### 8.2 数据同步模式

| 操作 | 前端乐观更新 | 后端确认 | 失败回滚 |
|------|-------------|----------|----------|
| 创建任务 | ❌ | 等后端返回 | — |
| 更新任务 | ❌ | 等后端返回 | — |
| 删除任务 | ❌ | 等后端返回 | — |
| 开始/暂停/恢复 | ✅ 立即改 status | 后端返回后确认 | 恢复旧状态 |
| 拖拽卡片 | ✅ 立即更新位置 | localStorage 持久化 | — |
| 计时器 | ✅ 前端 setInterval | 后端 session 为权威 | — |

---

## 9. 组件依赖图

```
App.vue
├── Dashboard.vue
│   ├── CalendarView.vue
│   ├── FaithDashboard.vue
│   ├── StatusPanel.vue
│   ├── DailyGoalPanel.vue
│   └── TaskList.vue
│       └── TaskForm.vue (弹窗)
├── KanbanPage.vue
│   └── KanbanBoard.vue
│       ├── KanbanColumn.vue
│       │   └── KanbanCard.vue
│       │       └── KanbanCardForm.vue (弹窗)
│       └── (KanbanCardForm 也可由列头 + 按钮触发)
└── FloatingWidget.vue
```

---

## 10. 图标与视觉规范

| 元素 | 颜色 | 说明 |
|------|------|------|
| 等级徽章 | `#ffd700` 金色 | 主主题色 |
| 进度条填充 | `#ffd700` | 升级进度 |
| 护甲条填充 | `#4ade80` 绿色 | 防护 |
| 工作标签 | `#ef4444` 红色系 | work |
| 学习标签 | `#3b82f6` 蓝色系 | study |
| 其他标签 | `#888899` 灰色 | other |
| 成功/完成 | `#4ade80` 绿色 | completed |
| 危险/删除 | `#ef4444` 红色 | delete / abandon |
| 进行中 | `#ffd700` 金色 | running |
| 暂停 | `#888899` 灰色 | paused |
