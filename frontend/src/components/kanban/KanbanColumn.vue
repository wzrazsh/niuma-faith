<template>
  <div
    class="kanban-column"
    @dragover.prevent
    @drop.prevent="onDrop"
    :class="{ 'drag-over': isDragOver }"
    @dragenter.prevent="isDragOver = true"
    @dragleave.prevent="isDragOver = false"
  >
    <div class="col-header">
      <span class="col-title">{{ column.title }}</span>
      <span class="col-count">{{ cards.length }}</span>
      <div class="col-actions">
        <button @click="$emit('addCard')">+</button>
        <button v-if="column.isCustom" class="danger" @click="$emit('deleteColumn')">×</button>
      </div>
    </div>
    <div class="col-cards">
      <KanbanCard
        v-for="(card, idx) in sortedCards"
        :key="card.taskId"
        :card="card"
        :index="idx"
        @drag-start="onDragStart"
      />
      <div v-if="cards.length === 0" class="empty">暂无任务</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { KanbanColumn, KanbanCard as KanbanCardType } from '@/types/kanban';
import KanbanCard from './KanbanCard.vue';

const props = defineProps<{ column: KanbanColumn; cards: KanbanCardType[] }>();
const emit = defineEmits<{ drop: [columnId: string, index: number]; addCard: []; deleteColumn: [] }>();

const isDragOver = ref(false);

const sortedCards = computed(() => [...props.cards].sort((a, b) => a.orderInColumn - b.orderInColumn));

function onDrop(e: DragEvent) {
  isDragOver.value = false;
  const cardId = e.dataTransfer?.getData('text/plain');
  if (!cardId) return;
  const idx = getDropIndex(e);
  emit('drop', props.column.id, idx);
}

function onDragStart(cardId: string) {
  // handled in parent
}

function getDropIndex(e: DragEvent): number {
  return sortedCards.value.length;
}
</script>

<style scoped>
.kanban-column { min-width: 280px; max-width: 280px; background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 8px; padding: 12px; display: flex; flex-direction: column; }
.kanban-column.drag-over { border-color: var(--color-primary); }
.col-header { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; padding-bottom: 8px; border-bottom: 1px solid var(--color-border); }
.col-title { font-weight: 600; }
.col-count { font-size: 0.75rem; color: var(--color-text-muted); background: var(--color-surface); padding: 1px 6px; border-radius: 10px; }
.col-actions { margin-left: auto; display: flex; gap: 4px; }
.col-cards { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 6px; }
.empty { text-align: center; color: var(--color-text-muted); padding: 16px; font-size: 0.85rem; }
</style>
