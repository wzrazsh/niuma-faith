<template>
  <div class="floating-widget" role="region" aria-label="牛马信仰悬浮组件">
    <div class="drag-handle" @mousedown="startDrag" @dblclick="toggleMaximize">
      <span class="title">牛马信仰</span>
      <button class="close-btn" @click="close" aria-label="关闭悬浮窗口">×</button>
    </div>
    <div class="content">
      <div class="faith-display" role="status" aria-live="polite">
        <div class="level" aria-label="当前等级">Lv.{{ status?.current_level || 1 }}</div>
        <div class="faith-value">{{ status?.cumulative_faith || 0 }} 信仰</div>
      </div>
      <div
        class="today-progress"
        role="progressbar"
        :aria-valuenow="todayFaith"
        aria-valuemin="0"
        aria-valuemax="100"
        :aria-label="`今日信仰进度 ${todayFaith}/100`"
      >
        <div class="progress-label">今日</div>
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: todayPercent + '%' }"></div>
        </div>
        <div class="progress-text">{{ todayFaith }}/100</div>
      </div>
      <div v-if="error" class="error-message" role="alert">{{ error }}</div>
      <div class="quick-actions" role="group" aria-label="快速操作">
        <button @click="quickCheckIn(4, 0)" :disabled="loading" aria-label="快速打卡工作4小时">工作</button>
        <button @click="quickCheckIn(0, 4)" :disabled="loading" aria-label="快速打卡学习4小时">学习</button>
        <button @click="quickCheckIn(2, 2)" :disabled="loading" aria-label="快速打卡混合2小时">混合</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { FaithStatus } from '@/types';

const status = ref<FaithStatus | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const cancelled = ref(false);
let currentController: AbortController | null = null;

const todayFaith = computed(() => {
  if (!status.value?.today) return 0;
  const t = status.value.today;
  return (t.survival_faith || 0) + (t.progress_faith || 0) + (t.discipline_faith || 0);
});

const todayPercent = computed(() => Math.min((todayFaith.value / 100) * 100, 100));

async function loadStatus() {
  currentController?.abort();
  currentController = new AbortController();

  try {
    error.value = null;
    status.value = await invoke<FaithStatus>('get_status', {
      userId: 'default_user',
      signal: currentController.signal,
    });
  } catch (e) {
    if (currentController?.signal.aborted) return;
    error.value = '加载状态失败，请重试';
    console.error('Failed to load status:', e);
  }
}

async function quickCheckIn(workHours: number, studyHours: number) {
  loading.value = true;
  error.value = null;
  currentController?.abort();
  currentController = new AbortController();

  try {
    status.value = await invoke<FaithStatus>('check_in', {
      userId: 'default_user',
      workMinutes: workHours * 60,
      studyMinutes: studyHours * 60,
      signal: currentController.signal,
    });
  } catch (e) {
    if (currentController?.signal.aborted) return;
    error.value = '打卡失败，请重试';
    console.error('Check-in failed:', e);
  }
  loading.value = false;
}

async function close() {
  try {
    await invoke('close_floating_widget');
  } catch (e) {
    error.value = '关闭失败，请重试';
    console.error('Close failed:', e);
  }
}

function startDrag(_e: MouseEvent) {
  // Drag is handled by window decorations: false in main.rs
}

function toggleMaximize() {
  // Future: toggle between compact and expanded view
}

onMounted(() => {
  cancelled.value = false;
  loadStatus();
});

onUnmounted(() => {
  cancelled.value = true;
  currentController?.abort();
});
</script>

<style scoped>
.floating-widget {
  width: 280px;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  color: #fff;
  user-select: none;
}

.drag-handle {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: rgba(0, 0, 0, 0.3);
  cursor: move;
}

.title {
  font-size: 12px;
  font-weight: 600;
  color: #a0a0a0;
}

.close-btn {
  width: 18px;
  height: 18px;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
  border-radius: 50%;
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: rgba(255, 100, 100, 0.5);
}

.content {
  padding: 12px;
}

.faith-display {
  text-align: center;
  margin-bottom: 10px;
}

.level {
  font-size: 28px;
  font-weight: 700;
  color: #ffd700;
  text-shadow: 0 0 10px rgba(255, 215, 0, 0.5);
}

.faith-value {
  font-size: 11px;
  color: #a0a0a0;
  margin-top: 2px;
}

.today-progress {
  margin-bottom: 10px;
}

.progress-label {
  font-size: 10px;
  color: #a0a0a0;
  margin-bottom: 4px;
}

.progress-bar {
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #00d4ff, #00ff88);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 10px;
  color: #00ff88;
  text-align: right;
  margin-top: 2px;
}

.error-message {
  font-size: 10px;
  color: #ff6b6b;
  background: rgba(255, 107, 107, 0.1);
  border: 1px solid rgba(255, 107, 107, 0.3);
  border-radius: 4px;
  padding: 6px 8px;
  margin-bottom: 8px;
  text-align: center;
}

.quick-actions {
  display: flex;
  gap: 6px;
}

.quick-actions button {
  flex: 1;
  padding: 6px 0;
  border: none;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.quick-actions button:nth-child(1) {
  background: rgba(255, 200, 0, 0.2);
  color: #ffc800;
}

.quick-actions button:nth-child(1):hover:not(:disabled) {
  background: rgba(255, 200, 0, 0.4);
}

.quick-actions button:nth-child(2) {
  background: rgba(0, 200, 255, 0.2);
  color: #00c8ff;
}

.quick-actions button:nth-child(2):hover:not(:disabled) {
  background: rgba(0, 200, 255, 0.4);
}

.quick-actions button:nth-child(3) {
  background: rgba(200, 100, 255, 0.2);
  color: #c864ff;
}

.quick-actions button:nth-child(3):hover:not(:disabled) {
  background: rgba(200, 100, 255, 0.4);
}

.quick-actions button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>