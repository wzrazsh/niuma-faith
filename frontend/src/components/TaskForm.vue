<script setup lang="ts">
import { ref, watch } from "vue";
import { useTaskStore } from "@/stores/task";
import type { Task, TaskCategory } from "@/types";

const props = defineProps<{
  task: Task | null;
}>();

const emit = defineEmits<{
  close: [];
}>();

const store = useTaskStore();

const isEdit = ref(props.task !== null);
const title = ref(props.task?.title ?? "");
const description = ref(props.task?.description ?? "");
const category = ref<TaskCategory>(props.task?.category ?? "work");
const estimatedHours = ref(props.task ? props.task.estimated_minutes / 60 : 1);
const notes = ref(props.task?.notes ?? "");
const isSubmitting = ref(false);
const errorMsg = ref("");

watch(() => props.task, (t) => {
  isEdit.value = t !== null;
  title.value = t?.title ?? "";
  description.value = t?.description ?? "";
  category.value = t?.category ?? "work";
  estimatedHours.value = t ? t.estimated_minutes / 60 : 1;
  notes.value = t?.notes ?? "";
});

async function handleSubmit() {
  if (!title.value.trim()) {
    errorMsg.value = "请输入任务名称";
    return;
  }
  if (estimatedHours.value <= 0) {
    errorMsg.value = "预计时长必须大于 0";
    return;
  }
  errorMsg.value = "";
  isSubmitting.value = true;
  try {
    const minutes = Math.round(estimatedHours.value * 60);
    if (isEdit.value && props.task) {
      await store.updateTask(props.task.id, title.value.trim(), description.value.trim(), minutes, notes.value.trim() || undefined);
    } else {
      await store.createTask(title.value.trim(), description.value.trim(), category.value, minutes);
    }
    emit("close");
  } catch (e: any) {
    errorMsg.value = String(e);
  } finally {
    isSubmitting.value = false;
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal-card">
      <div class="modal-header">
        <h3>{{ isEdit ? "编辑任务" : "新建任务" }}</h3>
        <button class="close-btn" @click="emit('close')">×</button>
      </div>

      <div class="form-group">
        <label class="form-label">任务名称 *</label>
        <input v-model="title" type="text" class="form-input" placeholder="例如：完成 React 文档" maxlength="100" />
      </div>

      <div class="form-group">
        <label class="form-label">描述</label>
        <textarea v-model="description" class="form-input" placeholder="任务详细描述（可选）" rows="3" maxlength="500"></textarea>
      </div>

      <div class="form-row">
        <div class="form-group">
          <label class="form-label">类别</label>
          <select v-model="category" class="form-input">
            <option value="work">工作</option>
            <option value="study">学习</option>
            <option value="other">其他</option>
          </select>
        </div>

        <div class="form-group">
          <label class="form-label">预计时长（小时）</label>
          <input v-model.number="estimatedHours" type="number" class="form-input" min="0.5" step="0.5" />
        </div>
      </div>

      <div v-if="isEdit" class="form-group">
        <label class="form-label">备注</label>
        <textarea v-model="notes" class="form-input" placeholder="备注（可选）" rows="2"></textarea>
      </div>

      <p v-if="errorMsg" class="error-msg">{{ errorMsg }}</p>

      <div class="modal-actions">
        <button class="cancel-btn" @click="emit('close')">取消</button>
        <button class="submit-btn" :disabled="isSubmitting" @click="handleSubmit">
          {{ isSubmitting ? "保存中..." : isEdit ? "保存" : "创建" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-card {
  background: var(--color-surface);
  border-radius: 16px;
  padding: 24px;
  width: min(480px, 90vw);
  max-height: 85vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.modal-header h3 {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
}

.close-btn {
  background: none;
  border: none;
  font-size: 1.5rem;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0 4px;
}

.form-group {
  margin-bottom: 16px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.form-label {
  display: block;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  margin-bottom: 6px;
}

.form-input {
  width: 100%;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 10px 14px;
  font-size: 0.9375rem;
  color: var(--color-text);
  box-sizing: border-box;
}

.form-input:focus {
  border-color: var(--color-primary);
}

textarea.form-input {
  resize: vertical;
}

.error-msg {
  color: #e06040;
  font-size: 0.8125rem;
  margin-bottom: 12px;
}

.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 20px;
}

.cancel-btn {
  padding: 10px 20px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  color: var(--color-text-muted);
  cursor: pointer;
}

.submit-btn {
  padding: 10px 24px;
  background: var(--color-primary);
  border: none;
  border-radius: 8px;
  color: #1a1a24;
  font-weight: 600;
  cursor: pointer;
}

.submit-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
