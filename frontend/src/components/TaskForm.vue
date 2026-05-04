<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal">
      <div class="modal-header">
        <span>新建任务</span>
        <button @click="$emit('close')">×</button>
      </div>
      <div class="modal-body">
        <input v-model="title" placeholder="标题" />
        <textarea v-model="description" placeholder="描述" rows="2"></textarea>
        <div class="field-row">
          <select v-model="category">
            <option value="work">work</option>
            <option value="study">study</option>
            <option value="other">other</option>
          </select>
          <input v-model.number="estimated" type="number" placeholder="预计时长(分钟)" min="1" />
        </div>
        <label class="checkbox"><input type="checkbox" v-model="daily" /> 每日执行</label>
      </div>
      <div class="modal-footer">
        <button @click="$emit('close')">取消</button>
        <button class="primary" @click="submit">保存</button>
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
.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 100; }
.modal { background: var(--color-surface); border-radius: 8px; width: 400px; max-width: 90vw; }
.modal-header { display: flex; justify-content: space-between; padding: 12px 16px; border-bottom: 1px solid var(--color-border); }
.modal-body { padding: 16px; display: flex; flex-direction: column; gap: 8px; }
.modal-footer { padding: 12px 16px; border-top: 1px solid var(--color-border); display: flex; justify-content: flex-end; gap: 8px; }
.field-row { display: flex; gap: 8px; }
.field-row input { width: 100%; }
.checkbox { font-size: 0.85rem; display: flex; align-items: center; gap: 4px; }
</style>
