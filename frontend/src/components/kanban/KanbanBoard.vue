<!-- frontend/src/components/kanban/KanbanBoard.vue -->
<script setup lang="ts">
import { onMounted, ref } from 'vue';
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
const showAddColumn = ref(false);
const newColumnTitle = ref('');

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

function handleAddColumn() {
  if (newColumnTitle.value.trim()) {
    store.addColumn(newColumnTitle.value.trim());
    newColumnTitle.value = '';
    showAddColumn.value = false;
  }
}

function handleDeleteColumn(columnId: string) {
  if (confirm('确定删除此列吗？列中的任务将移到待办列。')) {
    store.removeColumn(columnId);
  }
}
</script>

<template>
  <div class="kanban-board">
    <div v-if="store.isLoading" class="loading">加载中...</div>
    
    <template v-else>
      <div class="board-header">
        <button 
          v-if="!readonly" 
          class="add-column-btn" 
          @click="showAddColumn = !showAddColumn"
        >
          + 添加列
        </button>
      </div>

      <div v-if="showAddColumn && !readonly" class="add-column-form">
        <input 
          v-model="newColumnTitle" 
          placeholder="输入列标题" 
          @keyup.enter="handleAddColumn"
          ref="columnInput"
        />
        <button @click="handleAddColumn">确定</button>
        <button @click="showAddColumn = false">取消</button>
      </div>
      
      <div class="board-columns">
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
          @delete-column="handleDeleteColumn"
        />
      </div>
    </template>
  </div>
</template>

<style scoped>
.kanban-board {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.board-header {
  padding: 8px 16px;
  display: flex;
  justify-content: flex-end;
}

.add-column-btn {
  padding: 6px 12px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 0.875rem;
  color: var(--color-text);
  cursor: pointer;
  transition: all 0.15s;
}

.add-column-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.add-column-form {
  display: flex;
  gap: 8px;
  padding: 8px 16px;
  align-items: center;
}

.add-column-form input {
  padding: 6px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 0.875rem;
  background: var(--color-bg);
  color: var(--color-text);
}

.add-column-form button {
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.875rem;
  cursor: pointer;
  border: none;
  transition: opacity 0.15s;
}

.add-column-form button:first-of-type {
  background: var(--color-primary);
  color: #1a1a24;
}

.add-column-form button:last-child {
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  color: var(--color-text);
}

.add-column-form button:hover {
  opacity: 0.8;
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
  overflow-x: auto;
  flex: 1;
}
</style>
