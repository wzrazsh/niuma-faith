<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal" @click.stop>
      <div class="modal-header">
        <span class="modal-title">编辑任务</span>
        <button class="modal-close" @click="$emit('close')">&#10005;</button>
      </div>
      <div class="modal-body" v-if="task">
        <div class="field-group">
          <label class="field-label">任务名称</label>
          <input v-model="form.title" placeholder="输入任务名称..." />
        </div>
        <div class="field-group">
          <label class="field-label">描述</label>
          <textarea v-model="form.description" placeholder="任务描述（可选）" rows="2"></textarea>
        </div>
        <div class="field-row">
          <div class="field-group flex-1">
            <label class="field-label">分类</label>
            <select v-model="form.category">
              <option value="work">工作</option>
              <option value="study">学习</option>
              <option value="other">其他</option>
            </select>
          </div>
          <div class="field-group flex-1">
            <label class="field-label">预计时长</label>
            <input v-model.number="form.estimated" type="number" placeholder="分钟" min="1" />
          </div>
        </div>
        <div class="field-row">
          <div class="field-group flex-1">
            <label class="field-label">状态</label>
            <select v-model="form.status">
              <option value="paused">待办</option>
              <option value="running">进行中</option>
              <option value="completed">已完成</option>
              <option value="abandoned">已放弃</option>
            </select>
          </div>
          <div class="field-group flex-1">
            <label class="field-label">实际用时</label>
            <input v-model.number="form.actual" type="number" placeholder="分钟" min="0" />
          </div>
        </div>
        <div v-if="errorMsg" class="error-msg">{{ errorMsg }}</div>
      </div>
      <div class="modal-footer">
        <button class="modal-danger" @click="handleDelete">删除</button>
        <button class="modal-cancel" @click="$emit('close')">取消</button>
        <button class="primary" @click="handleSave" :disabled="saving">{{ saving ? '保存中...' : '保存' }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue';
import type { Task } from '@/types';
import { useTaskStore } from '@/stores/task';
import { useFaithStore } from '@/stores/faith';

const props = defineProps<{ taskId: string }>();
const emit = defineEmits<{ close: []; saved: [] }>();

const taskStore = useTaskStore();
const faith = useFaithStore();
const saving = ref(false);
const errorMsg = ref('');

const task = ref<Task | null>(null);
const form = reactive({ title: '', description: '', category: 'work', estimated: 30, actual: 0, status: 'paused' });

watch(() => props.taskId, initForm, { immediate: true });

function initForm() {
  errorMsg.value = '';
  const t = taskStore.tasks.find(t => t.id === props.taskId);
  if (t) {
    task.value = t;
    form.title = t.title;
    form.description = t.description;
    form.category = t.category;
    form.estimated = t.estimated_minutes;
    form.actual = t.actual_minutes;
    form.status = t.status;
  }
}

async function handleSave() {
  if (!form.title.trim()) return;
  if (form.estimated <= 0) return;
  saving.value = true;
  errorMsg.value = '';
  try {
    await taskStore.updateTask(props.taskId, {
      title: form.title,
      description: form.description,
      category: form.category,
      estimated_minutes: form.estimated,
      actual_minutes: form.actual,
      status: form.status,
    });
    await faith.refreshStatus();
    emit('saved');
    emit('close');
  } catch (e: any) {
    errorMsg.value = e?.message || e?.toString() || '保存失败';
  } finally {
    saving.value = false;
  }
}

async function handleDelete() {
  if (!confirm('确定删除该任务？')) return;
  try {
    await taskStore.deleteTask(props.taskId);
    await faith.refreshStatus();
    emit('saved');
    emit('close');
  } catch (e: any) {
    errorMsg.value = e?.message || e?.toString() || '删除失败';
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(8, 8, 16, 0.7);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
  animation: fade-in 0.2s ease;
}

.modal {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  width: 440px;
  max-width: 90vw;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  animation: modal-enter 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border-subtle);
}

.modal-title {
  font-family: var(--font-display);
  font-size: 1rem;
  font-weight: 700;
}

.modal-close {
  width: 28px;
  height: 28px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: var(--color-text-muted);
  border-radius: 50%;
  font-size: 0.85rem;
}

.modal-close:hover {
  background: var(--color-surface-hover);
  color: var(--color-text);
  transform: none;
}

.modal-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.field-group {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.field-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-muted);
  letter-spacing: 0.03em;
}

.flex-1 { flex: 1; }

.field-row {
  display: flex;
  gap: 12px;
}

.modal-footer {
  padding: 14px 20px;
  border-top: 1px solid var(--color-border-subtle);
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.modal-cancel {
  background: transparent;
  color: var(--color-text-muted);
}

.modal-cancel:hover {
  background: var(--color-surface-hover);
  color: var(--color-text);
  transform: none;
}

.modal-danger {
  background: transparent;
  color: var(--color-danger);
  margin-right: auto;
}

.modal-danger:hover {
  background: var(--color-danger-dim);
  color: var(--color-danger);
  transform: none;
}

.error-msg {
  font-size: 0.78rem;
  color: var(--color-danger);
  padding: 6px 10px;
  background: var(--color-danger-dim);
  border-radius: var(--radius-sm);
}

@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes modal-enter {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(10px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}
</style>
