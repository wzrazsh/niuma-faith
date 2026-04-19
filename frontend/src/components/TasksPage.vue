<script setup lang="ts">
import DailyGoalPanel from "./DailyGoalPanel.vue";
import FaithDashboard from "./FaithDashboard.vue";
import TaskList from "./TaskList.vue";
</script>

<template>
  <div class="tasks-page">
    <div class="top-panels">
      <DailyGoalPanel />
      <FaithDashboard />
    </div>
    <TaskList />
  </div>
</template>

<style scoped>
.tasks-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.top-panels {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

@media (max-width: 640px) {
  .top-panels {
    grid-template-columns: 1fr;
  }
}
</style>
