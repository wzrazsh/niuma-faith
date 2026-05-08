<template>
  <div class="dashboard">
    <aside class="sidebar">
      <CalendarView @select="onDateSelect" />
      <StatusPanel />
      <FaithDashboard />
      <DailyGoalPanel />
    </aside>
    <main class="main">
      <TaskList />
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import CalendarView from './CalendarView.vue';
import StatusPanel from './StatusPanel.vue';
import FaithDashboard from './FaithDashboard.vue';
import DailyGoalPanel from './DailyGoalPanel.vue';
import TaskList from './TaskList.vue';
import { useTaskStore } from '@/stores/task';

const task = useTaskStore();

onMounted(() => {
  task.loadTasksByDate(new Date().toISOString().slice(0, 10));
});

function onDateSelect(date: string) {
  task.loadTasksByDate(date);
}
</script>

<style scoped>
.dashboard {
  display: flex;
  gap: 20px;
  padding: 20px;
  height: calc(100vh - 44px);
  position: relative;
  z-index: 1;
}

.sidebar {
  width: 260px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  flex-shrink: 0;
  overflow-y: auto;
  padding-right: 4px;
}

.sidebar::-webkit-scrollbar {
  width: 3px;
}

.main {
  flex: 1;
  min-width: 0;
  overflow-y: auto;
}

.main::-webkit-scrollbar {
  width: 3px;
}
</style>
