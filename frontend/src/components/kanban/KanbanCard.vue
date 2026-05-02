<!-- frontend/src/components/kanban/KanbanCard.vue -->
<script setup lang="ts">
import { computed, ref, watch, onUnmounted } from 'vue';
import type { Task, TaskStatus } from '@/types';
import type { ProcessBinding } from '@/types/kanban';
import { useKanbanStore } from '@/stores/kanban';

const props = defineProps<{
  task: Task;
  columnId?: string; // pass the column this card lives in
  readonly?: boolean;
  processBinding?: ProcessBinding;
  isProcessRunning?: boolean;
}>();

const emit = defineEmits<{
  (e: 'start', task: Task): void;
  (e: 'pause', task: Task): void;
  (e: 'complete', task: Task): void;
  (e: 'edit', task: Task): void;
  (e: 'delete', task: Task): void;
  (e: 'bind-process', task: Task, binding: ProcessBinding): void;
  (e: 'unbind-process', task: Task): void;
}>();

const store = useKanbanStore();
const elapsedSeconds = ref(0);
const showBindForm = ref(false);
const bindAppName = ref('');
const bindAutoStart = ref(true);
const bindAutoPause = ref(true);
let updateInterval: number | null = null;

const categoryLabel = computed(() => {
  if (props.task.category === 'work') return '工作';
  if (props.task.category === 'study') return '学习';
  return '其他';
});

const isActive = computed(() => props.task.status === ('running' as TaskStatus));

function formatMinutes(min: number): string {
  if (min < 60) return `${min}min`;
  const h = Math.floor(min / 60);
  const m = min % 60;
  return m > 0 ? `${h}h${m}m` : `${h}h`;
}

function formatElapsed(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) {
    return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }
  return `${m}:${s.toString().padStart(2, '0')}`;
}

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

function handleDragStart(e: DragEvent) {
  if (e.dataTransfer) {
    e.dataTransfer.setData('taskId', props.task.id);
    e.dataTransfer.effectAllowed = 'move';
  }
}

function handleBind() {
  const name = bindAppName.value.trim();
  if (!name) return;
  emit('bind-process', props.task, {
    appName: name,
    autoStart: bindAutoStart.value,
    autoPause: bindAutoPause.value,
  });
  showBindForm.value = false;
  bindAppName.value = '';
  bindAutoStart.value = true;
  bindAutoPause.value = true;
}

watch(() => props.task.status, (newStatus) => {
  if (newStatus === ('active' as TaskStatus) && store.activeTimers.has(props.task.id)) {
    startElapsedUpdate();
  } else {
    stopElapsedUpdate();
  }
}, { immediate: true });

onUnmounted(() => {
  stopElapsedUpdate();
});
</script>

<template>
  <div
    class="kanban-card"
    :class="{ active: isActive }"
    draggable="true"
    @dragstart="handleDragStart"
    @dblclick="emit('edit', task)"
  >
    <div class="card-header">
      <span class="card-title">{{ task.title }}</span>
      <span v-if="(task.recurrence_kind ?? 'none') === 'daily' || task.template_id" class="recurring-badge">每日</span>
      <span class="card-category" :data-cat="task.category">{{ categoryLabel }}</span>
    </div>
    
    <div class="card-meta">
      <span>预计 {{ formatMinutes(task.estimated_minutes) }}</span>
      <span v-if="task.status === 'completed'">，实际 {{ formatMinutes(task.actual_minutes) }}</span>
    </div>
    
    <div v-if="isActive && store.activeTimers.has(task.id)" class="card-timer">
      {{ formatElapsed(elapsedSeconds) }}
    </div>

    <div v-if="!readonly" class="card-process">
      <div v-if="processBinding" class="process-binding">
        <span class="process-dot" :class="{ running: isProcessRunning }" />
        <span class="process-name">{{ processBinding.appName }}</span>
        <template v-if="isProcessRunning">运行中</template>
        <template v-else>未运行</template>
        <button class="action-btn unbind" @click.stop="emit('unbind-process', task)">解绑</button>
      </div>
      <button v-else class="action-btn bind" @click.stop="showBindForm = !showBindForm">+ 绑定进程</button>

      <div v-if="showBindForm" class="bind-form">
        <input
          v-model="bindAppName"
          placeholder="进程名 (如 notepad.exe)"
          class="bind-input"
          @keyup.enter="handleBind"
        />
        <div class="bind-options">
          <label class="bind-check"><input v-model="bindAutoStart" type="checkbox" /> 自动开始</label>
          <label class="bind-check"><input v-model="bindAutoPause" type="checkbox" /> 自动暂停</label>
        </div>
        <div class="bind-buttons">
          <button class="action-btn bind-confirm" @click.stop="handleBind">确定</button>
          <button class="action-btn bind-cancel" @click.stop="showBindForm = false">取消</button>
        </div>
      </div>
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
        <button class="action-btn start" @click="emit('start', task)">
          {{ props.columnId === 'paused' ? '继续' : '开始' }}
        </button>
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
  background: rgba(59, 130, 246, 0.05);
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

.recurring-badge {
  display: inline-block;
  padding: 2px 8px;
  background: var(--color-primary);
  color: #1a1a24;
  border-radius: 12px;
  font-size: 0.75rem;
  font-weight: 600;
}

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
  font-family: monospace;
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

.card-process {
  margin-bottom: 8px;
}

.process-binding {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.process-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #666;
  flex-shrink: 0;
}

.process-dot.running {
  background: #10b981;
  box-shadow: 0 0 4px #10b981;
}

.process-name {
  font-family: monospace;
  font-weight: 500;
}

.action-btn.bind {
  background: transparent;
  border: 1px dashed var(--color-border);
  color: var(--color-text-muted);
  font-size: 0.75rem;
}

.action-btn.unbind {
  background: transparent;
  color: var(--color-text-muted);
  font-size: 0.7rem;
  padding: 2px 6px;
}

.bind-form {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.bind-input {
  padding: 4px 8px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  font-size: 0.75rem;
  background: var(--color-bg);
  color: var(--color-text);
}

.bind-input:focus {
  border-color: var(--color-primary);
  outline: none;
}

.bind-options {
  display: flex;
  gap: 12px;
}

.bind-check {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 0.7rem;
  color: var(--color-text-muted);
  cursor: pointer;
}

.bind-check input {
  accent-color: var(--color-primary);
}

.bind-buttons {
  display: flex;
  gap: 4px;
}

.action-btn.bind-confirm {
  background: var(--color-primary);
  color: #1a1a24;
  padding: 3px 10px;
  font-size: 0.7rem;
}

.action-btn.bind-cancel {
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  color: var(--color-text-muted);
  padding: 3px 10px;
  font-size: 0.7rem;
}
</style>
