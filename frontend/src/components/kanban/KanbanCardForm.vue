<!-- frontend/src/components/kanban/KanbanCardForm.vue -->
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { Task, TaskCategory } from '@/types';
import { useTaskStore } from '@/stores/task';
import { useKanbanStore } from '@/stores/kanban';

const props = defineProps<{
  task?: Task; // 如果提供，则为编辑模式
  columnId?: string; // 默认列ID
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'saved'): void;
}>();

const taskStore = useTaskStore();
const kanbanStore = useKanbanStore();

const isEdit = ref(!!props.task);
const title = ref(props.task?.title ?? '');
const description = ref(props.task?.description ?? '');
const category = ref<TaskCategory>(props.task?.category ?? 'work');
const estimatedHours = ref(props.task ? props.task.estimated_minutes / 60 : 1);
const selectedColumn = ref(props.columnId ?? 'todo');

// 进程绑定
const enableProcessBinding = ref(false);
const appName = ref('');
const autoStart = ref(true);
const autoPause = ref(true);

// 提醒设置
const enableReminder = ref(false);
const reminderTime = ref('09:00');

const errorMsg = ref('');
const isSubmitting = ref(false);

onMounted(async () => {
  if (kanbanStore.columns.length === 0) {
    await kanbanStore.loadBoardConfig();
  }
});

async function handleSubmit() {
  if (!title.value.trim()) {
    errorMsg.value = '请输入任务标题';
    return;
  }
  
  if (estimatedHours.value <= 0) {
    errorMsg.value = '预计时长必须大于0';
    return;
  }
  
  errorMsg.value = '';
  isSubmitting.value = true;
  
  try {
    const minutes = Math.round(estimatedHours.value * 60);
    
    if (isEdit.value && props.task) {
      // 更新任务
      await taskStore.updateTask(
        props.task.id,
        title.value.trim(),
        description.value.trim(),
        minutes,
        undefined,
        undefined
      );
    } else {
      // 创建任务
      await taskStore.createTask(
        title.value.trim(),
        description.value.trim(),
        category.value,
        minutes
      );
    }
    
    // TODO: 保存看板相关设置（进程绑定、提醒）
    // 这部分需要 kanban store 支持
    
    emit('saved');
  } catch (e: any) {
    errorMsg.value = String(e);
  } finally {
    isSubmitting.value = false;
  }
}
</script>

<template>
  <div class="card-form-overlay" @click.self="emit('close')">
    <div class="card-form">
      <h3>{{ isEdit ? '编辑任务' : '创建任务' }}</h3>
      
      <div class="form-group">
        <label>标题</label>
        <input v-model="title" placeholder="输入任务标题" />
      </div>
      
      <div class="form-group">
        <label>描述</label>
        <textarea v-model="description" placeholder="输入任务描述（可选）" rows="3" />
      </div>
      
      <div class="form-row">
        <div class="form-group">
          <label>分类</label>
          <select v-model="category">
            <option value="work">工作</option>
            <option value="study">学习</option>
            <option value="other">其他</option>
          </select>
        </div>
        
        <div class="form-group">
          <label>预计时长（小时）</label>
          <input v-model.number="estimatedHours" type="number" min="0.5" step="0.5" />
        </div>
      </div>
      
      <div class="form-group">
        <label>所属列</label>
        <select v-model="selectedColumn">
          <option v-for="col in kanbanStore.sortedColumns" :key="col.id" :value="col.id">
            {{ col.title }}
          </option>
        </select>
      </div>
      
      <hr />
      
      <div class="form-group">
        <label>
          <input v-model="enableProcessBinding" type="checkbox" />
          绑定进程（自动检测）
        </label>
      </div>
      
      <div v-if="enableProcessBinding" class="indent">
        <div class="form-group">
          <label>进程名称</label>
          <input v-model="appName" placeholder="如：opencode.exe" />
          <small>输入进程名称（含.exe后缀）</small>
        </div>
        
        <div class="form-group">
          <label>
            <input v-model="autoStart" type="checkbox" />
            检测到进程时自动开始
          </label>
        </div>
        
        <div class="form-group">
          <label>
            <input v-model="autoPause" type="checkbox" />
            进程结束时自动暂停
          </label>
        </div>
      </div>
      
      <div class="form-group">
        <label>
          <input v-model="enableReminder" type="checkbox" />
          设置提醒
        </label>
      </div>
      
      <div v-if="enableReminder" class="indent">
        <div class="form-group">
          <label>提醒时间</label>
          <input v-model="reminderTime" type="time" />
        </div>
      </div>
      
      <div v-if="errorMsg" class="error-msg">{{ errorMsg }}</div>
      
      <div class="form-actions">
        <button class="btn-cancel" @click="emit('close')">取消</button>
        <button class="btn-submit" :disabled="isSubmitting" @click="handleSubmit">
          {{ isSubmitting ? '提交中...' : (isEdit ? '保存' : '创建') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.card-form-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.card-form {
  background: var(--color-surface);
  padding: 24px;
  border-radius: 12px;
  width: 500px;
  max-height: 90vh;
  overflow-y: auto;
}

.card-form h3 {
  margin: 0 0 16px;
  font-size: 1.125rem;
  color: var(--color-text);
}

.form-group {
  margin-bottom: 12px;
}

.form-group label {
  display: block;
  font-size: 0.875rem;
  color: var(--color-text-muted);
  margin-bottom: 4px;
}

.form-group input[type="text"],
.form-group input[type="number"],
.form-group input[type="time"],
.form-group textarea,
.form-group select {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 0.875rem;
  background: var(--color-bg);
  color: var(--color-text);
}

.form-group small {
  display: block;
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin-top: 4px;
}

.form-row {
  display: flex;
  gap: 12px;
}

.form-row .form-group {
  flex: 1;
}

.indent {
  padding-left: 16px;
  border-left: 2px solid var(--color-border);
  margin-left: 8px;
}

hr {
  border: none;
  border-top: 1px solid var(--color-border);
  margin: 16px 0;
}

.error-msg {
  color: #e06040;
  font-size: 0.875rem;
  margin-bottom: 12px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}

.btn-cancel {
  padding: 8px 16px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 0.875rem;
  color: var(--color-text);
  cursor: pointer;
}

.btn-submit {
  padding: 8px 16px;
  background: var(--color-primary);
  border: none;
  border-radius: 6px;
  font-size: 0.875rem;
  color: #1a1a24;
  cursor: pointer;
  font-weight: 600;
}

.btn-submit:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
