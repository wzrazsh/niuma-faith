<template>
  <div class="board">
    <div class="board-header">
      <div class="board-title">任务看板</div>
      <div class="board-actions">
        <button @click="kanban.addColumn('新列')">
          <span class="btn-icon">+</span> 添加列
        </button>
        <button @click="kanban.resetToDefault">
          重置默认
        </button>
      </div>
    </div>
    <div class="board-columns">
      <KanbanColumn v-for="col in kanban.columns" :key="col.id" :column="col" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useKanbanStore } from '@/stores/kanban';
import KanbanColumn from './KanbanColumn.vue';

const kanban = useKanbanStore();

onMounted(() => {
  kanban.loadBoard();
});
</script>

<style scoped>
.board {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}

.board-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.board-title {
  font-family: var(--font-display);
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--color-text);
}

.board-actions {
  display: flex;
  gap: 8px;
}

.board-actions button {
  font-size: 0.78rem;
  background: var(--color-surface);
  border: 1px solid var(--color-border-subtle);
  color: var(--color-text-muted);
  padding: 6px 12px;
}

.board-actions button:hover {
  background: var(--color-surface-hover);
  color: var(--color-text);
  border-color: var(--color-border);
}

.btn-icon {
  font-size: 0.85rem;
}

.board-columns {
  display: flex;
  gap: 14px;
  flex: 1;
  overflow-x: auto;
  padding-bottom: 8px;
}
</style>
