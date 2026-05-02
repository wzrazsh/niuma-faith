<!-- frontend/src/components/kanban/KanbanColumn.vue -->
<script setup lang="ts">
import { ref } from 'vue';
import type { KanbanColumn, ProcessBinding } from '@/types/kanban';
import type { Task } from '@/types';
import KanbanCard from './KanbanCard.vue';

const props = defineProps<{
  column: KanbanColumn;
  tasks: Task[];
  readonly?: boolean;
  processBindings?: Map<string, ProcessBinding | undefined>;
  processRunning?: Map<string, boolean>;
}>();

const emit = defineEmits<{
  (e: 'card-drop', taskId: string, toColumnId: string, newOrder: number): void;
  (e: 'card-start' | 'card-pause' | 'card-complete' | 'card-edit' | 'card-delete' | 'unbind-process', task: Task): void;
  (e: 'add-task' | 'delete-column', id: string): void;
  (e: 'bind-process', task: Task, binding: ProcessBinding): void;
}>();

const isDragOver = ref(false);

function handleDragOver(e: DragEvent) {
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'move';
  }
  isDragOver.value = true;
}

function handleDragLeave() {
  isDragOver.value = false;
}

function handleDrop(e: DragEvent) {
  e.preventDefault();
  isDragOver.value = false;
  
  if (e.dataTransfer) {
    const taskId = e.dataTransfer.getData('taskId');
    if (taskId) {
      const newOrder = props.column.taskIds.length;
      emit('card-drop', taskId, props.column.id, newOrder);
    }
  }
}
</script>

    <template>
      <div
        class="kanban-column"
        :class="{ 'drag-over': isDragOver }"
        @dragover="handleDragOver"
        @dragleave="handleDragLeave"
        @drop="handleDrop"
      >
        <div class="column-header">
          <div class="column-title-row">
            <h3 class="column-title">{{ column.title }}</h3>
            <span class="column-count">{{ tasks.length }}</span>
          </div>
          <div class="column-actions">
            <button
              v-if="!readonly"
              class="add-task-btn"
              @click="emit('add-task', column.id)"
              title="创建任务"
            >
              +
            </button>
            <button
              v-if="column.isCustom && !readonly"
              class="delete-column-btn"
              @click="emit('delete-column', column.id)"
              title="删除列"
            >
              ×
            </button>
          </div>
        </div>
        
        <div class="column-cards">
          <KanbanCard
            v-for="task in tasks"
            :key="task.id"
            :task="task"
            :column-id="column.id"
            :readonly="readonly"
            :process-binding="processBindings?.get(task.id)"
            :is-process-running="processRunning?.get(task.id) ?? false"
            @start="(t) => emit('card-start', t)"
            @pause="(t) => emit('card-pause', t)"
            @complete="(t) => emit('card-complete', t)"
            @edit="(t) => emit('card-edit', t)"
            @delete="(t) => emit('card-delete', t)"
            @bind-process="(t, b) => emit('bind-process', t, b)"
            @unbind-process="(t) => emit('unbind-process', t)"
          />
          
          <div v-if="tasks.length === 0" class="column-empty">
            暂无任务
          </div>
        </div>
      </div>
    </template>

<style scoped>
.kanban-column {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 16px;
  min-width: 280px;
  max-width: 280px;
  display: flex;
  flex-direction: column;
  transition: all 0.15s;
}

.kanban-column.drag-over {
  border-color: var(--color-primary);
  background: rgba(59, 130, 246, 0.05);
}

.column-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--color-border);
}

.column-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.column-title {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.column-count {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  background: var(--color-bg);
  padding: 2px 8px;
  border-radius: 10px;
}

.delete-column-btn {
  background: transparent;
  border: none;
  color: var(--color-text-muted);
  font-size: 1.25rem;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
  transition: color 0.15s;
}

.delete-column-btn:hover {
  color: #e06040;
}

.column-cards {
  flex: 1;
  overflow-y: auto;
  min-height: 100px;
}

.column-empty {
  text-align: center;
  padding: 24px;
  color: var(--color-text-muted);
  font-size: 0.875rem;
}
</style>
