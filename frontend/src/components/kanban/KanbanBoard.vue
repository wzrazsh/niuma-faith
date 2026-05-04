<template>
  <div class="kanban-board">
    <div class="board-header">
      <h2>任务看板</h2>
      <button class="primary" @click="addCol">+ 添加列</button>
    </div>
    <div class="board-columns" v-if="!kanban.isLoading">
      <KanbanColumn
        v-for="col in kanban.columns"
        :key="col.id"
        :column="col"
        :cards="getColumnCards(col.id)"
        @drop="onDrop"
        @add-card="addCard(col.id)"
        @delete-column="kanban.removeColumn(col.id)"
      />
    </div>
    <div v-else class="loading">加载中...</div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useKanbanStore } from '@/stores/kanban';
import type { KanbanCard } from '@/types/kanban';
import KanbanColumn from './KanbanColumn.vue';

const kanban = useKanbanStore();

onMounted(() => { kanban.loadBoard(); });

function getColumnCards(colId: string): KanbanCard[] {
  const ret: KanbanCard[] = [];
  for (const [_, card] of kanban.cards) {
    if (card.columnId === colId) ret.push(card);
  }
  return ret;
}

let draggedCardId: string | null = null;

function onDragStart(cardId: string) { draggedCardId = cardId; }

function onDrop(targetColumnId: string, targetIndex: number) {
  if (!draggedCardId) return;
  kanban.moveCard(draggedCardId, targetColumnId, targetIndex);
  draggedCardId = null;
}

function addCard(columnId: string) { /* Open card form - simplified */ }

async function addCol() {
  const name = prompt('列名:');
  if (name) kanban.addColumn(name);
}
</script>

<style scoped>
.kanban-board { padding: 16px; height: 100%; }
.board-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.board-header h2 { font-size: 1.2rem; }
.board-columns { display: flex; gap: 12px; overflow-x: auto; height: calc(100vh - 100px); }
.loading { text-align: center; color: var(--color-text-muted); padding: 24px; }
</style>
