<template>
  <div class="kanban-page">
    <KanbanBoard />
  </div>
</template>

<script setup lang="ts">
import KanbanBoard from './kanban/KanbanBoard.vue';
</script>

<style scoped>
.kanban-page {
  padding: 20px;
  height: calc(100vh - 44px);
  overflow: hidden;
  position: relative;
  z-index: 1;
}
</style>
