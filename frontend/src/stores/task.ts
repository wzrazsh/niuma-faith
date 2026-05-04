import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Task, TaskCompleteResult } from '@/types';
import * as api from '@/api/task';

export const useTaskStore = defineStore('task', () => {
  const tasks = ref<Task[]>([]);
  const dailyStats = ref<any>(null);
  const selectedDate = ref<string>(new Date().toISOString().slice(0, 10));
  const filter = ref<string>('all');

  const filteredTasks = computed(() => {
    if (filter.value === 'all') return tasks.value;
    return tasks.value.filter(t => t.status === filter.value);
  });

  async function loadTasksByDate(date: string) {
    selectedDate.value = date;
    tasks.value = await api.invoke_get_tasks_by_date(date);
  }

  async function createTask(title: string, description: string, category: string, estimatedMinutes: number, date?: string, recurrenceKind?: string) {
    const task = await api.invoke_create_task(title, description, category, estimatedMinutes, date, recurrenceKind);
    tasks.value.push(task);
    return task;
  }

  async function updateTask(id: string, fields: Record<string, any>) {
    const task = await api.invoke_update_task(id, fields.title, fields.description, fields.estimatedMinutes, fields.actualMinutes, fields.notes, fields.status);
    const idx = tasks.value.findIndex(t => t.id === id);
    if (idx !== -1) tasks.value[idx] = task;
    return task;
  }

  async function completeTask(id: string, actualMinutes: number) {
    const result = await api.invoke_complete_task(id, actualMinutes);
    const idx = tasks.value.findIndex(t => t.id === id);
    if (idx !== -1) tasks.value[idx] = result.task;
    return result;
  }

  async function abandonTask(id: string) {
    const task = await api.invoke_abandon_task(id);
    const idx = tasks.value.findIndex(t => t.id === id);
    if (idx !== -1) tasks.value[idx] = task;
    return task;
  }

  async function deleteTask(id: string) {
    await api.invoke_delete_task(id);
    tasks.value = tasks.value.filter(t => t.id !== id);
  }

  async function startTask(id: string) {
    const task = await api.invoke_start_task(id);
    const idx = tasks.value.findIndex(t => t.id === id);
    if (idx !== -1) tasks.value[idx] = task;
    return task;
  }

  async function pauseTask(id: string) {
    const task = await api.invoke_pause_task(id);
    const idx = tasks.value.findIndex(t => t.id === id);
    if (idx !== -1) tasks.value[idx] = task;
    return task;
  }

  async function resumeTask(id: string) {
    const task = await api.invoke_resume_task(id);
    const idx = tasks.value.findIndex(t => t.id === id);
    if (idx !== -1) tasks.value[idx] = task;
    return task;
  }

  async function endTask(id: string) {
    const task = await api.invoke_end_task(id);
    const idx = tasks.value.findIndex(t => t.id === id);
    if (idx !== -1) tasks.value[idx] = task;
    return task;
  }

  async function setRecurrence(id: string, kind: string) {
    const task = await api.invoke_set_task_recurrence(id, kind);
    const idx = tasks.value.findIndex(t => t.id === id);
    if (idx !== -1) tasks.value[idx] = task;
    return task;
  }

  return { tasks, dailyStats, selectedDate, filter, filteredTasks, loadTasksByDate, createTask, updateTask, completeTask, abandonTask, deleteTask, startTask, pauseTask, resumeTask, endTask, setRecurrence };
});
