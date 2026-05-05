<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal" @click.stop>
      <div class="modal-header">
        <span class="modal-title">新建任务</span>
        <button class="modal-close" @click="$emit('close')">✕</button>
      </div>
      <div class="modal-body">
        <div class="field-group">
          <label class="field-label">任务名称</label>
          <input v-model="title" placeholder="输入任务名称..." />
        </div>
        <div class="field-group">
          <label class="field-label">描述</label>
          <textarea v-model="description" placeholder="任务描述（可选）" rows="2"></textarea>
        </div>
        <div class="field-row">
          <div class="field-group flex-1">
            <label class="field-label">分类</label>
            <select v-model="category">
              <option value="work">工作</option>
              <option value="study">学习</option>
              <option value="other">其他</option>
            </select>
          </div>
          <div class="field-group flex-1">
            <label class="field-label">预计时长</label>
            <input v-model.number="estimated" type="number" placeholder="分钟" min="1" />
          </div>
        </div>
        <label class="checkbox-field">
          <input type="checkbox" v-model="daily" />
          <span class="check-box"></span>
          <span>每日执行</span>
        </label>
      </div>
      <div class="modal-footer">
        <button class="modal-cancel" @click="$emit('close')">取消</button>
        <button class="primary" @click="submit">创建任务</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useTaskStore } from '@/stores/task';
import { useFaithStore } from '@/stores/faith';

const emit = defineEmits<{ close: []; created: [] }>();
const taskStore = useTaskStore();
const faith = useFaithStore();

const title = ref('');
const description = ref('');
const category = ref('work');
const estimated = ref(30);
const daily = ref(false);

async function submit() {
  if (!title.value.trim()) return;
  if (estimated.value <= 0) return;
  await taskStore.createTask(title.value, description.value, category.value, estimated.value, undefined, daily.value ? 'daily' : undefined);
  await faith.refreshStatus();
  emit('created');
  emit('close');
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
  z-index: 100;
  animation: fade-in 0.2s ease;
}

.modal {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  width: 420px;
  max-width: 90vw;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 215, 0, 0.05);
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

.flex-1 {
  flex: 1;
}

.field-row {
  display: flex;
  gap: 12px;
}

.checkbox-field {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.85rem;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 4px 0;
}

.checkbox-field input {
  display: none;
}

.check-box {
  width: 18px;
  height: 18px;
  border: 2px solid var(--color-border);
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition-fast);
  flex-shrink: 0;
}

.checkbox-field input:checked + .check-box {
  background: var(--color-primary);
  border-color: var(--color-primary);
}

.checkbox-field input:checked + .check-box::after {
  content: '✓';
  font-size: 0.7rem;
  color: #0c0c16;
  font-weight: 700;
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
