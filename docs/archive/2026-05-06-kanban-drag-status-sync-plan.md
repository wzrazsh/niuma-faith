# 看板拖拽状态同步 — 完整设计计划（修订版）

> 创建日期：2026-05-06 | 修订日期：2026-05-06 | 状态：实施中
> 修订依据：Codex 架构评审（`.omc/artifacts/ask/codex-2026-05-06-kanban-plan-review.md`）

## 1. 问题诊断

### 1.1 症状

看板中拖动任务卡片到不同列后，任务的 `status` 字段未改变。刷新页面后卡片回到原始列。

### 1.2 根因分析

| # | 位置 | 问题 |
|---|------|------|
| A | `kanban.ts:118-129` (`moveCard`) | 仅更新 localStorage，从未调用后端同步 |
| B | `kanban.ts:55-93` (`loadBoard`) | 对已在 `existingTaskIds` 中的任务，本地列位置覆盖真实 status（刷新后不按 status 重新分配） |
| C | `kanban.ts:45-53` (`mapStatusToColumn`) | `todo→paused` 和 `paused→paused` 共享同一状态，反向不可逆 |
| D | `kanban.ts:152-162` (`addColumn`) | 自定义列无状态映射字段 |
| E | `api/task.ts:21-22` | 未提供字段传 `null` 而非省略，Mock 层 `!== undefined` 判断导致脏写 |

### 1.3 Codex 评审关键发现

1. **`update_task(status)` 绕过了所有业务逻辑**（session 创建/关闭、信仰奖励、虚拟任务物化）。必须用 `start_task` / `pause_task` / `complete_task` / `abandon_task`。
2. **`loadBoard` 不会按后端 status 重新分配已有任务** — 本地 localStorage 始终覆盖真实状态。
3. **两个列（todo/paused）映射到同一 status（paused）** — 反向映射必然丢失信息。

---

## 2. 设计决策（修订后）

### 2.1 范围：本次只处理默认四列

| 列 ID | 列标题 | 拖入时调用的命令 | 说明 |
|-------|--------|-----------------|------|
| `todo` | 待办 | `pauseTask(cardId)` | 停止计时，设为 paused |
| `inprogress` | 进行中 | `startTask(cardId)` | 开始计时，创建 session（已 running/completed/abandoned 除外） |
| `paused` | 暂停中 | `pauseTask(cardId)` | 停止计时，关闭 session，累计时长 |
| `done` | 已完成 | `completeTask(cardId, estimatedMinutes)` | 先 pause（如 running），再设为 completed，发放信仰奖励 |

自定义列的拖拽同步延迟到后续 PR，届时重新设计持久化方案。

### 2.2 业务命令 vs update_task

| 命令 | 执行的关键副作用 |
|------|-----------------|
| `start_task` | 虚拟任务物化 → 创建 `task_sessions` → 设 `started_at` → 状态=running → 更新 daily_record |
| `pause_task` | 关闭 `task_sessions` → 累计 `duration_seconds` → 更新 daily_record → 状态=paused |
| `complete_task` | 内部先 pause（如 running）→ 状态=completed → 设 `completed_at` → 发信仰奖励 → 更新 daily_record |
| `abandon_task` | 内部先 pause（如 running）→ 状态=abandoned |
| `update_task` | **仅改字段 + `updated_at`**，不执行业务副作用 — 拖拽时不再使用 |

### 2.3 列映射的不可逆问题

`todo` 和 `paused` 都映射到 `paused` 状态。这意味着刷新后两者无法区分。**这是可接受的**：用户视觉上把"待办"当 backlog，"暂停中"当已开始但暂存的。两个列反映的是用户意图，不是数据实体差异。

### 2.4 历史日期保护（修正）

原计划用 `new Date().toISOString().slice(0, 10)` 在中国时区凌晨会误判。改为用 `taskStore.selectedDate`（YYYY-MM-DD 格式），与后端 `today_str()` 对齐。

### 2.5 竞态处理

用 per-task 的序列号/时间戳确保 last-user-action-wins：
- 拖拽时记录操作序号，调用后端时携带
- 后端返回后检查序号是否最新，过期的结果静默丢弃

---

## 3. 代码修改计划

### 3.1 修复 Mock 层 null 字段问题

**文件**: `frontend/src/api/task.ts`

将 `status: status || null` 改为 `status: status ?? undefined`，确保未提供时不传 `null`：

```typescript
export function invoke_update_task(id: string, title?: string, description?: string, category?: string, estimatedMinutes?: number, actualMinutes?: number, notes?: string, status?: string): Promise<Task> {
  return safeInvoke('update_task', { id, title: title ?? undefined, description: description ?? undefined, category: category ?? undefined, estimatedMinutes: estimatedMinutes ?? undefined, actualMinutes: actualMinutes ?? undefined, notes: notes ?? undefined, status: status ?? undefined });
}
```

### 3.2 修复 loadBoard — 让后端 status 覆盖 localStorage 列位置

**文件**: `frontend/src/stores/kanban.ts`

在 `loadBoard()` 末尾增加 reconcile 步骤：

```typescript
// reconcile: 将已在 localStorage 但 status 不匹配列的任务移到正确列
for (const col of columns.value) {
  for (let i = col.taskIds.length - 1; i >= 0; i--) {
    const id = col.taskIds[i];
    const task = taskStore.tasks.find(t => t.id === id);
    if (!task) continue;
    const expectedColId = mapStatusToColumn(task.status);
    if (expectedColId !== col.id) {
      col.taskIds.splice(i, 1);
      const targetCol = columns.value.find(c => c.id === expectedColId);
      if (targetCol) {
        targetCol.taskIds.push(id);
      } else {
        // 目标列不存在（自定义列被删），放回 todo
        const fallback = columns.value.find(c => c.id === 'todo');
        if (fallback) fallback.taskIds.push(id);
      }
    }
  }
}
saveConfig({ columns: columns.value });
```

### 3.3 修改 moveCard — 用业务命令替代 update_task

**文件**: `frontend/src/stores/kanban.ts`

```typescript
// 拖拽序号，用于竞态处理
const dragSeq = ref<Map<string, number>>(new Map());

function moveCard(cardId: string, targetColumnId: string, targetIndex: number) {
  const card = cards.value.get(cardId);
  if (!card) return;

  const sourceCol = columns.value.find(c => c.id === card.columnId);
  const targetCol = columns.value.find(c => c.id === targetColumnId);
  if (!sourceCol || !targetCol) return;

  // 同列移动：只更新顺序，不同步后端
  if (sourceCol.id === targetCol.id) {
    sourceCol.taskIds = sourceCol.taskIds.filter(id => id !== cardId);
    sourceCol.taskIds.splice(targetIndex, 0, cardId);
    saveConfig({ columns: columns.value });
    return;
  }

  // 跨列移动：更新 localStorage + 同步后端
  sourceCol.taskIds = sourceCol.taskIds.filter(id => id !== cardId);
  targetCol.taskIds.splice(targetIndex, 0, cardId);
  card.columnId = targetColumnId;
  cards.value.set(cardId, card);
  saveConfig({ columns: columns.value });

  // 保护检查
  const taskStore = useTaskStore();
  const task = taskStore.tasks.find(t => t.id === cardId);
  if (!task) return;
  if (task.date < taskStore.selectedDate) return;  // 历史日期
  if (task.task_type === 'project') return;         // 项目任务
  if (task.id.startsWith('daily:')) return;         // 虚拟每日任务

  // 竞态序号
  const seq = (dragSeq.value.get(cardId) ?? 0) + 1;
  dragSeq.value.set(cardId, seq);

  // 根据目标列调用对应业务命令
  syncTaskStatus(cardId, targetColumnId, task, seq);
}

function syncTaskStatus(cardId: string, columnId: string, task: Task, seq: number) {
  const taskStore = useTaskStore();
  let promise: Promise<any>;

  switch (columnId) {
    case 'inprogress':
      // 终态任务不能重新开始
      if (task.status === 'completed' || task.status === 'abandoned') return;
      promise = taskStore.startTask(cardId);
      break;
    case 'paused':
    case 'todo':
      promise = taskStore.pauseTask(cardId);
      break;
    case 'done':
      promise = taskStore.completeTask(cardId, task.actual_minutes || task.estimated_minutes);
      break;
    default:
      return; // 自定义列暂不处理
  }

  promise.then(() => {
    // 只接受最新操作的结果
    if (dragSeq.value.get(cardId) === seq) {
      dragSeq.value.delete(cardId);
    }
  }).catch(e => {
    // 后端拒绝（如历史日期）时回滚 UI
    console.warn('[kanban] status sync failed, rolling back:', e);
    if (dragSeq.value.get(cardId) === seq) {
      // 回滚：将卡片移回原列
      const currentCol = columns.value.find(c => c.id === columnId);
      const originalColId = mapStatusToColumn(task.status);
      const originalCol = columns.value.find(c => c.id === originalColId);
      if (currentCol && originalCol) {
        currentCol.taskIds = currentCol.taskIds.filter(id => id !== cardId);
        originalCol.taskIds.push(cardId);
        const card = cards.value.get(cardId);
        if (card) { card.columnId = originalColId; cards.value.set(cardId, card); }
        saveConfig({ columns: columns.value });
      }
      dragSeq.value.delete(cardId);
    }
  });
}
```

### 3.4 类型定义（无需改）

`KanbanColumn` 暂不增加 `mappedStatus`（自定义列延迟）。当前类型定义不变。

### 3.5 KanbanBoard.vue — 添加列保留简单行为

保持当前 `kanban.addColumn('新列')` 行为不变。自定义列的拖拽状态同步延迟处理。

### 3.6 KanbanColumn.vue — drop 位置计算（可选增强）

```typescript
function getDropIndex(e: DragEvent): number {
  const cardEls = (e.currentTarget as HTMLElement).querySelectorAll('.card');
  const mouseY = e.clientY;
  for (let i = 0; i < cardEls.length; i++) {
    const rect = cardEls[i].getBoundingClientRect();
    if (mouseY < rect.top + rect.height / 2) return i;
  }
  return cardEls.length;
}
```

---

## 4. 测试用例

### 新增

| ID | 场景 | 预期 |
|----|------|------|
| TC-KANBAN-006 | 拖到 inprogress | 调用 start_task；status=running；session 创建；刷新后仍在进行中 |
| TC-KANBAN-007 | 拖到 paused | 调用 pause_task；status=paused；session 关闭；刷新后仍在暂停中 |
| TC-KANBAN-008 | 拖到 done | 调用 complete_task；status=completed；信仰奖励发放；刷新后仍在已完成 |
| TC-KANBAN-009 | 拖到 todo | 调用 pause_task；status=paused；刷新后可能在 todo 或 paused（共享 status） |
| TC-KANBAN-010 | 同列内拖动 | 不触发任何后端请求 |
| TC-KANBAN-011 | 历史日期拖动 | 乐观更新 UI；不触发后端；无错误弹窗 |
| TC-KANBAN-012 | completed→inprogress 拖动 | 被静默跳过（终态不能重新开始） |
| TC-KANBAN-013 | 快速连续拖拽 A→B→C | last-user-action-wins；最终状态=C；中间返回结果被丢弃 |
| TC-KANBAN-014 | 后端拒绝后 UI 回滚 | 卡片移回原始列 |

### 更新已有

**TC-KANBAN-002**: 预期增加"后端 status 同步 + 刷新后位置持久"。

---

## 5. 文档修改

| 文档 | 修改内容 |
|------|----------|
| `docs/workflows.md` §5.2 | 重写拖拽流程：用业务命令替代 update_task；增加 reconcile 步骤 |
| `docs/api-contract.md` | 标注 `update_task` 不宜用于状态迁移 |
| `docs/ui-spec.md` | 无需修改（UI 表面不变） |
| `docs/testing/test-cases/kanban-process-binding.md` | 新增 TC-KANBAN-006~014；更新 TC-KANBAN-002 |
| `docs/tasks.md` | 新增 P0 任务 |

---

## 6. 风险和边界

| 风险 | 缓解 |
|------|------|
| `todo` 和 `paused` 共享 `paused` 状态 | 可接受；两者反映用户意图层级，不是数据差异 |
| `complete_task` 需要 actualMinutes | 默认用 estimatedMinutes，用户可在详情弹窗中修改 |
| 已完成/放弃任务拖回进行中 | 静默跳过 — 终态保护 |
| 虚拟每日任务 `daily:...` | 跳过 — 只有 start_task 能物化 |
| 项目任务 | 跳过 — 后端禁止 UI 修改 |
| 竞态（快速拖拽）| per-task 序号 + 丢弃过期结果 + 失败回滚 |
| 后端失败后 UI 分裂 | 失败时回滚卡片位置到原始列 |
| 自定义列拖拽 | 本次不处理；拖入自定义列只更新 localStorage，不同步后端 |

---

## 7. 执行步骤

| 步骤 | 内容 | 涉及文件 |
|------|------|----------|
| 1 | 修复 Mock 层 null 字段 | `api/task.ts` |
| 2 | 修复 loadBoard reconcile | `stores/kanban.ts` |
| 3 | 重写 moveCard（业务命令 + 竞态处理） | `stores/kanban.ts` |
| 4 | KanbanColumn drop 位置计算增强 | `components/kanban/KanbanColumn.vue` |
| 5 | 文档更新 | `workflows.md`, `test-cases/kanban-process-binding.md`, `tasks.md` |
| 6 | `npm run build` 验证 | — |
