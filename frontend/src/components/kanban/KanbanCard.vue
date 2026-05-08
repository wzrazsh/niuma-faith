<template>
  <div class="card" :draggable="!isHistorical" @dragstart="onDragStart"
    @dblclick="isHistorical ? null : $emit('edit', card.taskId)">
    <div class="card-info">
      <span class="card-title">{{ cardTitle }}</span>
      <span v-if="cardDetail" class="card-category" :class="cardDetail.category">
        {{ cardDetail.category }}
      </span>
    </div>
    <div v-if="timerRunning" class="card-timer">
      <span class="timer-dot"></span>
      <span>进行中</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useKanbanStore } from '@/stores/kanban';
import { useTaskStore } from '@/stores/task';

const props = defineProps<{ card: any; columnId: string }>();
defineEmits<{ edit: [id: string] }>();
const kanban = useKanbanStore();
const taskStore = useTaskStore();

const cardTitle = computed(() => {
  const task = kanban.taskMap[props.card.taskId];
  return task ? task.title : '加载中...';
});

const cardDetail = computed(() => {
  return kanban.taskMap[props.card.taskId] || null;
});

const isHistorical = computed(() => {
  const task = kanban.taskMap[props.card.taskId];
  if (!task) return false;
  return task.date < taskStore.selectedDate;
});

const timerRunning = computed(() => {
  return kanban.activeTimers.has(props.card.taskId);
});

function onDragStart(e: DragEvent) {
  e.dataTransfer?.setData('text/plain', props.card.taskId);
}
</script>

<style scoped>
.card {
  background: var(--color-bg);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-sm);
  padding: 10px 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.card:hover {
  border-color: var(--color-border);
  background: var(--color-bg-alt);
  transform: translateX(2px);
}

.card:active {
  cursor: grabbing;
}

.card-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.card-title {
  font-size: 0.82rem;
  font-weight: 500;
  color: var(--color-text);
  line-height: 1.4;
}

.card-category {
  font-size: 0.65rem;
  padding: 1px 5px;
  border-radius: 3px;
  font-weight: 600;
  text-transform: uppercase;
  align-self: flex-start;
}

.card-category.work { background: rgba(251, 114, 133, 0.15); color: var(--color-work); }
.card-category.study { background: rgba(96, 165, 250, 0.15); color: var(--color-study); }
.card-category.other { background: rgba(167, 139, 250, 0.15); color: var(--color-other); }

.card-timer {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 0.7rem;
  color: var(--color-primary);
}

.timer-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--color-primary);
  animation: glow-pulse 1.5s ease-in-out infinite;
}
</style>
