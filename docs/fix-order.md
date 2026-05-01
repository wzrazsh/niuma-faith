# 牛马信仰 — 修复任务清单

> 来源：回归测试报告 | 日期：2026-05-01 | 总问题数：4

---

## B1 · 阻塞 🔴 · TaskList.vue Git merge conflict

**位置**：`frontend/src/components/TaskList.vue:109`

**当前状态**：
```
109: <<<<<<< HEAD
110:         <div v-if="!readonly" class="task-actions">
111-126:    [操作按钮模板]
```

文件中仅残留 `<<<<<<< HEAD` 冲突标记，无对应 `=======` 和 `>>>>>>>` 分隔符。

**影响**：浏览器中直接显示 `<<<<<<< HEAD` 原始文本；任务操作按钮被标记包围（实际逻辑可用但 UI 泄漏源码）。

**修复方案**：删除第 109 行 `<<<<<<< HEAD`，保留 110-126 行的 `<div v-if="!readonly" class="task-actions">` 模板。

**预估工时**：2 分钟

---

## B2 · 中等 🟡 · mock-invoke.ts 命令处理器不完整

**位置**：`frontend/src/api/mock-invoke.ts`

**影响**：浏览器 `npm run dev` 模式下无法验证任务启停、签到、日统计等完整业务闭环。

**缺失的命令**：

| 命令 | 浏览器端行为 | 关联测试 |
|------|-------------|----------|
| `start_task` | 更新 task.status='running'，记录 started_at | 看板计时器验证 |
| `pause_task` | 更新 task.status='paused'，累计 actual_minutes | 看板暂停验证 |
| `resume_task` | 更新 task.status='running' | 看板恢复验证 |
| `end_task` | 同 complete_task，但通过 started_at 计算时长 | 计时器结束验证 |
| `check_in` | 创建当天 DailyRecord，返回 FaithStatus | 签到流程验证 |
| `get_daily_stats` | 返回日统计 {survival/progress/discipline/total} | 日统计验证 |
| `get_task_session` | 返回当前进行中的 task_session | 计时器轮询验证 |

**保留服务端验证的 Mock**：

| 命令 | 当前行为 | 处理 |
|------|---------|------|
| `get_status` | 返回旧版 `{faith, level, title, checked_in}` | 升级为 FaithStatus 2.0（含 armor/total_armor/cumulative_faith/next_threshold/current_level/level_title/progress_to_next/today） |
| `get_today_record` | 旧版 flat record | 升级为 DailyRecord 2.0（含 survival_faith/progress_faith/discipline_faith/total_faith） |
| `get_or_create_user` | 返回 base 字段 | 补充 armor/total_armor 字段 |

**修复方案**：
1. 补充 7 个缺失命令到 `handlers` 对象
2. 升级 `get_status` 返回体到 FaithStatus 2.0 结构
3. 升级 `get_today_record` 返回体到 DailyRecord 2.0 结构
4. 升级 `get_or_create_user` 返回体补充 armor 字段

**预估工时**：30 分钟

---

## B3 · 中等 🟡 · 等级进度条显示"已达最高等级"

**位置**：`frontend/src/components/StatusPanel.vue:62-63`

**现象**：
```
距下一级  0 / MAX
已达到最高等级 · 牛马圣徒
```
但标题显示 `Lv.1 见习牛马`。

**根因**：
1. `frontend/src/stores/faith.ts:37` — `currentLevel.cumulative_threshold` 硬编码为 `0`
2. `StatusPanel.vue:17-18` — `percentToNext` 中 `nextThreshold - cumulative_threshold` 得出 `15000 - 0 = 15000`，但 `cumulativeFaith` 也是 `0`，`made = 0 - 0 = 0`，`progress = 0/15000*100 = 0%`（这条逻辑正确）
3. 真正问题：`progressNeeded` 是 `faithStatus.progress_to_next`（从后端获取），新用户为零值 → 触发 `v-else` 分支显示"已达最高等级"

**修复方案**：
- 方案 A（推荐）：在 StatusPanel 中，当 `currentLevel.level < 15`（最高等级）时，即使 `progressNeeded` 为 0 也不显示"已达最高等级"，改为显示 `progressNeeded` 实际值或默认文案"继续积累即可升级"
- 方案 B：修复后端确保新用户 `progress_to_next = next_threshold - cumulative_threshold` 不为 0

**预估工时**：10 分钟

---

## B4 · 低 🟢 · 17 项 TypeScript 错误

**位置**：`npx vue-tsc --noEmit` 输出

| # | 文件:行 | 错误码 | 问题 | 
|---|---------|--------|------|
| 1 | `mock-invoke.ts:66` | TS6133 | `saveFaith` declared but never read |
| 2-6 | `mock-invoke.ts:168-203` | TS6133 | 5 处 `args` 参数声明但未使用 |
| 7-10 | `api/task.ts:76-88` | TS2304 | 4 处 `invoke` 未找到（函数名拼写错误或导入缺失） |
| 11 | `KanbanBoard.vue:95` | TS2367 | `TaskStatus` 字面量 `'active'` 类型不匹配 |
| 12 | `KanbanBoard.vue:97` | TS2367 | 同上 |
| 13 | `KanbanBoard.vue:178` | TS2345 | `'active'` 字面量不匹配 `TaskStatus` 类型 — 新增 |
| 14 | `KanbanBoard.vue:329` | TS2322 | `null` 不可赋给 `Task \| undefined` — 新增 |
| 15 | `KanbanCard.vue:39` | TS2367 | `TaskStatus` 字面量 `'active'` 类型不匹配 |
| 16 | `KanbanCard.vue:93` | TS2367 | 同上 |
| 17 | `stores/task.ts:35` | TS6133 | `activeTasks` 声明但未使用 |

**影响**：不影响 `vite build`（构建仅打包），但 TS 严格模式下编译失败。

**修复方案**：
1. `mock-invoke.ts`（7 处）：未使用参数加 `_` 前缀 → `_args`
2. `api/task.ts`（4 处）：补 `import { invoke } from "@tauri-apps/api/core"` 或使用 `safeInvoke`
3. `KanbanBoard.vue` / `KanbanCard.vue`（5 处）：将 `TaskStatus` 类型补 `'active'` 字面量，或传参时强制 `as TaskStatus`
4. `stores/task.ts`（1 处）：删除未使用的 `activeTasks`

**预估工时**：15 分钟

---

## 执行顺序（优先级）

```
B1（阻塞）→ B3（UI bug）→ B2（mock 补全）→ B4（TS 修复）
```

| 阶段 | 任务 | 预估 | 状态 |
|------|------|------|------|
| 1 | B1 — 删除 merge conflict 标记 | 2 min | 🔲 |
| 2 | B3 — 修复等级进度条逻辑 | 10 min | 🔲 |
| 3 | B2 — mock-invoke.ts 补全命令 | 30 min | 🔲 |
| 4 | B4 — 修复 17 项 TS 错误 | 15 min | 🔲 |
| **总计** | | **~60 min** | |
