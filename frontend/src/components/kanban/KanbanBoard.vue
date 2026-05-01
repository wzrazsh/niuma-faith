<!-- frontend/src/components/kanban/KanbanBoard.vue -->
<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { useKanbanStore } from '@/stores/kanban';
import { useTaskStore } from '@/stores/task';
import type { Task, TaskStatus } from '@/types';
import type { ProcessBinding, KanbanCard as KanbanCardType } from '@/types/kanban';
import KanbanColumn from './KanbanColumn.vue';
import KanbanCardForm from './KanbanCardForm.vue';
import { reminderService } from '@/services/reminder-service';
import { processDetector } from '@/services/process-detector';
import { kanbanApi } from '@/services/kanban-api';

const props = defineProps<{
  tasks: Task[];
  readonly?: boolean;
}>();

const emit = defineEmits<{
  (e: 'refresh'): void;
}>();

const store = useKanbanStore();
const taskStore = useTaskStore();
const showAddColumn = ref(false);
const newColumnTitle = ref('');
const showForm = ref(false);
const editingTask = ref<Task | undefined>(undefined);
const selectedColumnId = ref<string | undefined>(undefined);

const cardConfigs = ref<Map<string, KanbanCardType>>(new Map());
const processRunning = ref<Map<string, boolean>>(new Map());
const pollingCleanups = ref<Map<string, () => void>>(new Map());

onMounted(async () => {
  await store.loadBoardConfig();
  await loadCardConfigs();
  reminderService.start();
  registerTaskReminders();
});

onUnmounted(() => {
  reminderService.stop();
  stopAllPolling();
});

// 监听任务变化，重新注册提醒
watch(() => props.tasks, () => {
  loadCardConfigs();
  registerTaskReminders();
});

function registerTaskReminders() {
  reminderService.stop();
  reminderService.start();
  
  for (const task of props.tasks) {
    const card = cardConfigs.value.get(task.id);
    if (card?.reminder?.enabled && card.reminder.time) {
      reminderService.addReminder(task.id, card.reminder.time, task.title);
    }
  }
}

function loadCardConfigs() {
  const newMap = new Map<string, KanbanCardType>();
  for (const task of props.tasks) {
    try {
      const key = `kanban-card-${task.id}`;
      const stored = localStorage.getItem(key);
      if (stored) {
        const card: KanbanCardType = JSON.parse(stored);
        newMap.set(task.id, card);
        if (card.processBinding) {
          processRunning.value.set(task.id, false);
          startProcessPolling(task.id, card.processBinding);
        }
      }
    } catch (e) {
      console.error("Failed to load card config for", task.id, e);
    }
  }
  cardConfigs.value = newMap;
}

function startProcessPolling(taskId: string, binding: ProcessBinding) {
  const cleanup = processDetector.startPolling(
    binding.appName,
    3000,
    (running) => {
      processRunning.value.set(taskId, running);
      const task = props.tasks.find((t) => t.id === taskId);
      if (!task) return;

      if (running && binding.autoStart && task.status !== ("active" as TaskStatus)) {
        handleCardStart(task);
      } else if (!running && binding.autoPause && task.status === ("active" as TaskStatus)) {
        handleCardPause(task);
      }
    }
  );
  pollingCleanups.value.set(taskId, cleanup);
}

function stopProcessPolling(taskId: string) {
  const cleanup = pollingCleanups.value.get(taskId);
  if (cleanup) {
    cleanup();
    pollingCleanups.value.delete(taskId);
  }
  processRunning.value.delete(taskId);
}

function stopAllPolling() {
  for (const cleanup of pollingCleanups.value.values()) {
    cleanup();
  }
  pollingCleanups.value.clear();
  processRunning.value.clear();
}

function handleBindProcess(task: Task, binding: ProcessBinding) {
  kanbanApi.bindProcess(task.id, binding);
  const card = cardConfigs.value.get(task.id) || {
    task,
    columnId: "",
    orderInColumn: 0,
  };
  card.processBinding = binding;
  cardConfigs.value.set(task.id, card);
  processRunning.value.set(task.id, false);
  startProcessPolling(task.id, binding);
}

function handleUnbindProcess(task: Task) {
  kanbanApi.unbindProcess(task.id);
  stopProcessPolling(task.id);
  const card = cardConfigs.value.get(task.id);
  if (card) {
    delete card.processBinding;
    cardConfigs.value.set(task.id, card);
  }
}

function getProcessBinding(taskId: string): ProcessBinding | undefined {
  return cardConfigs.value.get(taskId)?.processBinding;
}

function getTasksForColumn(columnId: string): Task[] {
  const column = store.columns.find(c => c.id === columnId);
  if (!column) return [];
  return column.taskIds
    .map(taskId => props.tasks.find(t => t.id === taskId))
    .filter((t): t is Task => t !== undefined);
}

function getColumnProcessBindings(columnId: string): Map<string, ProcessBinding | undefined> {
  const result = new Map<string, ProcessBinding | undefined>();
  const column = store.columns.find(c => c.id === columnId);
  if (!column) return result;
  for (const taskId of column.taskIds) {
    result.set(taskId, getProcessBinding(taskId));
  }
  return result;
}

function handleCardDrop(taskId: string, toColumnId: string, newOrder: number) {
  const fromColumn = store.columns.find(col => 
    col.taskIds.includes(taskId)
  );
  if (fromColumn) {
    store.moveCard(taskId, fromColumn.id, toColumnId, newOrder);
  }
}

async function handleCardStart(task: Task) {
  try {
    await taskStore.updateTask(task.id, undefined, undefined, undefined, undefined, undefined, 'active' as TaskStatus);
    store.moveToColumn(task.id, 'doing');
    store.startTimer(task.id);
    emit('refresh');
  } catch (error) {
    console.error('Failed to start task:', error);
  }
}

async function handleCardPause(task: Task) {
  try {
    const elapsed = store.stopTimer(task.id);
    const actualMinutes = task.actual_minutes + Math.ceil(elapsed / 60000);
    
    await taskStore.updateTask(task.id, undefined, undefined, undefined, actualMinutes, undefined, 'paused');
    store.moveToColumn(task.id, 'paused');
    emit('refresh');
  } catch (error) {
    console.error('Failed to pause task:', error);
  }
}

async function handleCardComplete(task: Task) {
  try {
    const elapsed = store.stopTimer(task.id);
    const actualMinutes = task.actual_minutes + Math.ceil(elapsed / 60000);
    
    await taskStore.completeTask(task.id, actualMinutes);
    store.moveToColumn(task.id, 'done');
    emit('refresh');
  } catch (error) {
    console.error('Failed to complete task:', error);
  }
}

function handleCardEdit(task: Task) {
  editingTask.value = task;
  showForm.value = true;
}

function openCreateForm(columnId?: string) {
  editingTask.value = undefined;
  selectedColumnId.value = columnId;
  showForm.value = true;
}

function handleFormClose() {
  showForm.value = false;
  editingTask.value = undefined;
}

function handleFormSaved(columnId?: string) {
  const wasEditing = !!editingTask.value;
  showForm.value = false;
  editingTask.value = undefined;
  // 将新建的任务添加到对应列
  if (!wasEditing && columnId) {
    const newTask = taskStore.tasks[taskStore.tasks.length - 1];
    if (newTask) {
      store.addCardToColumn(newTask.id, columnId);
    }
  }
  emit('refresh');
}

async function handleCardDelete(task: Task) {
  if (confirm(`确定删除任务「${task.title}」？`)) {
    try {
      await taskStore.deleteTask(task.id);
      emit('refresh');
    } catch (error) {
      console.error('Failed to delete task:', error);
    }
  }
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
        <div class="header-left">
          <button 
            v-if="!readonly" 
            class="add-task-btn" 
            @click="openCreateForm()"
          >
            + 创建任务
          </button>
          <button 
            v-if="!readonly" 
            class="add-column-btn" 
            @click="showAddColumn = !showAddColumn"
          >
            + 添加列
          </button>
        </div>
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
          :process-bindings="getColumnProcessBindings(column.id)"
          :process-running="processRunning"
          @card-drop="handleCardDrop"
          @card-start="handleCardStart"
          @card-pause="handleCardPause"
          @card-complete="handleCardComplete"
          @card-edit="handleCardEdit"
          @card-delete="handleCardDelete"
          @delete-column="handleDeleteColumn"
          @add-task="openCreateForm(column.id)"
          @bind-process="handleBindProcess"
          @unbind-process="handleUnbindProcess"
        />
      </div>
    </template>
    
    <KanbanCardForm
      v-if="showForm && !readonly"
      :task="editingTask"
      :column-id="selectedColumnId"
      @close="handleFormClose"
      @saved="handleFormSaved"
    />
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
