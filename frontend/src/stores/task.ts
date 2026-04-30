import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Task, DailyStats, TaskStatus, TaskCategory } from "@/types";
import { useFaithStore } from "./faith";
import {
  invoke_get_tasks_by_date,
  invoke_create_task,
  invoke_complete_task,
  invoke_start_task,
  invoke_pause_task,
  invoke_resume_task,
  invoke_end_task,
  invoke_update_task,
  invoke_abandon_task,
  invoke_delete_task,
  invoke_get_daily_stats,
} from "@/api/task";

function todayString(): string {
  return new Date().toLocaleDateString('en-CA'); // YYYY-MM-DD local, matches backend Local time
}

export const useTaskStore = defineStore("task", () => {
  const tasks = ref<Task[]>([]);
  const dailyStats = ref<DailyStats | null>(null);
  const filter = ref<TaskStatus | "all">("all");
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  const selectedDate = ref(todayString());
  const calendarView = ref<"month" | "week" | "day">("month");

  const runningTasks = computed(() => tasks.value.filter(t => t.status === "running"));
  const pausedTasks = computed(() => tasks.value.filter(t => t.status === "paused"));
  const activeTasks = computed(() => tasks.value.filter(t => t.status === "running" || t.status === "paused"));
  const completedTasks = computed(() => tasks.value.filter(t => t.status === "completed"));
  const abandonedTasks = computed(() => tasks.value.filter(t => t.status === "abandoned"));

  const filteredTasks = computed(() => {
    if (filter.value === "all") return tasks.value;
    return tasks.value.filter(t => t.status === filter.value);
  });

  // US-009: renamed to fetchTasksByDate
  async function fetchTasksByDate(date: string, status?: TaskStatus) {
    try {
      isLoading.value = true;
      error.value = null;
      tasks.value = await invoke_get_tasks_by_date(date, status);
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  // US-009: createTask accepts optional date param
  async function createTask(
    title: string,
    description: string,
    category: TaskCategory,
    estimated_minutes: number,
    date?: string
  ) {
    try {
      isLoading.value = true;
      error.value = null;
      const task = await invoke_create_task(title, description, category, estimated_minutes, date ?? selectedDate.value);
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
      const idx = tasks.value.findIndex(t => t.id === id);
      if (idx !== -1) tasks.value[idx] = result.task;
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

  async function startTask(id: string) {
    try {
      isLoading.value = true;
      error.value = null;
      const updated = await invoke_start_task(id);
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

  async function pauseTask(id: string) {
    try {
      isLoading.value = true;
      error.value = null;
      const updated = await invoke_pause_task(id);
      const idx = tasks.value.findIndex(t => t.id === id);
      if (idx !== -1) tasks.value[idx] = updated;
      const faithStore = useFaithStore();
      await faithStore.fetchTodayRecord();
      return updated;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function resumeTask(id: string) {
    try {
      isLoading.value = true;
      error.value = null;
      const updated = await invoke_resume_task(id);
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

  async function endTask(id: string) {
    try {
      isLoading.value = true;
      error.value = null;
      const updated = await invoke_end_task(id);
      const idx = tasks.value.findIndex(t => t.id === id);
      if (idx !== -1) tasks.value[idx] = updated;
      const faithStore = useFaithStore();
      await faithStore.fetchTodayRecord();
      return updated;
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
    actual_minutes?: number,
    notes?: string,
    status?: TaskStatus
  ) {
    try {
      isLoading.value = true;
      error.value = null;
      const updated = await invoke_update_task(id, title, description, estimated_minutes, actual_minutes, notes, status);
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

  // US-009: new actions
  function setSelectedDate(date: string) {
    selectedDate.value = date;
  }

  function setCalendarView(view: "month" | "week" | "day") {
    calendarView.value = view;
  }

  return {
    tasks,
    dailyStats,
    filter,
    isLoading,
    error,
    runningTasks,
    pausedTasks,
    completedTasks,
    abandonedTasks,
    filteredTasks,
    selectedDate,
    calendarView,
    fetchTasksByDate,
    createTask,
    completeTask,
    startTask,
    pauseTask,
    resumeTask,
    endTask,
    updateTask,
    abandonTask,
    deleteTask,
    fetchDailyStats,
    setFilter,
    setSelectedDate,
    setCalendarView,
  };
});
