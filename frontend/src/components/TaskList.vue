<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useTaskStore } from "@/stores/task";
import type { Task } from "@/types";
import TaskForm from "./TaskForm.vue";

defineProps<{
  readonly?: boolean;
}>();

const store = useTaskStore();
const showForm = ref(false);
const editingTask = ref<Task | null>(null);

onMounted(() => {
  store.fetchTasksByDate(store.selectedDate);
});

function openCreateForm() {
  editingTask.value = null;
  showForm.value = true;
}

function openEditForm(task: Task) {
  editingTask.value = task;
  showForm.value = true;
}

function closeForm() {
  showForm.value = false;
  editingTask.value = null;
}

async function handleComplete(task: Task) {
  const actual = task.actual_minutes > 0 ? task.actual_minutes : task.estimated_minutes;
  await store.completeTask(task.id, actual);
}

async function handleAbandon(task: Task) {
  await store.abandonTask(task.id);
}

async function handleDelete(task: Task) {
  if (confirm(`确定删除任务「${task.title}」？`)) {
    await store.deleteTask(task.id);
  }
}

function formatMinutes(min: number): string {
  if (min < 60) return `${min}min`;
  const h = Math.floor(min / 60);
  const m = min % 60;
  return m > 0 ? `${h}h${m}m` : `${h}h`;
}

function categoryLabel(cat: string): string {
  if (cat === "work") return "工作";
  if (cat === "study") return "学习";
  return "其他";
}
</script>

<template>
  <section class="task-list">
    <div class="list-header">
      <div class="tabs">
        <button
          v-for="f in (['all', 'active', 'completed', 'abandoned'] as const)"
          :key="f"
          class="tab-btn"
          :class="{ active: store.filter === f }"
          @click="store.setFilter(f)"
        >
          {{ f === 'all' ? '全部' : f === 'active' ? '进行中' : f === 'completed' ? '已完成' : '已放弃' }}
        </button>
      </div>
      <button v-if="!readonly" class="add-btn" @click="openCreateForm">+ 添加任务</button>
      <span v-else class="readonly-indicator">只读</span>
    </div>

    <div v-if="store.isLoading" class="loading">加载中...</div>
    <div v-else-if="store.filteredTasks.length === 0" class="empty">
      <p>暂无{{ store.filter === 'all' ? '' : store.filter === 'active' ? '进行中' : store.filter === 'completed' ? '已完成' : '已放弃' }}任务</p>
    </div>
    <ul v-else class="task-items">
      <li v-for="task in store.filteredTasks" :key="task.id" class="task-item">
        <div class="task-info">
          <div class="task-top">
            <span class="task-title">{{ task.title }}</span>
            <span class="task-category" :data-cat="task.category">{{ categoryLabel(task.category) }}</span>
          </div>
          <div class="task-meta">
            <span>预计 {{ formatMinutes(task.estimated_minutes) }}</span>
            <span v-if="task.status === 'completed'">，实际 {{ formatMinutes(task.actual_minutes) }}</span>
          </div>
          <p v-if="task.description" class="task-desc">{{ task.description }}</p>
        </div>
        <div v-if="!readonly" class="task-actions">
          <template v-if="task.status === 'active'">
            <button class="action-btn complete" @click="handleComplete(task)">完成</button>
            <button class="action-btn edit" @click="openEditForm(task)">编辑</button>
            <button class="action-btn abandon" @click="handleAbandon(task)">放弃</button>
          </template>
          <button class="action-btn delete" @click="handleDelete(task)">删除</button>
        </div>
      </li>
    </ul>

    <TaskForm v-if="showForm && !readonly" :task="editingTask" @close="closeForm" />
  </section>
</template>

<style scoped>
.task-list {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 16px;
  padding: 20px;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.tabs {
  display: flex;
  gap: 4px;
}

.tab-btn {
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  background: transparent;
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.15s;
}

.tab-btn.active {
  background: var(--color-primary);
  color: #1a1a24;
  font-weight: 600;
}

.add-btn {
  padding: 8px 16px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  font-size: 0.875rem;
  color: var(--color-text);
  cursor: pointer;
}

.add-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.readonly-indicator {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  font-style: italic;
}

.loading, .empty {
  text-align: center;
  padding: 32px;
  color: var(--color-text-muted);
  font-size: 0.875rem;
}

.task-items {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.task-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  padding: 14px;
  background: var(--color-bg);
  border-radius: 12px;
}

.task-info {
  flex: 1;
  min-width: 0;
}

.task-top {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.task-title {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--color-text);
}

.task-category {
  font-size: 0.75rem;
  padding: 2px 8px;
  border-radius: 6px;
  font-weight: 500;
}

.task-category[data-cat="work"] { background: var(--color-survival); color: #1a1a24; }
.task-category[data-cat="study"] { background: var(--color-progress); color: #1a1a24; }
.task-category[data-cat="other"] { background: var(--color-discipline); color: #1a1a24; }

.task-meta {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.task-desc {
  margin: 6px 0 0;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  white-space: pre-wrap;
}

.task-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.action-btn {
  padding: 5px 10px;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 500;
  border: none;
  cursor: pointer;
  transition: opacity 0.15s;
}

.action-btn:hover { opacity: 0.8; }

.action-btn.complete { background: var(--color-discipline); color: #1a1a24; }
.action-btn.edit { background: var(--color-progress); color: #1a1a24; }
.action-btn.abandon { background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-text-muted); }
.action-btn.delete { background: transparent; color: #e06040; }
</style>
