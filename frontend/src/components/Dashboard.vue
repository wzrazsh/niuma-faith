<template>
  <div class="dashboard">
    <aside class="sidebar">
      <CalendarView @select="onDateSelect" />
      <StatusPanel />
      <FaithDashboard />
      <DailyGoalPanel />
    </aside>
    <main class="main-area">
      <TaskList />
    </main>
  </div>
</template>

<script setup lang="ts">
import CalendarView from './CalendarView.vue';
import FaithDashboard from './FaithDashboard.vue';
import StatusPanel from './StatusPanel.vue';
import DailyGoalPanel from './DailyGoalPanel.vue';
import TaskList from './TaskList.vue';
import { useTaskStore } from '@/stores/task';

function onDateSelect(date: string) {
  const task = useTaskStore();
  task.loadTasksByDate(date);
}
</script>

<style scoped>
.dashboard { display: flex; height: calc(100vh - 42px); }
.sidebar { width: 320px; border-right: 1px solid var(--color-border); overflow-y: auto; padding: 12px; display: flex; flex-direction: column; gap: 12px; }
.main-area { flex: 1; overflow-y: auto; padding: 12px; }
</style>
