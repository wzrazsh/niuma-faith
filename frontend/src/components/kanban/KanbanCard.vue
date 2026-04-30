<!-- frontend/src/components/kanban/KanbanCard.vue -->
<script setup lang="ts">
import { computed, ref, watch, onUnmounted } from 'vue';
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
const elapsedSeconds = ref(0);
let updateInterval: number | null = null;

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
      {{ formatElapsed(elapsedSeconds) }}
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
</style>
