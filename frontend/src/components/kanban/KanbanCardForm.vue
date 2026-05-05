<template>
  <div class="overlay" @click.self="$emit('close')">
    <div class="form-panel" @click.stop>
      <div class="form-title">添加卡片</div>
      <div class="form-field">
        <label>关联任务</label>
        <select v-model="taskId">
          <option v-for="t in availableTasks" :key="t.id" :value="t.id">{{ t.title }}</option>
        </select>
      </div>
      <div class="form-actions">
        <button @click="$emit('close')">取消</button>
        <button class="primary" @click="submit">添加</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useKanbanStore } from '@/stores/kanban';
import { useTaskStore } from '@/stores/task';

const props = defineProps<{ columnId: string }>();
const emit = defineEmits<{ close: []; created: [] }>();
const kanban = useKanbanStore();
const taskStore = useTaskStore();
const taskId = ref('');

const availableTasks = computed(() => {
  return taskStore.tasks.filter(t => !kanban.cards.find(c => c.taskId === t.id));
});

function submit() {
  if (!taskId.value) return;
  kanban.addCard(props.columnId, taskId.value);
  emit('created');
  emit('close');
}
</script>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(8, 8, 16, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.form-panel {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: 20px;
  width: 360px;
  max-width: 90vw;
  display: flex;
  flex-direction: column;
  gap: 14px;
  animation: fade-slide-up 0.2s ease;
}

.form-title {
  font-family: var(--font-display);
  font-size: 0.95rem;
  font-weight: 700;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.form-field label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-muted);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}
</style>
