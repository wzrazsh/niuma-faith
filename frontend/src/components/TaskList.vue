<template>
  <div class="task-list">
    <div class="task-header">
      <div class="filter-tabs">
        <button v-for="tab in tabs" :key="tab.key"
          :class="{ active: task.filter === tab.key }"
          @click="task.filter = tab.key">
          {{ tab.label }}
        </button>
      </div>
      <button class="primary" @click="showForm = true">+ 新建任务</button>
    </div>
    <div class="tasks">
      <div v-for="t in task.filteredTasks" :key="t.id" class="task-row" :class="'status-' + t.status">
        <div class="task-indicator" :class="'indicator-' + t.status"></div>
        <div class="task-info">
          <span class="task-title">{{ t.title }}</span>
          <span class="task-meta">
            <span class="task-category" :class="t.category">{{ t.category }}</span>
            <span class="meta-sep">·</span>
            <span>预计{{ t.estimated_minutes }}分钟</span>
            <span class="meta-sep">·</span>
            <span>已用{{ t.duration_seconds }}秒</span>
          </span>
        </div>
        <div class="task-actions" v-if="!isHistorical">
          <button v-if="t.status === 'paused'" @click="task.startTask(t.id)">开始</button>
          <button v-if="t.status === 'running'" class="primary" @click="task.pauseTask(t.id)">暂停</button>
          <button v-if="t.status === 'running'" class="success" @click="onComplete(t)">完成</button>
          <button v-if="t.status === 'running'" class="danger" @click="task.abandonTask(t.id)">放弃</button>
          <button v-if="t.status === 'paused'" class="success" @click="task.resumeTask(t.id)">继续</button>
        </div>
        <div v-else class="task-historical-badge">只读</div>
      </div>
      <div v-if="task.filteredTasks.length === 0" class="empty">
        <span class="empty-icon">◈</span>
        <span>暂无任务</span>
      </div>
    </div>
    <TaskForm v-if="showForm" @close="showForm = false" @created="showForm = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useTaskStore } from '@/stores/task';
import TaskForm from './TaskForm.vue';

const task = useTaskStore();
const showForm = ref(false);

const isHistorical = computed(() => task.selectedDate < new Date().toISOString().slice(0, 10));

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
.task-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.filter-tabs {
  display: flex;
  gap: 4px;
}

.filter-tabs button {
  font-size: 0.78rem;
  padding: 5px 12px;
  background: transparent;
  color: var(--color-text-muted);
  border: 1px solid transparent;
}

.filter-tabs button:hover {
  color: var(--color-text);
  background: var(--color-surface);
  border-color: var(--color-border-subtle);
}

.filter-tabs button.active {
  background: var(--color-primary-glow);
  color: var(--color-primary);
  border-color: rgba(255, 215, 0, 0.15);
  font-weight: 600;
}

.tasks {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}

.task-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  background: var(--color-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.task-row:hover {
  border-color: var(--color-border);
  background: var(--color-surface-hover);
}

.task-indicator {
  width: 4px;
  height: 32px;
  border-radius: 2px;
  flex-shrink: 0;
}

.indicator-paused { background: var(--color-text-dim); }
.indicator-running { background: var(--color-primary); box-shadow: 0 0 8px var(--color-primary-glow); }
.indicator-completed { background: var(--color-success); }
.indicator-abandoned { background: var(--color-danger); }

.task-historical-badge {
  font-size: 0.7rem;
  color: var(--color-text-dim);
  border: 1px solid var(--color-border-subtle);
  padding: 2px 8px;
  border-radius: 3px;
}

.task-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.task-title {
  font-weight: 600;
  font-size: 0.88rem;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-meta {
  font-size: 0.72rem;
  color: var(--color-text-muted);
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}

.task-category {
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 0.68rem;
  font-weight: 600;
  text-transform: uppercase;
}

.task-category.work { background: rgba(251, 114, 133, 0.15); color: var(--color-work); }
.task-category.study { background: rgba(96, 165, 250, 0.15); color: var(--color-study); }
.task-category.other { background: rgba(167, 139, 250, 0.15); color: var(--color-other); }

.meta-sep {
  color: var(--color-text-dim);
  font-size: 0.6rem;
}

.task-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--color-text-muted);
  padding: 32px;
  font-size: 0.85rem;
}

.empty-icon {
  font-size: 1.5rem;
  opacity: 0.3;
}
</style>
