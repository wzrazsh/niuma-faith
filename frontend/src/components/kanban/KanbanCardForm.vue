<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal">
      <div class="modal-header">
        <span>编辑任务</span>
        <button @click="$emit('close')">×</button>
      </div>
      <div class="modal-body">
        <input v-model="title" placeholder="标题" />
        <div class="field-row">
          <select v-model="category">
            <option value="work">work</option>
            <option value="study">study</option>
            <option value="other">other</option>
          </select>
          <input v-model.number="estimated" type="number" placeholder="预计(分钟)" />
        </div>
        <div class="binding-section">
          <div class="section-title">进程绑定</div>
          <input v-model="appName" placeholder="应用名 (如 notepad.exe)" />
          <label class="checkbox"><input type="checkbox" v-model="autoStart" /> 启动时自动开始</label>
          <label class="checkbox"><input type="checkbox" v-model="autoPause" /> 结束时自动暂停</label>
        </div>
      </div>
      <div class="modal-footer">
        <button @click="$emit('close')">取消</button>
        <button class="primary" @click="save">保存</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useTaskStore } from '@/stores/task';

const emit = defineEmits<{ close: [] }>();
const props = defineProps<{ taskId?: string }>();

const taskStore = useTaskStore();
const title = ref('');
const category = ref('work');
const estimated = ref(30);
const appName = ref('');
const autoStart = ref(false);
const autoPause = ref(false);

async function save() {
  if (!title.value.trim()) return;
  if (props.taskId) {
    await taskStore.updateTask(props.taskId, { title: title.value, estimatedMinutes: estimated.value });
  }
  emit('close');
}
</script>

<style scoped>
.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 100; }
.modal { background: var(--color-surface); border-radius: 8px; width: 420px; }
.modal-header { display: flex; justify-content: space-between; padding: 12px 16px; border-bottom: 1px solid var(--color-border); }
.modal-body { padding: 16px; display: flex; flex-direction: column; gap: 8px; }
.modal-footer { padding: 12px 16px; border-top: 1px solid var(--color-border); display: flex; justify-content: flex-end; gap: 8px; }
.field-row { display: flex; gap: 8px; }
.field-row input { width: 100%; }
.binding-section { border: 1px solid var(--color-border); border-radius: 6px; padding: 8px; display: flex; flex-direction: column; gap: 4px; }
.section-title { font-size: 0.8rem; color: var(--color-text-muted); }
.checkbox { font-size: 0.8rem; display: flex; align-items: center; gap: 4px; }
</style>
