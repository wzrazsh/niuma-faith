<template>
  <div class="task-list">
    <div class="task-header">
      <div class="filter-tabs">
        <button v-for="tab in tabs" :key="tab.key" :class="{ active: task.filter === tab.key }" @click="task.filter = tab.key">{{ tab.label }}</button>
      </div>
      <button class="primary" @click="showForm = true">+ 新建任务</button>
    </div>
    <div class="tasks">
      <div v-for="t in task.filteredTasks" :key="t.id" class="task-row">
        <div class="task-info">
          <span class="task-title">{{ t.title }}</span>
          <span class="task-meta">{{ t.category }} | 预计{{ t.estimated_minutes }}分钟 | 已用{{ t.duration_seconds }}秒</span>
        </div>
        <div class="task-actions">
          <button v-if="t.status === 'paused'" @click="task.startTask(t.id)">开始</button>
          <button v-if="t.status === 'running'" class="primary" @click="task.pauseTask(t.id)">暂停</button>
          <button v-if="t.status === 'running'" class="success" @click="onComplete(t)">完成</button>
          <button v-if="t.status === 'running'" class="danger" @click="task.abandonTask(t.id)">放弃</button>
          <button v-if="t.status === 'paused'" class="success" @click="task.resumeTask(t.id)">继续</button>
        </div>
      </div>
      <div v-if="task.filteredTasks.length === 0" class="empty">暂无任务</div>
    </div>
    <TaskForm v-if="showForm" @close="showForm = false" @created="showForm = false" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useTaskStore } from '@/stores/task';
import TaskForm from './TaskForm.vue';

const task = useTaskStore();
const showForm = ref(false);
const tabs = [
  { key: 'all', label: '全部' },
  { key: 'running', label: '进行中' },
  { key: 'paused', label: '暂停' },
  { key: 'completed', label: '已完成' },
  { key: 'abandoned', label: '已放弃' },
];

function onComplete(t: any) {
  const mins = prompt('实际用时(分钟):', String(t.estimated_minutes));
  if (mins) task.completeTask(t.id, parseInt(mins));
}
</script>

<style scoped>
.task-list { display: flex; flex-direction: column; gap: 8px; }
.task-header { display: flex; justify-content: space-between; align-items: center; }
.filter-tabs { display: flex; gap: 4px; }
.filter-tabs button.active { background: var(--color-primary); color: #1a1a24; }
.tasks { display: flex; flex-direction: column; gap: 4px; }
.task-row { display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: var(--color-surface); border-radius: 6px; }
.task-info { display: flex; flex-direction: column; }
.task-title { font-weight: 500; }
.task-meta { font-size: 0.75rem; color: var(--color-text-muted); }
.task-actions { display: flex; gap: 4px; }
.empty { text-align: center; color: var(--color-text-muted); padding: 24px; }
</style>
