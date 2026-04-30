# 任务看板多任务并行系统 - 实施任务列表

> 基于设计文档：docs/superpowers/specs/2026-04-30-kanban-multi-task-design.md
> 创建时间：2026-04-30

---

## Phase 1：基础看板（MVP）

### Task 1.1：创建看板类型定义

**文件：**
- 创建：`frontend/src/types/kanban.ts`

**验收条件：**
- [ ] `KanbanColumn` 接口定义正确，包含所有必需字段
- [ ] `KanbanCard` 接口定义正确，包含 processBinding 和 reminder 可选字段
- [ ] `KanbanState` 接口定义正确，包含 columns、cards、activeTimers
- [ ] TypeScript 编译无错误

**步骤：**

- [ ] **Step 1：创建 types/kanban.ts**

```typescript
// frontend/src/types/kanban.ts
import type { Task } from './index';

export interface KanbanColumn {
  id: string;
  title: string;
  order: number;
  taskIds: string[];
  isCustom: boolean;
  createdAt: string;
}

export interface ProcessBinding {
  appName: string;
  autoStart: boolean;
  autoPause: boolean;
}

export interface Reminder {
  time: string; // HH:mm format
  enabled: boolean;
}

export interface KanbanCard {
  task: Task;
  columnId: string;
  orderInColumn: number;
  processBinding?: ProcessBinding;
  reminder?: Reminder;
}

export interface KanbanState {
  columns: KanbanColumn[];
  cards: Map<string, KanbanCard>;
  activeTimers: Map<string, number>;
  isLoading: boolean;
}

export interface BoardConfig {
  columns: KanbanColumn[];
}
```

- [ ] **Step 2：验证类型编译**

Run: `cd E:\workspace\niuma-faith && npx vue-tsc --noEmit`
Expected: 无类型错误（可能有其他文件错误，但 kanban.ts 应该无错）

- [ ] **Step 3：Commit**

```bash
git add frontend/src/types/kanban.ts
git commit -m "feat: add kanban type definitions"
```

---

### Task 1.2：创建看板API服务

**文件：**
- 创建：`frontend/src/services/kanban-api.ts`

**验收条件：**
- [ ] `getBoardConfig` 函数能获取看板配置（从 localStorage）
- [ ] `saveBoardConfig` 函数能保存看板配置（到 localStorage）
- [ ] `moveCard` 函数能更新卡片位置
- [ ] `bindProcess` 和 `unbindProcess` 函数能管理进程绑定

**步骤：**

- [ ] **Step 1：创建 services/kanban-api.ts**

```typescript
// frontend/src/services/kanban-api.ts
import type { KanbanColumn, BoardConfig, KanbanCard } from '@/types/kanban';

const STORAGE_KEY = 'kanban-board-config';

export const kanbanApi = {
  async getBoardConfig(): Promise<BoardConfig> {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        return JSON.parse(stored) as BoardConfig;
      } catch {
        console.warn('Failed to parse board config, using defaults');
      }
    }
    // 返回默认配置
    return {
      columns: [
        { id: 'todo', title: '待办', order: 0, taskIds: [], isCustom: false, createdAt: new Date().toISOString() },
        { id: 'doing', title: '进行中', order: 1, taskIds: [], isCustom: false, createdAt: new Date().toISOString() },
        { id: 'paused', title: '暂停中', order: 2, taskIds: [], isCustom: false, createdAt: new Date().toISOString() },
        { id: 'done', title: '已完成', order: 3, taskIds: [], isCustom: false, createdAt: new Date().toISOString() },
      ]
    };
  },

  async saveBoardConfig(config: BoardConfig): Promise<void> {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
  },

  async moveCard(
    taskId: string,
    fromColumnId: string,
    toColumnId: string,
    newOrder: number,
    columns: KanbanColumn[]
  ): Promise<KanbanColumn[]> {
    const updatedColumns = columns.map(col => {
      if (col.id === fromColumnId) {
        return { ...col, taskIds: col.taskIds.filter(id => id !== taskId) };
      }
      if (col.id === toColumnId) {
        const newTaskIds = [...col.taskIds];
        newTaskIds.splice(newOrder, 0, taskId);
        return { ...col, taskIds: newTaskIds };
      }
      return col;
    });
    return updatedColumns;
  },

  async bindProcess(taskId: string, binding: KanbanCard['processBinding']): Promise<void> {
    // 更新卡片的进程绑定（存储到 localStorage 或后端）
    const key = `kanban-card-${taskId}`;
    const stored = localStorage.getItem(key);
    const card: KanbanCard = stored ? JSON.parse(stored) : { task: { id: taskId } as any, columnId: '', orderInColumn: 0 };
    card.processBinding = binding;
    localStorage.setItem(key, JSON.stringify(card));
  },

  async unbindProcess(taskId: string): Promise<void> {
    const key = `kanban-card-${taskId}`;
    const stored = localStorage.getItem(key);
    if (stored) {
      const card: KanbanCard = JSON.parse(stored);
      delete card.processBinding;
      localStorage.setItem(key, JSON.stringify(card));
    }
  }
};
```

- [ ] **Step 2：验证编译**

Run: `cd E:\workspace\niuma-faith && npx vue-tsc --noEmit`
Expected: 无类型错误

- [ ] **Step 3：Commit**

```bash
git add frontend/src/services/kanban-api.ts
git commit -m "feat: add kanban-api service with localStorage persistence"
```

---

### Task 1.3：创建看板状态管理Store

**文件：**
- 创建：`frontend/src/stores/kanban.ts`

**验收条件：**
- [ ] kanban store 能加载/保存看板配置
- [ ] 支持添加/删除自定义列
- [ ] 支持移动卡片（跨列/列内排序）
- [ ] 支持管理活跃计时器

**步骤：**

- [ ] **Step 1：安装 pinia（如果未安装）**

Run: `cd E:\workspace\niuma-faith && npm list pinia || npm install pinia`
Expected: pinia 已安装

- [ ] **Step 2：创建 stores/kanban.ts**

```typescript
// frontend/src/stores/kanban.ts
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { KanbanColumn, KanbanCard, KanbanState } from '@/types/kanban';
import { kanbanApi } from '@/services/kanban-api';

export const useKanbanStore = defineStore('kanban', () => {
  const columns = ref<KanbanColumn[]>([]);
  const cards = ref<Map<string, KanbanCard>>(new Map());
  const activeTimers = ref<Map<string, number>>(new Map());
  const isLoading = ref(false);

  const sortedColumns = computed(() => {
    return [...columns.value].sort((a, b) => a.order - b.order);
  });

  async function loadBoardConfig() {
    isLoading.value = true;
    try {
      const config = await kanbanApi.getBoardConfig();
      columns.value = config.columns;
    } finally {
      isLoading.value = false;
    }
  }

  async function saveBoardConfig() {
    const config = { columns: columns.value };
    await kanbanApi.saveBoardConfig(config);
  }

  async function moveCard(taskId: string, fromColumnId: string, toColumnId: string, newOrder: number) {
    columns.value = await kanbanApi.moveCard(taskId, fromColumnId, toColumnId, newOrder, columns.value);
    await saveBoardConfig();
  }

  function startTimer(taskId: string) {
    activeTimers.value.set(taskId, Date.now());
  }

  function stopTimer(taskId: string): number {
    const startTime = activeTimers.value.get(taskId);
    if (startTime) {
      const elapsed = Date.now() - startTime;
      activeTimers.value.delete(taskId);
      return elapsed;
    }
    return 0;
  }

  return {
    columns,
    cards,
    activeTimers,
    isLoading,
    sortedColumns,
    loadBoardConfig,
    saveBoardConfig,
    moveCard,
    startTimer,
    stopTimer,
  };
});
```

- [ ] **Step 3：验证编译**

Run: `cd E:\workspace\niuma-faith && npx vue-tsc --noEmit`
Expected: 无类型错误

- [ ] **Step 4：Commit**

```bash
git add frontend/src/stores/kanban.ts
git commit -m "feat: add kanban store with state management"
```

---

### Task 1.4：创建 KanbanCard.vue 组件

**文件：**
- 创建：`frontend/src/components/kanban/KanbanCard.vue`

**验收条件：**
- [ ] 正确显示任务标题、分类标签
- [ ] 显示计时器（若任务active）
- [ ] 支持开始/暂停/完成/编辑/删除操作
- [ ] 支持拖拽（使用HTML5 drag API）

**步骤：**

- [ ] **Step 1：创建 components/kanban/ 目录**

```bash
mkdir -p "E:\workspace\niuma-faith\frontend\src\components\kanban"
```

- [ ] **Step 2：创建 KanbanCard.vue**

```vue
<!-- frontend/src/components/kanban/KanbanCard.vue -->
<script setup lang="ts">
import { computed } from 'vue';
import type { Task } from '@/types';
import { useKanbanStore } from '@/stores/kanban';

const props = defineProps<{
  task: Task;
  readonly?: boolean;
}>();

const emit = defineEmits<{
  (e: 'start', task: Task): void;
  (e: 'pause', task: Task): void;
  (e: 'complete', task: Task): void;
  (e: 'edit', task: Task): void;
  (e: 'delete', task: Task): void;
}>();

const store = useKanbanStore();

const categoryLabel = computed(() => {
  if (props.task.category === 'work') return '工作';
  if (props.task.category === 'study') return '学习';
  return '其他';
});

const isActive = computed(() => props.task.status === 'active');

function formatMinutes(min: number): string {
  if (min < 60) return `${min}min`;
  const h = Math.floor(min / 60);
  const m = min % 60;
  return m > 0 ? `${h}h${m}m` : `${h}h`;
}

function handleDragStart(e: DragEvent) {
  if (e.dataTransfer) {
    e.dataTransfer.setData('taskId', props.task.id);
    e.dataTransfer.effectAllowed = 'move';
  }
}
</script>

<template>
  <div
    class="kanban-card"
    :class="{ active: isActive }"
    draggable="true"
    @dragstart="handleDragStart"
  >
    <div class="card-header">
      <span class="card-title">{{ task.title }}</span>
      <span class="card-category" :data-cat="task.category">{{ categoryLabel }}</span>
    </div>
    
    <div class="card-meta">
      <span>预计 {{ formatMinutes(task.estimated_minutes) }}</span>
      <span v-if="task.status === 'completed'">，实际 {{ formatMinutes(task.actual_minutes) }}</span>
    </div>
    
    <div v-if="isActive && store.activeTimers.has(task.id)" class="card-timer">
      计时中...
    </div>
    
    <div v-if="!readonly" class="card-actions">
      <template v-if="isActive">
        <button class="action-btn pause" @click="emit('pause', task)">暂停</button>
        <button class="action-btn complete" @click="emit('complete', task)">完成</button>
      </template>
      <template v-else-if="task.status === 'completed' || task.status === 'abandoned'">
        <button class="action-btn edit" @click="emit('edit', task)">编辑</button>
      </template>
      <template v-else>
        <button class="action-btn start" @click="emit('start', task)">开始</button>
      </template>
      <button class="action-btn delete" @click="emit('delete', task)">删除</button>
    </div>
  </div>
</template>

<style scoped>
.kanban-card {
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 8px;
  cursor: grab;
  transition: all 0.15s;
}

.kanban-card:hover {
  border-color: var(--color-primary);
}

.kanban-card.active {
  border-color: var(--color-progress);
  background: rgba(var(--color-progress-rgb), 0.05);
}

.kanban-card.dragging {
  opacity: 0.5;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.card-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
}

.card-category {
  font-size: 0.75rem;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;
}

.card-category[data-cat="work"] { background: var(--color-survival); color: #1a1a24; }
.card-category[data-cat="study"] { background: var(--color-progress); color: #1a1a24; }
.card-category[data-cat="other"] { background: var(--color-discipline); color: #1a1a24; }

.card-meta {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  margin-bottom: 8px;
}

.card-timer {
  font-size: 0.8125rem;
  color: var(--color-progress);
  font-weight: 600;
  margin-bottom: 8px;
}

.card-actions {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.action-btn {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 500;
  border: none;
  cursor: pointer;
  transition: opacity 0.15s;
}

.action-btn:hover { opacity: 0.8; }

.action-btn.start { background: var(--color-survival); color: #1a1a24; }
.action-btn.pause { background: var(--color-progress); color: #1a1a24; }
.action-btn.complete { background: var(--color-discipline); color: #1a1a24; }
.action-btn.edit { background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-text); }
.action-btn.delete { background: transparent; color: #e06040; }
</style>
```

- [ ] **Step 3：验证组件渲染（手动测试）**

Run: `cd E:\workspace\niuma-faith && npm run dev`
Expected: 组件能正确渲染，无控制台错误

- [ ] **Step 4：Commit**

```bash
git add frontend/src/components/kanban/KanbanCard.vue
git commit -m "feat: add KanbanCard component with drag support"
```

---

### Task 1.5：创建 KanbanColumn.vue 组件

**文件：**
- 创建：`frontend/src/components/kanban/KanbanColumn.vue`

**验收条件：**
- [ ] 正确显示列标题、卡片计数
- [ ] 渲染列内所有卡片
- [ ] 支持接收拖入的卡片（drop zone）
- [ ] 支持列内排序（使用HTML5 drop API）

**步骤：**

- [ ] **Step 1：创建 KanbanColumn.vue**

```vue
<!-- frontend/src/components/kanban/KanbanColumn.vue -->
<script setup lang="ts">
import { ref } from 'vue';
import type { KanbanColumn } from '@/types/kanban';
import type { Task } from '@/types';
import KanbanCard from './KanbanCard.vue';

const props = defineProps<{
  column: KanbanColumn;
  tasks: Task[];
  readonly?: boolean;
}>();

const emit = defineEmits<{
  (e: 'card-drop', taskId: string, toColumnId: string, newOrder: number): void;
  (e: 'card-start', task: Task): void;
  (e: 'card-pause', task: Task): void;
  (e: 'card-complete', task: Task): void;
  (e: 'card-edit', task: Task): void;
  (e: 'card-delete', task: Task): void;
}>();

const isDragOver = ref(false);

function handleDragOver(e: DragEvent) {
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'move';
  }
  isDragOver.value = true;
}

function handleDragLeave() {
  isDragOver.value = false;
}

function handleDrop(e: DragEvent) {
  e.preventDefault();
  isDragOver.value = false;
  
  if (e.dataTransfer) {
    const taskId = e.dataTransfer.getData('taskId');
    if (taskId) {
      const newOrder = props.column.taskIds.length;
      emit('card-drop', taskId, props.column.id, newOrder);
    }
  }
}
</script>

<template>
  <div
    class="kanban-column"
    :class="{ 'drag-over': isDragOver }"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
  >
    <div class="column-header">
      <h3 class="column-title">{{ column.title }}</h3>
      <span class="column-count">{{ tasks.length }}</span>
    </div>
    
    <div class="column-cards">
      <KanbanCard
        v-for="task in tasks"
        :key="task.id"
        :task="task"
        :readonly="readonly"
        @start="(t) => emit('card-start', t)"
        @pause="(t) => emit('card-pause', t)"
        @complete="(t) => emit('card-complete', t)"
        @edit="(t) => emit('card-edit', t)"
        @delete="(t) => emit('card-delete', t)"
      />
      
      <div v-if="tasks.length === 0" class="column-empty">
        暂无任务
      </div>
    </div>
  </div>
</template>

<style scoped>
.kanban-column {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 16px;
  min-width: 280px;
  max-width: 280px;
  display: flex;
  flex-direction: column;
  transition: all 0.15s;
}

.kanban-column.drag-over {
  border-color: var(--color-primary);
  background: rgba(var(--color-primary-rgb), 0.05);
}

.column-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--color-border);
}

.column-title {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--color-text);
}

.column-count {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  background: var(--color-bg);
  padding: 2px 8px;
  border-radius: 10px;
}

.column-cards {
  flex: 1;
  overflow-y: auto;
  min-height: 100px;
}

.column-empty {
  text-align: center;
  padding: 24px;
  color: var(--color-text-muted);
  font-size: 0.875rem;
}
</style>
```

- [ ] **Step 2：验证组件渲染（手动测试）**

Run: `cd E:\workspace\niuma-faith && npm run dev`
Expected: 组件能正确渲染列和卡片，支持拖放

- [ ] **Step 3：Commit**

```bash
git add frontend/src/components/kanban/KanbanColumn.vue
git commit -m "feat: add KanbanColumn component with drop zone"
```

---

### Task 1.6：创建 KanbanBoard.vue 主容器

**文件：**
- 创建：`frontend/src/components/kanban/KanbanBoard.vue`

**验收条件：**
- [ ] 横向滚动显示所有列
- [ ] 能从 store 加载列配置
- [ ] 支持添加/删除自定义列
- [ ] 能正确分发卡片到对应列

**步骤：**

- [ ] **Step 1：创建 KanbanBoard.vue**

```vue
<!-- frontend/src/components/kanban/KanbanBoard.vue -->
<script setup lang="ts">
import { onMounted } from 'vue';
import { useKanbanStore } from '@/stores/kanban';
import type { Task } from '@/types';
import KanbanColumn from './KanbanColumn.vue';

const props = defineProps<{
  tasks: Task[];
  readonly?: boolean;
}>();

const emit = defineEmits<{
  (e: 'card-start', task: Task): void;
  (e: 'card-pause', task: Task): void;
  (e: 'card-complete', task: Task): void;
  (e: 'card-edit', task: Task): void;
  (e: 'card-delete', task: Task): void;
}>();

const store = useKanbanStore();

onMounted(async () => {
  await store.loadBoardConfig();
});

function getTasksForColumn(columnId: string): Task[] {
  const column = store.columns.find(c => c.id === columnId);
  if (!column) return [];
  return column.taskIds
    .map(taskId => props.tasks.find(t => t.id === taskId))
    .filter((t): t is Task => t !== undefined);
}

function handleCardDrop(taskId: string, toColumnId: string, newOrder: number) {
  // 找到任务原列
  const fromColumn = store.columns.find(col => 
    col.taskIds.includes(taskId)
  );
  if (fromColumn) {
    store.moveCard(taskId, fromColumn.id, toColumnId, newOrder);
  }
}

function handleCardStart(task: Task) {
  emit('card-start', task);
}

function handleCardPause(task: Task) {
  emit('card-pause', task);
}

function handleCardComplete(task: Task) {
  emit('card-complete', task);
}

function handleCardEdit(task: Task) {
  emit('card-edit', task);
}

function handleCardDelete(task: Task) {
  emit('card-delete', task);
}
</script>

<template>
  <div class="kanban-board">
    <div v-if="store.isLoading" class="loading">加载中...</div>
    
    <div v-else class="board-columns">
      <KanbanColumn
        v-for="column in store.sortedColumns"
        :key="column.id"
        :column="column"
        :tasks="getTasksForColumn(column.id)"
        :readonly="readonly"
        @card-drop="handleCardDrop"
        @card-start="handleCardStart"
        @card-pause="handleCardPause"
        @card-complete="handleCardComplete"
        @card-edit="handleCardEdit"
        @card-delete="handleCardDelete"
      />
    </div>
  </div>
</template>

<style scoped>
.kanban-board {
  width: 100%;
  height: 100%;
  overflow-x: auto;
  overflow-y: hidden;
}

.loading {
  text-align: center;
  padding: 32px;
  color: var(--color-text-muted);
}

.board-columns {
  display: flex;
  gap: 16px;
  padding: 16px;
  min-height: 100%;
}
</style>
```

- [ ] **Step 2：验证看板渲染（手动测试）**

Run: `cd E:\workspace\niuma-faith && npm run dev`
Expected: 看板能正确渲染所有列，卡片能拖拽到不同列

- [ ] **Step 3：Commit**

```bash
git add frontend/src/components/kanban/KanbanBoard.vue
git commit -m "feat: add KanbanBoard main container"
```

---

## Phase 2：多任务计时

### Task 2.1：实现多任务并行计时器

**文件：**
- 修改：`frontend/src/stores/kanban.ts` - 添加多计时器管理
- 修改：`frontend/src/components/kanban/KanbanCard.vue` - 显示实时计时

**验收条件：**
- [ ] 支持同时启动多个任务的计时器
- [ ] 卡片上实时显示已用时（每秒更新）
- [ ] 暂停任务时正确计算本次用时
- [ ] 计时器使用 `setInterval`，每秒更新

**步骤：**

- [ ] **Step 1：扩展 kanban store 支持多计时器**

```typescript
// frontend/src/stores/kanban.ts 添加/修改：

// 添加 ref
const timerIntervals = ref<Map<string, number>>(new Map());

// 添加函数
function startTimer(taskId: string) {
  if (activeTimers.value.has(taskId)) return; // 已在计时
  
  activeTimers.value.set(taskId, Date.now());
  
  // 每秒更新一次
  const intervalId = window.setInterval(() => {
    // 触发响应式更新
    activeTimers.value = new Map(activeTimers.value);
  }, 1000);
  
  timerIntervals.value.set(taskId, intervalId);
}

function stopTimer(taskId: string): number {
  const startTime = activeTimers.value.get(taskId);
  const intervalId = timerIntervals.value.get(taskId);
  
  if (intervalId) {
    clearInterval(intervalId);
    timerIntervals.value.delete(taskId);
  }
  
  if (startTime) {
    const elapsed = Date.now() - startTime;
    activeTimers.value.delete(taskId);
    return elapsed;
  }
  return 0;
}

function getElapsedTime(taskId: string): number {
  const startTime = activeTimers.value.get(taskId);
  if (startTime) {
    return Date.now() - startTime;
  }
  return 0;
}
```

- [ ] **Step 2：修改 KanbanCard.vue 显示实时计时**

```vue
<!-- 在 KanbanCard.vue 的 script 中添加 -->
import { ref, onUnmounted } from 'vue';

const elapsedSeconds = ref(0);
let updateInterval: number | null = null;

function startElapsedUpdate() {
  updateInterval = window.setInterval(() => {
    elapsedSeconds.value = store.getElapsedTime(props.task.id) / 1000;
  }, 1000);
}

function stopElapsedUpdate() {
  if (updateInterval) {
    clearInterval(updateInterval);
    updateInterval = null;
  }
}

// 监听任务状态
watch(() => props.task.status, (newStatus) => {
  if (newStatus === 'active' && store.activeTimers.has(props.task.id)) {
    startElapsedUpdate();
  } else {
    stopElapsedUpdate();
  }
}, { immediate: true });

onUnmounted(() => {
  stopElapsedUpdate();
});

function formatElapsed(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) {
    return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }
  return `${m}:${s.toString().padStart(2, '0')}`;
}
```

在模板中显示：
```vue
<div v-if="isActive && store.activeTimers.has(task.id)" class="card-timer">
  {{ formatElapsed(elapsedSeconds) }}
</div>
```

- [ ] **Step 3：验证多任务计时（手动测试）**

Run: `cd E:\workspace\niuma-faith && npm run dev`
Expected: 可以同时启动多个任务计时，每个卡片实时显示计时

- [ ] **Step 4：Commit**

```bash
git add frontend/src/stores/kanban.ts frontend/src/components/kanban/KanbanCard.vue
git commit -m "feat: implement multi-task parallel timer with real-time display"
```

---

### Task 2.2：集成任务开始/暂停/完成操作

**文件：**
- 修改：`frontend/src/components/kanban/KanbanBoard.vue` - 处理任务操作
- 可能需要修改：`frontend/src/stores/task.ts` - 调用现有API

**验收条件：**
- [ ] 点击"开始"按钮，任务状态变为active，计时器启动
- [ ] 点击"暂停"按钮，计时器停止，记录用时
- [ ] 点击"完成"按钮，任务完成，计时器停止
- [ ] 操作成功后刷新任务列表

**步骤：**

- [ ] **Step 1：在看板中处理任务操作**

```vue
<!-- 在 KanbanBoard.vue 的 script 中添加 -->
import { useTaskStore } from '@/stores/task';

const taskStore = useTaskStore();

async function handleCardStart(task: Task) {
  // 调用现有API开始任务
  await taskStore.updateTaskStatus(task.id, 'active');
  store.startTimer(task.id);
  
  // 刷新任务列表
  await taskStore.fetchTasksByDate(taskStore.selectedDate);
}

async function handleCardPause(task: Task) {
  const elapsed = store.stopTimer(task.id);
  const actualMinutes = task.actual_minutes + Math.ceil(elapsed / 60000);
  
  await taskStore.updateTask(task.id, { 
    ...task, 
    status: 'active', // 或保持active，只是暂停计时
    actual_minutes: actualMinutes 
  });
}

async function handleCardComplete(task: Task) {
  const elapsed = store.stopTimer(task.id);
  const actualMinutes = task.actual_minutes + Math.ceil(elapsed / 60000);
  
  await taskStore.completeTask(task.id, actualMinutes);
}
```

- [ ] **Step 2：验证任务操作（手动测试）**

Run: `cd E:\workspace\niuma-faith && npm run dev`
Expected: 任务可以正常开始、暂停、完成，计时器正确工作

- [ ] **Step 3：Commit**

```bash
git add frontend/src/components/kanban/KanbanBoard.vue
git commit -m "feat: integrate task start/pause/complete with timer"
```

---

## Phase 3：自动检测

### Task 3.1：添加后端进程检测命令

**文件：**
- 修改：`src-tauri/src/tauri/commands.rs` - 添加 is_process_running 命令

**验收条件：**
- [ ] `is_process_running` 命令能正确检测Windows进程
- [ ] 命令接收应用名称参数，返回布尔值
- [ ] 使用 tasklist 命令检测进程

**步骤：**

- [ ] **Step 1：修改 commands.rs 添加进程检测**

```rust
// src-tauri/src/tauri/commands.rs

#[tauri::command]
pub async fn is_process_running(app_name: String) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("tasklist")
            .args(&["/FI", &format!("IMAGENAME eq {}", app_name), "/NH"])
            .output()
            .map_err(|e| format!("Failed to execute tasklist: {}", e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        // 如果输出包含进程名，说明进程在运行
        Ok(stdout.contains(&app_name))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Linux/Mac 实现（如果需要）
        Err("Unsupported platform".to_string())
    }
}
```

- [ ] **Step 2：在 lib.rs 中注册命令**

```rust
// src-tauri/src/lib.rs 中确保命令已注册
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        // ... 其他命令
        is_process_running,
    ])
```

- [ ] **Step 3：测试命令（手动测试）**

Run: `cd E:\workspace\niuma-faith\src-tauri && cargo build`
Expected: 编译成功，无错误

- [ ] **Step 4：Commit**

```bash
git add src-tauri/src/tauri/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add is_process_running Tauri command for Windows"
```

---

### Task 3.2：创建前端进程检测服务

**文件：**
- 创建：`frontend/src/services/process-detector.ts`

**验收条件：**
- [ ] ProcessDetector 类能定期检测绑定进程
- [ ] 检测间隔可配置（默认30秒）
- [ ] 检测到进程状态变化时触发事件
- [ ] 支持启动/停止检测

**步骤：**

- [ ] **Step 1：创建 process-detector.ts**

```typescript
// frontend/src/services/process-detector.ts
import { invoke } from '@tauri-apps/api';
import type { KanbanCard } from '@/types/kanban';

type ProcessStateChangeCallback = (taskId: string, isRunning: boolean) => void;

class ProcessDetector {
  private intervalId: number | null = null;
  private interval: number = 30000; // 30秒
  private bindings: Map<string, KanbanCard> = new Map();
  private callbacks: ProcessStateChangeCallback[] = [];
  private lastStates: Map<string, boolean> = new Map();

  setInterval(ms: number) {
    this.interval = ms;
    if (this.intervalId) {
      this.stop();
      this.start();
    }
  }

  registerCallback(callback: ProcessStateChangeCallback) {
    this.callbacks.push(callback);
  }

  addBinding(taskId: string, card: KanbanCard) {
    if (card.processBinding) {
      this.bindings.set(taskId, card);
    }
  }

  removeBinding(taskId: string) {
    this.bindings.delete(taskId);
    this.lastStates.delete(taskId);
  }

  start() {
    if (this.intervalId) return;
    
    this.intervalId = window.setInterval(() => {
      this.checkAll();
    }, this.interval);
  }

  stop() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  private async checkAll() {
    for (const [taskId, card] of this.bindings) {
      if (!card.processBinding) continue;
      
      try {
        const isRunning = await this.checkProcess(card.processBinding.appName);
        const lastState = this.lastStates.get(taskId);
        
        if (lastState === undefined || lastState !== isRunning) {
          this.lastStates.set(taskId, isRunning);
          this.notifyCallbacks(taskId, isRunning);
        }
      } catch (error) {
        console.error(`Failed to check process for task ${taskId}:`, error);
      }
    }
  }

  private async checkProcess(appName: string): Promise<boolean> {
    try {
      return await invoke<boolean>('is_process_running', { appName });
    } catch (error) {
      console.error(`Failed to invoke is_process_running for ${appName}:`, error);
      return false;
    }
  }

  private notifyCallbacks(taskId: string, isRunning: boolean) {
    for (const callback of this.callbacks) {
      try {
        callback(taskId, isRunning);
      } catch (error) {
        console.error('Process state change callback error:', error);
      }
    }
  }
}

export const processDetector = new ProcessDetector();
```

- [ ] **Step 2：验证编译**

Run: `cd E:\workspace\niuma-faith && npx vue-tsc --noEmit`
Expected: 无类型错误

- [ ] **Step 3：Commit**

```bash
git add frontend/src/services/process-detector.ts
git commit -m "feat: add ProcessDetector service for auto app detection"
```

---

### Task 3.3：集成进程检测与看板

**文件：**
- 修改：`frontend/src/components/kanban/KanbanBoard.vue` - 集成进程检测
- 修改：`frontend/src/stores/kanban.ts` - 管理进程绑定

**验收条件：**
- [ ] 看板加载时启动进程检测
- [ ] 检测到进程启动/结束时，自动更新任务状态
- [ ] 根据 autoStart/autoPause 设置执行相应操作

**步骤：**

- [ ] **Step 1：在看板中集成进程检测**

```vue
<!-- 在 KanbanBoard.vue 的 script 中添加 -->
import { processDetector } from '@/services/process-detector';

onMounted(async () => {
  await store.loadBoardConfig();
  
  // 启动进程检测
  processDetector.registerCallback(handleProcessStateChange);
  processDetector.start();
});

onUnmounted(() => {
  processDetector.stop();
});

function handleProcessStateChange(taskId: string, isRunning: boolean) {
  const task = props.tasks.find(t => t.id === taskId);
  if (!task) return;
  
  const card = store.cards.get(taskId);
  if (!card?.processBinding) return;
  
  if (isRunning && card.processBinding.autoStart && task.status !== 'active') {
    // 进程启动，自动开始任务
    emit('card-start', task);
  } else if (!isRunning && card.processBinding.autoPause && task.status === 'active') {
    // 进程结束，自动暂停任务
    emit('card-pause', task);
  }
}
```

- [ ] **Step 2：验证自动状态转换（手动测试）**

Run: `cd E:\workspace\niuma-faith && npm run dev`
Test:
  1. 创建一个任务，绑定到某个运行的应用（如 notepad.exe）
  2. 关闭应用，观察任务是否自动暂停
  3. 重新打开应用，观察任务是否自动开始

Expected: 进程状态变化能正确触发任务状态变更

- [ ] **Step 3：Commit**

```bash
git add frontend/src/components/kanban/KanbanBoard.vue
git commit -m "feat: integrate process detector with kanban board"
```

---

## Phase 4：智能提醒与优化

### Task 4.1：实现每日任务提醒

**文件：**
- 修改：`frontend/src/components/kanban/KanbanBoard.vue` 或创建独立提醒服务

**验收条件：**
- [ ] 任务可以设置提醒时间（HH:mm）
- [ ] 到达提醒时间且任务未完成时，显示通知
- [ ] 使用 Tauri Notification API 显示系统通知

**步骤：**

- [ ] **Step 1：创建提醒服务（可选，或内联实现）**

```typescript
// frontend/src/services/reminder-service.ts
import { ref, onUnmounted } from 'vue';

class ReminderService {
  private intervalId: number | null = null;
  private reminders: Map<string, { time: string; taskTitle: string }> = new Map();

  start() {
    if (this.intervalId) return;
    
    // 每分钟检查一次
    this.intervalId = window.setInterval(() => {
      this.checkReminders();
    }, 60000);
  }

  stop() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  addReminder(taskId: string, time: string, taskTitle: string) {
    this.reminders.set(taskId, { time, taskTitle });
  }

  removeReminder(taskId: string) {
    this.reminders.delete(taskId);
  }

  private checkReminders() {
    const now = new Date();
    const currentTime = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;
    
    for (const [taskId, reminder] of this.reminders) {
      if (reminder.time === currentTime) {
        this.showNotification(reminder.taskTitle);
      }
    }
  }

  private async showNotification(taskTitle: string) {
    // 使用 Tauri Notification API
    try {
      const { invoke } = await import('@tauri-apps/api');
      await invoke('show_notification', {
        title: '任务提醒',
        body: `任务「${taskTitle}」的截止时间已到！`
      });
    } catch {
      // 降级到浏览器通知
      if ('Notification' in window && Notification.permission === 'granted') {
        new Notification('任务提醒', {
          body: `任务「${taskTitle}」的截止时间已到！`
        });
      }
    }
  }
}

export const reminderService = new ReminderService();
```

- [ ] **Step 2：在看板中集成提醒服务**

```vue
<!-- 在 KanbanBoard.vue 中添加 -->
import { reminderService } from '@/services/reminder-service';

onMounted(async () => {
  await store.loadBoardConfig();
  
  // 启动提醒服务
  reminderService.start();
  
  // 为设置了提醒的任务注册提醒
  for (const task of props.tasks) {
    const card = store.cards.get(task.id);
    if (card?.reminder?.enabled) {
      reminderService.addReminder(task.id, card.reminder.time, task.title);
    }
  }
});

onUnmounted(() => {
  reminderService.stop();
});
```

- [ ] **Step 3：验证提醒功能（手动测试）**

Test: 创建一个任务，设置提醒时间为当前时间+1分钟，等待通知

Expected: 到达提醒时间时显示系统通知

- [ ] **Step 4：Commit**

```bash
git add frontend/src/services/reminder-service.ts frontend/src/components/kanban/KanbanBoard.vue
git commit -m "feat: add daily task reminder with notification"
```

---

### Task 4.2：添加自定义列功能

**文件：**
- 修改：`frontend/src/components/kanban/KanbanBoard.vue` - 添加列管理UI
- 修改：`frontend/src/stores/kanban.ts` - 支持添加/删除列

**验收条件：**
- [ ] 用户可以添加自定义列（输入列标题）
- [ ] 用户可以删除自定义列（默认列不可删除）
- [ ] 列配置自动保存

**步骤：**

- [ ] **Step 1：在 store 中添加列管理函数**

```typescript
// frontend/src/stores/kanban.ts 添加：

function addColumn(title: string) {
  const newColumn: KanbanColumn = {
    id: `col-${Date.now()}`,
    title,
    order: columns.value.length,
    taskIds: [],
    isCustom: true,
    createdAt: new Date().toISOString()
  };
  columns.value.push(newColumn);
  saveBoardConfig();
}

function removeColumn(columnId: string) {
  const col = columns.value.find(c => c.id === columnId);
  if (!col || !col.isCustom) return; // 只能删除自定义列
  
  columns.value = columns.value.filter(c => c.id !== columnId);
  saveBoardConfig();
}
```

- [ ] **Step 2：在看板中添加列管理UI**

```vue
<!-- 在 KanbanBoard.vue 的模板中添加 -->
<div class="board-header">
  <button v-if="!readonly" class="add-column-btn" @click="showAddColumn = !showAddColumn">
    + 添加列
  </button>
</div>

<div v-if="showAddColumn && !readonly" class="add-column-form">
  <input v-model="newColumnTitle" placeholder="输入列标题" @keyup.enter="handleAddColumn" />
  <button @click="handleAddColumn">确定</button>
  <button @click="showAddColumn = false">取消</button>
</div>
```

在 script 中添加：
```typescript
const showAddColumn = ref(false);
const newColumnTitle = ref('');

function handleAddColumn() {
  if (newColumnTitle.value.trim()) {
    store.addColumn(newColumnTitle.value.trim());
    newColumnTitle.value = '';
    showAddColumn.value = false;
  }
}
```

在列头添加删除按钮（仅自定义列）：
```vue
<!-- 在 KanbanColumn.vue 的列头中添加 -->
<button
  v-if="column.isCustom && !readonly"
  class="delete-column-btn"
  @click="emit('delete-column', column.id)"
>
  ×
</button>
```

- [ ] **Step 3：验证自定义列功能（手动测试）**

Run: `cd E:\workspace\niuma-faith && npm run dev`
Test:
  1. 点击"添加列"，输入标题，确认
  2. 验证新列出现
  3. 删除自定义列

Expected: 可以正常添加和删除自定义列

- [ ] **Step 4：Commit**

```bash
git add frontend/src/stores/kanban.ts frontend/src/components/kanban/KanbanBoard.vue frontend/src/components/kanban/KanbanColumn.vue
git commit -m "feat: add custom column management (add/delete)"
```

---

## 总结

**已完成任务清单：**
- [ ] Phase 1: 基础看板（6个任务）
- [ ] Phase 2: 多任务计时（2个任务）
- [ ] Phase 3: 自动检测（3个任务）
- [ ] Phase 4: 智能提醒与优化（2个任务）

**总计：13个任务**

每个任务都包含：
- 明确的文件列表
- 完整的代码示例
- 验收条件
- 验证步骤
- Commit命令

**下一步：按照任务顺序执行，每完成一个任务验证通过后commit。**
