<!-- frontend/src/components/KanbanPage.vue -->
<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useTaskStore } from '@/stores/task';
import KanbanBoard from './kanban/KanbanBoard.vue';

const taskStore = useTaskStore();

const isLoading = ref(false);

onMounted(async () => {
  await loadTasks();
});

async function loadTasks() {
  try {
    isLoading.value = true;
    await taskStore.fetchTasksByDate(taskStore.selectedDate);
  } catch (error) {
    console.error('Failed to load tasks:', error);
  } finally {
    isLoading.value = false;
  }
}

function handleRefresh() {
  loadTasks();
}
</script>

<template>
  <div class="kanban-page">
    <KanbanBoard
      :tasks="taskStore.tasks"
      @refresh="handleRefresh"
    />
  </div>
</template>

<style scoped>
.kanban-page {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
}
</style>
