<template>
  <div class="column" :class="{ dragging: dragOver }"
    @dragover.prevent="dragOver = true"
    @dragleave="dragOver = false"
    @drop="onDrop">
    <div class="column-header">
      <div class="column-info">
        <span class="column-title">{{ column.title }}</span>
        <span class="column-count">{{ totalCards }}</span>
      </div>
      <button class="column-add" :title="'添加到 ' + column.title" @click="showForm = true">+</button>
    </div>
    <div class="column-cards">
      <div v-for="group in kanban.columnSwimlanes(column.id)" :key="column.id + '-' + group.categoryId" class="swimlane">
        <div class="swimlane-header">
          <span class="swimlane-dot" :class="group.categoryId"></span>
          <span class="swimlane-label">{{ group.label }}</span>
          <span class="swimlane-count">{{ group.cards.length }}</span>
        </div>
        <KanbanCard v-for="card in group.cards" :key="card.taskId" :card="card" :column-id="column.id" @edit="id => $emit('edit', id)" />
      </div>
    </div>
    <KanbanCardForm v-if="showForm" :column-id="column.id" @close="showForm = false" @created="showForm = false" :key="'form-' + Date.now()" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useKanbanStore } from '@/stores/kanban';
import KanbanCard from './KanbanCard.vue';
import KanbanCardForm from './KanbanCardForm.vue';

const props = defineProps<{ column: any }>();
const kanban = useKanbanStore();
const dragOver = ref(false);
const showForm = ref(false);

const totalCards = computed(() => {
  return kanban.columnCards(props.column.id).length;
});

function onDrop(e: DragEvent) {
  dragOver.value = false;
  const cardId = e.dataTransfer?.getData('text/plain');
  if (cardId) {
    const targetIndex = getDropIndex(e);
    kanban.moveCard(cardId, props.column.id, targetIndex);
  }
}

function getDropIndex(e: DragEvent): number {
  const cardEls = (e.currentTarget as HTMLElement).querySelectorAll('.card');
  const mouseY = e.clientY;
  for (let i = 0; i < cardEls.length; i++) {
    const rect = cardEls[i].getBoundingClientRect();
    if (mouseY < rect.top + rect.height / 2) return i;
  }
  return cardEls.length;
}
</script>

<style scoped>
.column {
  min-width: 260px;
  max-width: 300px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex-shrink: 0;
  background: var(--color-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.column.dragging {
  border-color: var(--color-primary);
  background: var(--color-primary-glow);
}

.column-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 14px 0;
}

.column-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.column-title {
  font-family: var(--font-display);
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-text);
}

.column-count {
  background: var(--color-bg);
  color: var(--color-text-muted);
  font-size: 0.72rem;
  padding: 1px 7px;
  border-radius: 10px;
  font-family: var(--font-mono);
}

.column-add {
  width: 26px;
  height: 26px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: var(--color-text-muted);
  border-radius: 50%;
  font-size: 1.1rem;
}

.column-add:hover {
  background: var(--color-surface-hover);
  color: var(--color-primary);
  transform: none;
}

.column-cards {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0 10px 10px;
  flex: 1;
}

.swimlane {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.swimlane-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 2px 2px;
}

.swimlane-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.swimlane-dot.work { background: var(--color-work); }
.swimlane-dot.study { background: var(--color-study); }
.swimlane-dot.other { background: var(--color-other); }

.swimlane-label {
  font-size: 0.68rem;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
}

.swimlane-count {
  font-size: 0.62rem;
  color: var(--color-text-muted);
  opacity: 0.6;
  font-family: var(--font-mono);
}
</style>
