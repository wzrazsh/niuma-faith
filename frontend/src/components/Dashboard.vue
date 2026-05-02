<script setup lang="ts">
import { ref, computed, watch } from "vue";
import CalendarView from "./CalendarView.vue";
import TaskList from "./TaskList.vue";
import FaithDashboard from "./FaithDashboard.vue";
import StatusPanel from "./StatusPanel.vue";
import { useTaskStore } from "@/stores/task";
import { useFaithStore } from "@/stores/faith";

const taskStore = useTaskStore();
const faithStore = useFaithStore();

const todayString = new Date().toLocaleDateString('en-CA'); // YYYY-MM-DD local, matches backend
const selectedDate = ref(todayString);
const calendarView = ref<"month" | "week" | "day">("month");

const isToday = computed(() => selectedDate.value === todayString);
const isPast = computed(() => selectedDate.value < todayString);

const formattedDate = computed(() => {
  const [y, m, d] = selectedDate.value.split("-");
  return `${y}年${m}月${d}日`;
});

async function loadData() {
  await Promise.all([
    taskStore.fetchTasksByDate(selectedDate.value),
    faithStore.fetchStatus(),
  ]);
}

// Watch for date changes
watch(selectedDate, loadData, { immediate: true });
</script>

<template>
  <div class="dashboard">
    <!-- Left Sidebar -->
    <aside class="sidebar">
      <CalendarView
        :model-value="selectedDate"
        :view="calendarView"
        @update:model-value="selectedDate = $event"
        @update:view="calendarView = $event"
      />

      <div class="view-toggle">
        <button
          v-for="v in (['month', 'week', 'day'] as const)"
          :key="v"
          class="toggle-btn"
          :class="{ active: calendarView === v }"
          @click="calendarView = v"
        >
          {{ v === "month" ? "月" : v === "week" ? "周" : "日" }}
        </button>
      </div>

      <!-- Moved: Readonly banner for past dates -->
      <div v-if="isPast" class="readonly-banner">
        <span class="readonly-icon">&#128274;</span>
        历史记录只读
      </div>

      <!-- Moved: Faith Section -->
      <div class="faith-section">
        <StatusPanel v-if="faithStore.faithStatus" />
        <FaithDashboard v-if="faithStore.faithStatus" :status="faithStore.faithStatus" />
      </div>
    </aside>

    <!-- Right Content Area -->
    <main class="content">
      <!-- Date Header -->
      <div class="date-header">
        <h2 class="date-title">{{ formattedDate }}</h2>
        <span v-if="isToday" class="today-badge">今天</span>
      </div>

      <!-- Task List -->
      <div class="task-section">
        <TaskList :readonly="isPast" />
      </div>
    </main>
  </div>
</template>

<style scoped>
.dashboard {
  display: flex;
  gap: 20px;
  height: 100%;
  padding: 20px;
  box-sizing: border-box;
}

.sidebar {
  width: 320px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.view-toggle {
  display: flex;
  gap: 4px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 4px;
}

.toggle-btn {
  flex: 1;
  padding: 6px 0;
  border-radius: 8px;
  font-size: 0.875rem;
  background: transparent;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.toggle-btn.active {
  background: var(--color-primary);
  color: #1a1a24;
  font-weight: 600;
}

.content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.date-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.readonly-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  background: rgba(200, 180, 100, 0.1);
  border: 1px solid rgba(200, 180, 100, 0.3);
  border-radius: 10px;
  color: #c8a860;
  font-size: 0.875rem;
  font-weight: 500;
}

.readonly-icon {
  font-size: 1rem;
}

.date-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.date-title {
  font-size: 1rem;
  font-weight: 700;
  color: var(--color-text);
  margin: 0;
}

.today-badge {
  padding: 2px 10px;
  background: var(--color-primary);
  color: #1a1a24;
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 600;
}

.faith-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 16px;
  padding: 16px;
}

.task-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 16px;
  padding: 16px;
  flex: 1;
}
</style>
