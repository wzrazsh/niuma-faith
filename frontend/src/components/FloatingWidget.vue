<template>
  <div
    class="floating-icon"
    role="button"
    aria-label="打开牛马信仰主窗口"
    @mousedown="startDrag"
    @click="showMain"
  >
    <div class="level-circle">
      <span class="level-text">Lv.{{ status?.current_level || 1 }}</span>
      <span class="faith-text">{{ status?.cumulative_faith || 0 }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { FaithStatus } from '@/types';

const status = ref<FaithStatus | null>(null);
let pollTimer: ReturnType<typeof setInterval> | null = null;

function startDrag(_e: MouseEvent) {
  getCurrentWindow().startDragging();
}

async function loadStatus() {
  try {
    status.value = await invoke<FaithStatus>('get_status', {
      userId: 'default_user',
    });
  } catch (e) {
    console.error('Failed to load status:', e);
  }
}

async function showMain() {
  try {
    await invoke('show_main_window');
  } catch (e) {
    console.error('Failed to show main window:', e);
  }
}

onMounted(() => {
  loadStatus();
  pollTimer = setInterval(loadStatus, 10000);
});

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
});
</script>

<style scoped>
.floating-icon {
  width: 80px;
  height: 80px;
  cursor: grab;
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
}

.level-circle {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  border: 2px solid rgba(255, 215, 0, 0.4);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4), 0 0 12px rgba(255, 215, 0, 0.15);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  overflow: hidden;
}

.level-circle:hover {
  border-color: rgba(255, 215, 0, 0.8);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5), 0 0 20px rgba(255, 215, 0, 0.25);
  transform: scale(1.05);
}

.level-circle:active {
  transform: scale(0.95);
  cursor: grabbing;
}

.level-text {
  font-size: 14px;
  font-weight: 700;
  color: #ffd700;
  text-shadow: 0 0 8px rgba(255, 215, 0, 0.5);
  line-height: 1;
}

.faith-text {
  font-size: 8px;
  color: #a0a0a0;
  margin-top: 1px;
  line-height: 1;
}
</style>

<style>
html, body, #app {
  background: transparent !important;
  margin: 0;
  padding: 0;
  min-height: auto !important;
  overflow: hidden;
}
</style>
