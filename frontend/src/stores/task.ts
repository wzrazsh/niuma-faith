import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Task, DailyStats, TaskStatus, TaskCategory } from "@/types";
import { useFaithStore } from "./faith";
import {
  invoke_get_tasks,
  invoke_create_task,
  invoke_complete_task,
  invoke_update_task,
  invoke_abandon_task,
  invoke_delete_task,
  invoke_get_daily_stats,
} from "@/api/task";

export const useTaskStore = defineStore("task", () => {
  const tasks = ref<Task[]>([]);
  const dailyStats = ref<DailyStats | null>(null);
  const filter = ref<TaskStatus | "all">("all");
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  const activeTasks = computed(() => tasks.value.filter(t => t.status === "active"));
  const completedTasks = computed(() => tasks.value.filter(t => t.status === "completed"));
  const abandonedTasks = computed(() => tasks.value.filter(t => t.status === "abandoned"));

  const filteredTasks = computed(() => {
    if (filter.value === "all") return tasks.value;
    return tasks.value.filter(t => t.status === filter.value);
  });

  async function fetchTasks(status?: TaskStatus) {
    try {
      isLoading.value = true;
      error.value = null;
      tasks.value = await invoke_get_tasks(status);
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function createTask(
    title: string,
    description: string,
    category: TaskCategory,
    estimated_minutes: number
  ) {
    try {
      isLoading.value = true;
      error.value = null;
      const task = await invoke_create_task(title, description, category, estimated_minutes);
      tasks.value.push(task);
      return task;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function completeTask(id: string, actual_minutes: number) {
    try {
      isLoading.value = true;
      error.value = null;
      const result = await invoke_complete_task(id, actual_minutes);
      // Update task in list
      const idx = tasks.value.findIndex(t => t.id === id);
      if (idx !== -1) tasks.value[idx] = result.task;
      // Sync faith store (only today's record needed)
      const faithStore = useFaithStore();
      await faithStore.fetchTodayRecord();
      return result;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function updateTask(
    id: string,
    title?: string,
    description?: string,
    estimated_minutes?: number,
    notes?: string
  ) {
    try {
      isLoading.value = true;
      error.value = null;
      const updated = await invoke_update_task(id, title, description, estimated_minutes, notes);
      const idx = tasks.value.findIndex(t => t.id === id);
      if (idx !== -1) tasks.value[idx] = updated;
      return updated;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function abandonTask(id: string) {
    try {
      isLoading.value = true;
      error.value = null;
      const updated = await invoke_abandon_task(id);
      const idx = tasks.value.findIndex(t => t.id === id);
      if (idx !== -1) tasks.value[idx] = updated;
      return updated;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function deleteTask(id: string) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke_delete_task(id);
      tasks.value = tasks.value.filter(t => t.id !== id);
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function fetchDailyStats(date: string) {
    try {
      dailyStats.value = await invoke_get_daily_stats(date);
    } catch (e) {
      error.value = String(e);
    }
  }

  function setFilter(f: TaskStatus | "all") {
    filter.value = f;
  }

  return {
    tasks,
    dailyStats,
    filter,
    isLoading,
    error,
    activeTasks,
    completedTasks,
    abandonedTasks,
    filteredTasks,
    fetchTasks,
    createTask,
    completeTask,
    updateTask,
    abandonTask,
    deleteTask,
    fetchDailyStats,
    setFilter,
  };
});
