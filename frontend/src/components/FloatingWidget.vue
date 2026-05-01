<template>
  <div
    class="level-circle"
    role="button"
    aria-label="双击打开牛马信仰主窗口"
    title="拖动移动 / 双击打开主窗口"
    @mousedown="onMouseDown"
    @dblclick="showMain"
  >
    <span class="level-text">Lv.{{ status?.current_level || 1 }}</span>
    <span class="faith-text">{{ status?.cumulative_faith || 0 }}</span>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { FaithStatus } from '@/types';

const status = ref<FaithStatus | null>(null);
let pollTimer: ReturnType<typeof setInterval> | null = null;

const DRAG_THRESHOLD_PX = 4;
let downPos: { x: number; y: number } | null = null;
let dragStarted = false;

function onMouseDown(e: MouseEvent) {
  if (e.button !== 0) return;
  downPos = { x: e.screenX, y: e.screenY };
  dragStarted = false;
  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseup', onMouseUp, { once: true });
}

function onMouseMove(e: MouseEvent) {
  if (!downPos || dragStarted) return;
  const dx = e.screenX - downPos.x;
  const dy = e.screenY - downPos.y;
  if (dx * dx + dy * dy >= DRAG_THRESHOLD_PX * DRAG_THRESHOLD_PX) {
    dragStarted = true;
    getCurrentWindow().startDragging();
  }
}

function onMouseUp() {
  downPos = null;
  window.removeEventListener('mousemove', onMouseMove);
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
  window.removeEventListener('mousemove', onMouseMove);
});
</script>

<style scoped>
.level-circle {
  width: 100vw;
  height: 100vh;
  border-radius: 50%;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  border: 2px solid rgba(255, 215, 0, 0.4);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4), 0 0 12px rgba(255, 215, 0, 0.15);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  transition: border-color 0.15s ease, box-shadow 0.15s ease, transform 0.15s ease;
  overflow: hidden;
  cursor: grab;
  user-select: none;
  box-sizing: border-box;
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
  background-color: transparent !important;
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  min-height: 0 !important;
  overflow: hidden;
}
</style>
