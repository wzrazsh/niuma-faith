# 牛马信仰 — UI 组件与交互规范

> 本文档精确描述前端所有组件、页面布局、交互行为和样式规范。删除代码后，仅凭本文档 + design-doc.md 可还原全部 Vue 组件。

## 1. 页面路由

```
/           → Dashboard.vue      (主仪表盘)
/kanban     → KanbanPage.vue     (任务看板)
/floating   → FloatingWidget.vue (悬浮窗，无边框透明)
*           → 重定向到 /
```

路由模式：Hash 模式 (`createWebHashHistory`)

> **更新日期**: 2026-05-06 — 全组件设计大修，暗金主题 v2。CSS 选择器变更汇总见附录 A。

## 2. 全局布局

### 2.1 App.vue（根组件）

```
┌────────────────────────────────────────────────────────┐
│  ✦ 牛马信仰       ◈ 仪表盘   ▣ 任务看板                 │  ← nav-bar，仅在非 /floating 路由显示
├────────────────────────────────────────────────────────┤
│                                                        │
│              <router-view />                           │
│              (全屏填充)                                 │
│                                                        │
└────────────────────────────────────────────────────────┘
```

**nav-bar 结构**:
```
.nav-bar (44px 高度, position:relative, z-index:10)
├── .nav-brand → .nav-logo (✦, 金色发光) + .nav-title (font-display, 0.95rem)
├── .nav-links → router-link × 2 (仪表盘/任务看板，每项含图标+文本)
└── .nav-glow (底部 1px 渐变金线)
```

**导航栏样式**:
- 背景: `var(--color-surface)` (#16162a)
- 底部边框: `1px solid var(--color-border-subtle)`
- 高度: 44px, padding: `0 20px`, gap: 24px
- 链接: 圆角 `var(--radius-sm)` (6px), 字号 0.85rem, gap: 6px
  - 默认: `var(--color-text-muted)`
  - hover: `var(--color-text)` + `rgba(255,255,255,0.04)` 背景
  - active (`.router-link-active`): `var(--color-primary)` + `var(--color-primary-glow)` 背景, 底部 2px 金线下划线 + 发光
- Logo: 字号 1.1rem, 金色, `logo-pulse` 动画 (2s infinite)

### 2.2 CSS 主题变量（style.css）

```css
:root {
  /* 背景色系 */
  --color-bg: #0c0c16;                /* 主背景 — 深蓝黑 */
  --color-bg-alt: #111122;            /* 交替背景 */
  --color-surface: #16162a;           /* 卡片/面板表面 */
  --color-surface-hover: #1e1e38;     /* 悬停表面 */
  --color-surface-raised: #1e1e3a;    /* 浮起表面 */

  /* 边框 */
  --color-border: #2a2a4a;            /* 主边框 */
  --color-border-subtle: #1e1e36;     /* 微妙边框 */

  /* 文本 */
  --color-text: #e4ddd0;              /* 主文本 — 暖白 */
  --color-text-muted: #7a7a9a;        /* 次文本 */
  --color-text-dim: #555570;          /* 淡化文本 */

  /* 主题色 */
  --color-primary: #ffd700;           /* 金色 — 等级/信仰主色 */
  --color-primary-dim: #b8860b;       /* 暗金 */
  --color-primary-glow: rgba(255, 215, 0, 0.12);       /* 弱发光 */
  --color-primary-glow-strong: rgba(255, 215, 0, 0.25); /* 强发光 */

  /* 语义色 */
  --color-success: #4ade80;           /* 绿色 — 成功/完成 */
  --color-success-dim: #166534;       /* 暗绿 */
  --color-danger: #ef4444;            /* 红色 — 删除/危险 */
  --color-danger-dim: #7f1d1d;       /* 暗红 */
  --color-work: #fb7185;              /* 工作标签 — 玫瑰红 */
  --color-study: #60a5fa;             /* 学习标签 — 天蓝 */
  --color-other: #a78bfa;             /* 其他标签 — 紫 */

  /* 字体 */
  --font-display: 'Noto Serif SC', 'Songti SC', Georgia, serif;
  --font-body: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, sans-serif;
  --font-mono: 'JetBrains Mono', 'Consolas', monospace;

  /* 圆角 */
  --radius-sm: 6px;
  --radius-md: 10px;
  --radius-lg: 16px;
  --radius-xl: 24px;

  /* 阴影 */
  --shadow-panel: 0 2px 20px rgba(0,0,0,0.3), inset 0 1px 0 rgba(255,255,255,0.03);
  --shadow-glow: 0 0 20px var(--color-primary-glow), 0 2px 12px rgba(0,0,0,0.4);

  /* 过渡 */
  --transition-fast: 0.15s cubic-bezier(0.4,0,0.2,1);
  --transition-normal: 0.25s cubic-bezier(0.4,0,0.2,1);
  --transition-slow: 0.4s cubic-bezier(0.4,0,0.2,1);
}
```

### 2.3 全局样式

- **body**: font-family `var(--font-body)`, background `var(--color-bg)`, line-height 1.6, antialiased, `overflow:hidden`
- **#app**: 伪元素背景 — 三层径向渐变（金色顶部、蓝色右下、玫瑰色左下），无交互 (`pointer-events:none`)
- **button**: 圆角 `var(--radius-sm)`, padding 6px 14px, 字号 0.8rem, 字重 500, hover `translateY(-1px)`
  - `.primary`: 金色渐变 `linear-gradient(135deg, #ffd700, #b8860b)`, 文字深色, 发光阴影
  - `.danger`: 暗红背景 + 红文字 + 边框; hover 红底白字
  - `.success`: 暗绿背景 + 绿文字 + 边框; hover 绿底深字
- **input/select/textarea**: 深色背景, border 2a2a4a, focus 金色边框 + 3px 发光
- **select**: 自定义下拉箭头 SVG, padding-right 28px
- **滚动条**: 宽度 6px, 圆角, hover 颜色加深

### 2.4 全局工具类

| 类名 | 效果 |
|------|------|
| `.glass-panel` | `var(--color-surface)` 背景 + 微妙边框 + `var(--shadow-panel)` 阴影; hover 边框加深 |
| `.section-title` | `font-display`, 0.8rem, 600字重, 大写, muted 色 |
| `.glow-border` | 相对定位; `::after` 金色渐变边框, hover 显示 |
| `.animate-in` | `fade-slide-up` 入场动画; nth-child 交错延迟 0.05s |

### 2.5 全局动画关键帧

- `fade-slide-up`: opacity 0→1 + translateY 8→0
- `glow-pulse`: box-shadow 强弱交替 (8px↔20px)
- `shimmer`: 光泽扫光效果, background-position 200% 循环
- `float-up`: 上浮淡入
- `ember`: 火花粒子消散 (缩放+上升+淡出)

---

## 3. 仪表盘页面（Dashboard.vue）

### 3.1 布局

```
┌──────────────────────────────────────────────────────────────┐
│  Dashboard (flex, gap: 18px, padding: 20px)                  │
│  ┌─────────────────┬───────────────────────────────────────┐ │
│  │  .sidebar        │  .main                               │ │
│  │  (320px 固定宽)  │  (flex:1, overflow:auto)             │ │
│  │                 │                                       │ │
│  │  CalendarView   │  TaskList (任务列表)                  │ │
│  │  FaithDashboard │                                       │ │
│  │  StatusPanel    │                                       │ │
│  │  DailyGoalPanel │                                       │ │
│  └─────────────────┴───────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

- `.sidebar`: 320px, 垂直 flex 布局, gap: 14px
- `.main`: flex: 1, overflow-y: auto
- dashboard 外层: display flex, gap 18px, padding 20px

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

**结构**:
```
.status-panel (玻璃面板, padding: 16px, gap: 14px, 顶部 1px 渐变金线)
├── .level-badge
│   ├── .level-ring           ← 72px 圆形, conic-gradient 金色环
│   │   └── .level-num        ← "Lv.{level}", font-display, 1.3rem, 金色发光
│   └── .level-title           ← 称号 (见习牛马…), 0.8rem, muted
├── .stat-section
│   ├── .stat-label            ← "信仰积累", 小号大写 label
│   ├── .progress-track        ← 8px 高进度条
│   │   └── .progress-fill     ← 渐变色填充 + shimmer 光泽动画
│   └── .stat-value            ← "累计 / 上限", mono 字体
│       ├── .stat-divider       ← "/" 分隔符
│       └── .stat-target        ← 上限值, 暗金色
├── .armor-section
│   ├── .armor-header
│   │   ├── .stat-label         ← "护甲值"
│   │   └── .armor-value        ← "{armor} / {total_armor}", 绿色, mono
│   └── .armor-track           ← 6px 高护甲条
│       └── .armor-fill         ← 渐变绿填充
```

**关键样式**:
- `.level-ring`: `conic-gradient(from 0deg, #ffd700, #b8860b, #ffd700)`, `::before` 内嵌 3px 形成圆环
- `.progress-fill::after`: shimmer 光泽扫光
- `.armor-track`: 6px 高, 圆角 3px
- 满级: `.max-level` 显示 "已达最高等级", font-display

**CSS 选择器对照** (v2):
- `.armor-section` (v1: `.armor-bar`)
- `.progress-track` (v1: `.progress-section`)

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

**结构**:
```
.task-list
├── .task-header
│   ├── .filter-tabs            ← 筛选项: [全部] [进行中] [暂停] [已完成] [已放弃]
│   └── button.primary          ← "+ 新建任务"
└── .tasks
    ├── .task-row (.status-{status})  ← 每个任务行
    │   ├── .task-indicator      ← 4px 宽色条 (paused=灰, running=金+发光, completed=绿, abandoned=红)
    │   ├── .task-info
    │   │   ├── .task-title      ← 0.88rem, 600字重, 文字溢出省略号
    │   │   └── .task-meta       ← flex 行, 分类徽章 · 预计分钟 · 已用秒
    │   │       ├── .task-category (.work/.study/.other) ← 分类彩色标签
    │   │       └── .meta-sep    ← "·" 分隔符, dim color
    │   └── .task-actions        ← 根据 status 动态显示操作按钮
    └── .empty                   ← "◈ 暂无任务" (图标 + 文字)

筛选标签样式:
- 默认: transparent 背景, muted 文字
- hover: surface 背景, text 文字, subtle 边框
- active: primary-glow 背景, primary 文字, 金色边框
```

**任务操作按钮**（根据状态动态显示）:
- `Paused` → [开始] [编辑] [删除]
- `Running` → [暂停] [完成] [放弃]
- `Paused` (额外) → [继续] `.success`
- `Completed` → 无操作（灰色不可编辑）
- `Abandoned` → 无操作

**任务行样式**:
- 边框: `1px solid var(--color-border-subtle)`
- 圆角: `var(--radius-md)`
- padding: 12px 14px, gap: 12px
- hover: border 加深 + surface-hover 背景

---

## 4. 任务表单（TaskForm.vue）

### 4.1 弹窗

```
┌─────────────────────────────────────┐
│  新建任务                        [✕] │  ← modal-header, font-display title
├─────────────────────────────────────┤
│  任务名称                            │  ← .field-group > .field-label
│  [                               ]  │
│                                     │
│  描述                                │
│  [                               ]  │  ← textarea, rows=2
│                                     │
│  ┌──────────┐  ┌──────────┐        │  ← .field-row (flex, gap: 12px)
│  │ 分类      │  │ 预计时长  │        │
│  │ [工作 ▼] │  │ [___] 分钟│        │
│  └──────────┘  └──────────┘        │
│                                     │
│  [✓] 每日执行                       │  ← .checkbox-field (custom checkbox)
├─────────────────────────────────────┤
│                   [取消]  [创建任务]  │  ← modal-footer, primary 按钮
└─────────────────────────────────────┘
```

### 4.2 字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| title | text | ✅ | 任务名称 |
| description | textarea | ❌ | 可选描述 |
| category | select | ✅ | work / study / other (展示中文: 工作/学习/其他) |
| estimated | number | ✅ | > 0, 分钟 |
| daily | checkbox | ❌ | 自定义复选框 (`.check-box`) |

### 4.3 动画

- `.modal-overlay`: `fade-in` 0.2s ease + `blur(8px)` 背景模糊
- `.modal`: `modal-enter` 0.25s cubic-bezier — 缩放 0.95→1 + 上移 10px
- 关闭按钮: 圆形, hover 背景出现

### 4.4 自定义复选框

```html
<label class="checkbox-field">
  <input type="checkbox" v-model="daily" />
  <span class="check-box"></span>
  <span>每日执行</span>
</label>
```

- `.check-box`: 18×18, border 2px, 圆角 4px
- `input:checked + .check-box`: 金色背景 + "✓" 伪元素
- input 默认 `display:none`

---

## 5. 看板页面（KanbanPage.vue）

### 5.1 布局

```
┌──────────────────────────────────────────────────────────────┐
│  .kanban-page (padding:20px, height:calc(100vh-44px), overflow:hidden)│
│  ┌──────────────────────────────────────────────────────────┐ │
│  │  .board                                                  │ │
│  │  ┌────────────────────────────────────────────────────┐  │ │
│  │  │  .board-header                                     │  │ │
│  │  │  .board-title "任务看板"  .board-actions            │  │ │
│  │  │                          [+ 添加列] [重置默认]     │  │ │
│  │  ├────────────────────────────────────────────────────┤  │ │
│  │  │  .board-columns (flex, gap:14px, overflow-x:auto)  │  │ │
│  │  │  ┌──────────┬──────────┬──────────┬──────────┐    │  │ │
│  │  │  │ 待办  [1]│ 进行中[2]│ 暂停中[1]│ 已完成[1]│    │  │ │
│  │  │  │   [+]    │   [+]    │   [+]    │   [+]    │    │  │ │
│  │  │  │ ● 工作   │ ● 学习   │ ● 工作   │ ● 学习   │    │  │ │
│  │  │  │  card    │  card    │  card    │  card    │    │  │ │
│  │  │  │          │ ● 其他   │          │          │    │  │ │
│  │  │  │          │  card    │          │          │    │  │ │
│  │  │  └──────────┴──────────┴──────────┴──────────┘    │  │ │
│  │  └────────────────────────────────────────────────────┘  │ │
│  └──────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### 5.2 KanbanBoard.vue

**结构**:
```
.board (flex-column, gap:16px, height:100%)
├── .board-header (flex, space-between, align:center)
│   ├── .board-title (font-display, 1.1rem, 700字重)
│   └── .board-actions (flex, gap:8px)
│       ├── button: "+ 添加列" (surface bg, subtle border)
│       └── button: "重置默认"
└── .board-columns (flex, gap:14px, flex:1, overflow-x:auto)
    └── KanbanColumn × N
```

**操作按钮样式**: 0.78rem, surface 背景, subtle 边框, muted 文字; hover 颜色加深

### 5.3 KanbanColumn.vue

**结构**:
```
.column (min-width:260px, max-width:300px, flex-shrink:0)
├── .column-header (padding:14px 14px 0)
│   ├── .column-info
│   │   ├── .column-title    ← font-display, 0.85rem, 600字重
│   │   └── .column-count     ← mono font, 0.72rem, 圆角徽章
│   └── .column-add           ← 圆形 "+" 按钮, hover 变金色
├── .column-cards (padding:0 10px 10px, gap:8px)
│   ├── .swimlane × N
│   │   ├── .swimlane-header
│   │   │   ├── .swimlane-dot       ← 6px 颜色圆点 (work/study/other)
│   │   │   ├── .swimlane-label     ← 0.68rem, uppercase, muted
│   │   │   └── .swimlane-count     ← 0.62rem, mono font
│   │   └── KanbanCard × N
└── KanbanCardForm (条件渲染)
```

**泳道分组**: 每列内按 Task.category（work/study/other）分为 工作/学习/其他 三组，仅显示非空组。

**交互**:
- `@dragover.prevent` + `@dragleave` → 切换 `.dragging` 类
- 拖入时: 边框变金色, 背景变 primary-glow
- `@drop` → `kanban.moveCard(cardId, targetColumnId, targetIndex)`

### 5.4 KanbanCard.vue

**结构**:
```
.card (bg-alt, subtle border, 圆角, cursor:pointer)
├── .card-info
│   ├── .card-title             ← 关联任务标题, 0.82rem, 500字重
│   └── .card-category          ← work/study/other 彩色标签, 0.65rem
└── .card-timer (条件: timerRunning)
    ├── .timer-dot              ← 5px 金色圆点, 脉冲动画
    └── "进行中"                ← 0.7rem, primary color
```

**交互**:
- `draggable="true"`, `@dragstart` 设置 `text/plain` data
- `@click` → emits `edit(card.id)`, 打开编辑弹窗
- hover: border 加深, bg 变亮, `translateX(2px)`
- active: `cursor:grabbing`

### 5.5 KanbanCardForm.vue

**结构**:
```
.overlay (fixed, inset:0, blur(4px))
└── .form-panel (width:360px, surface bg, border, 圆角)
    ├── .form-title "添加卡片"  (font-display, 0.95rem, 700)
    ├── .form-field
    │   ├── label "关联任务"    (0.75rem, 600字重, muted)
    │   └── select (未关联到看板的 tasks)
    └── .form-actions
        ├── [取消]
        └── [添加] .primary
```

**功能**: 选中未关联看板的已有任务，添加到指定列

**CSS 选择器对照** (v2):
- `.board` (v1: `.kanban-board`)
- `.board-title` (v1: `.board-header h2`)
- `.column` (v1: `.kanban-column`)
- `.card` (v1: `.kanban-card`)

---

## 6. 悬浮窗（FloatingWidget.vue）

### 6.1 布局

```
┌──────────────────┐
│                  │
│   .widget-bg     │  ← conic-gradient 旋转环 (8s infinite spin)
│   ┌──────────┐   │
│   │          │   │
│   │  Lv.0    │   │  ← .widget-inner (inset:4px, rounded, bg-bg)
│   │  ──────  │   │  ← .widget-divider (渐变横线)
│   │    0     │   │  ← .widget-faith (小字, mono, muted)
│   │          │   │
│   └──────────┘   │
│                  │
└──────────────────┘
```

- 窗口 80×80，组件使用 `width: min(100vw, 100vh); height: min(100vw, 100vh)` 保证正方形
- `.widget-bg`: 绝对定位, `border-radius: 50%` (完美圆形), `conic-gradient` 从 surface → primary-dim → surface, 8s 线性旋转
- `.widget-inner`: 绝对定位, inset:4px, `border-radius: 50%`, `var(--color-bg)`, flex-column 居中, gap:2px, 1px 金色半透明边框
- `.widget-level`: `font-display`, 1rem, 900字重, 金色发光
- `.widget-divider`: 20px 宽, 1px 高, 渐变成水平线
- `.widget-faith`: 0.65rem, `font-mono`, muted

### 6.2 交互

- **拖拽**: 鼠标按住超过 4px 阈值后调用 `getCurrentWindow().startDragging()` 拖动窗口
- **双击**: 调用 `show_main_window` 命令打开/聚焦主窗口（主窗口不存在时自动创建 900×700 窗口）
- **数据刷新**: 每 5 秒轮询 `get_status` 更新等级和信仰值

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

### 6.4 触发方式

- **系统托盘菜单**: 右键托盘图标 → "打开悬浮窗" (`id: "floating"`)
- **Tauri 命令**: `open_floating_widget` / `close_floating_widget`
- **路由直接访问**: `/#/floating` (开发调试用)
- **Tauri 窗口 URL**: 使用 `/?f=1` 查询参数（非 `/#/floating`），因 Tauri 不完全支持 URL 中的 `#` 字符。`index.html` 内联脚本在 Vue 启动前将 `?f=1` 转换为 `#/floating`
- 若悬浮窗已存在，调用 `show()`；否则用 `WebviewWindowBuilder` 创建

> **注意**: 浏览器模式无法触发托盘菜单，悬浮窗测试需在 Tauri 环境中通过托盘菜单启动。

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

## 10. 图标、字体与视觉规范

### 10.1 字体系统

| 用途 | CSS 变量 | 字体栈 | 文件 |
|------|---------|--------|------|
| 标题/展示 | `--font-display` | Noto Serif SC, Songti SC, Georgia, serif | Google Fonts |
| 正文/UI | `--font-body` | Plus Jakarta Sans, -apple-system, sans-serif | Google Fonts |
| 代码/数字 | `--font-mono` | JetBrains Mono, Consolas, monospace | Google Fonts |

Google Fonts 在 `index.html` 中以 `<link>` 方式加载，包含字号 400/500/600/700/900。

### 10.2 颜色语义

| 用途 | CSS 变量 | 色值 | 使用场景 |
|------|---------|------|---------|
| 主背景 | `--color-bg` | `#0c0c16` | body, app 全局背景 |
| 面板背景 | `--color-surface` | `#16162a` | 卡片, 面板, nav-bar |
| 悬停背景 | `--color-surface-hover` | `#1e1e38` | button:hover, 行 hover |
| 主文本 | `--color-text` | `#e4ddd0` | 标题, 正文 |
| 次文本 | `--color-text-muted` | `#7a7a9a` | 元数据, label |
| 淡化文本 | `--color-text-dim` | `#555570` | placeholder, 分隔符 |
| 金色 | `--color-primary` | `#ffd700` | 等级徽章, 进度条填充, 激活态 |
| 暗金 | `--color-primary-dim` | `#b8860b` | 渐变终点, 次要金色元素 |
| 金色发光 | `--color-primary-glow` | `rgba(255,215,0,.12)` | 发光阴影, 金色背景层 |
| 绿色 | `--color-success` | `#4ade80` | 护甲值, 完成态 |
| 暗绿 | `--color-success-dim` | `#166534` | success 按钮背景 |
| 红色 | `--color-danger` | `#ef4444` | 删除, 废弃态 |
| 暗红 | `--color-danger-dim` | `#7f1d1d` | danger 按钮背景 |
| 工作标签 | `--color-work` | `#fb7185` | work category badge |
| 学习标签 | `--color-study` | `#60a5fa` | study category badge |
| 其他标签 | `--color-other` | `#a78bfa` | other category badge |

### 10.3 任务状态色指示

| 状态 | 指示器颜色 | 说明 |
|------|-----------|------|
| running | `#ffd700` 金 + glow | 进行中, 脉冲发光 |
| paused | `#555570` 灰 | 暂停, dim |
| completed | `#4ade80` 绿 | 已完成 |
| abandoned | `#ef4444` 红 | 已放弃 |

### 10.4 间距与尺寸规范

| 元素 | 值 |
|------|----|
| nav-bar 高度 | 44px |
| sidebar 宽度 | 320px |
| kanban 列宽 | min 260px, max 300px |
| 卡片 padding | 12px 14px |
| 组件间 gap | 14-18px |
| 悬浮窗尺寸 | 80×80px |

---

## 附录 A：CSS 选择器迁移对照 (v1 → v2)

| v1 选择器 | v2 选择器 | 影响范围 |
|-----------|-----------|---------|
| `.main-area` | `.main` | Dashboard.vue |
| `.kanban-board` | `.board` | KanbanBoard, smoke.spec.ts |
| `.kanban-column` | `.column` | KanbanColumn |
| `.kanban-card` | `.card` | KanbanCard |
| `.armor-bar` | `.armor-section` | StatusPanel, smoke.spec.ts |
| `.progress-section` | `.progress-track` | StatusPanel, smoke.spec.ts |
| `.board-header h2` | `.board-header .board-title` | KanbanBoard, smoke.spec.ts |

## 附录 B：新增 DOM 结构

| 组件 | 新增结构 |
|------|---------|
| StatusPanel | `.level-ring` (72px 圆环) + `.level-num` 内嵌; `.armor-header` + `.armor-track`; `.stat-section` |
| TaskList | `.task-indicator` (4px 状态色条); `.task-category` (分类标签) |
| TaskForm | `.field-group` + `.field-label`; `.checkbox-field` + `.check-box` (自定义复选框) |
| FloatingWidget | `.widget-bg` (旋转环) + `.widget-inner` (内嵌圆); `.widget-divider` |
| App.vue | `.nav-glow` (底部渐变金线); `.nav-icon` (导航图标); `logo-pulse` 动画 |
