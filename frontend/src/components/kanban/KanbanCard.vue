<template>
  <div
    v-if="task"
    class="kanban-card"
    draggable="true"
    @dragstart="onDragStart"
  >
    <div class="card-title">{{ task.title }}</div>
    <div class="card-meta">{{ task.category }} | 预计 {{ task.estimated_minutes }}分钟</div>
    <div v-if="task.status === 'running'" class="card-timer">计时中: {{ formatDuration(task.duration_seconds) }}</div>
    <div class="card-actions" v-if="task.status !== 'completed' && task.status !== 'abandoned'">
      <button v-if="task.status === 'paused'" class="primary" @click="doStart">开始</button>
      <button v-if="task.status === 'running'" @click="doPause">暂停</button>
      <button v-if="task.status === 'running'" class="success" @click="doComplete">完成</button>
      <button v-if="task.status === 'running'" class="danger" @click="doAbandon">放弃</button>
      <button v-if="task.status === 'paused'" class="success" @click="doResume">继续</button>
    </div>
    <div v-if="card.processBinding" class="card-binding">📎 {{ card.processBinding.appName }}</div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue';
import type { KanbanCard } from '@/types/kanban';
import { useTaskStore } from '@/stores/task';
import { useFaithStore } from '@/stores/faith';
import { formatDuration } from '@/utils/format';

const props = defineProps<{ card: KanbanCard; index?: number }>();
const emit = defineEmits<{ dragStart: [cardId: string] }>();

const taskStore = useTaskStore();
const faith = useFaithStore();

const task = computed(() => taskStore.tasks.find(t => t.id === props.card.taskId));

async function doStart() { await taskStore.startTask(props.card.taskId); await faith.refreshStatus(); }
async function doPause() { await taskStore.pauseTask(props.card.taskId); await faith.refreshStatus(); }
async function doResume() { await taskStore.resumeTask(props.card.taskId); await faith.refreshStatus(); }
async function doComplete() {
  const mins = prompt('实际用时(分钟):', String(task.value?.estimated_minutes ?? 30));
  if (mins) { await taskStore.completeTask(props.card.taskId, parseInt(mins)); await faith.refreshStatus(); }
}
async function doAbandon() { await taskStore.abandonTask(props.card.taskId); await faith.refreshStatus(); }

function onDragStart(e: DragEvent) {
  e.dataTransfer?.setData('text/plain', props.card.taskId);
  emit('dragStart', props.card.taskId);
}
</script>

<style scoped>
.kanban-card { background: var(--color-surface); border-radius: 6px; padding: 8px 10px; cursor: grab; border: 1px solid var(--color-border); }
.kanban-card:hover { border-color: var(--color-primary); }
.card-title { font-weight: 500; font-size: 0.85rem; margin-bottom: 4px; }
.card-meta { font-size: 0.7rem; color: var(--color-text-muted); }
.card-timer { font-size: 0.7rem; color: var(--color-success); margin-top: 4px; }
.card-actions { display: flex; gap: 4px; margin-top: 6px; }
.card-binding { font-size: 0.7rem; color: var(--color-primary-dim); margin-top: 4px; }
</style>
