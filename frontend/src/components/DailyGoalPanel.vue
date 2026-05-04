<template>
  <div class="goal-panel">
    <div class="goal-title">每日目标</div>
    <div class="goal-item">
      <span>工作进度: {{ workMin }} / 480 分钟</span>
      <div class="goal-bar"><div class="goal-fill work" :style="{ width: Math.min(100, workMin / 480 * 100) + '%' }"></div></div>
    </div>
    <div class="goal-item">
      <span>学习进度: {{ studyMin }} / 480 分钟</span>
      <div class="goal-bar"><div class="goal-fill study" :style="{ width: Math.min(100, studyMin / 480 * 100) + '%' }"></div></div>
    </div>
    <div class="goal-max">信仰上限: 1000</div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useFaithStore } from '@/stores/faith';

const faith = useFaithStore();
const workMin = computed(() => faith.todayRecord?.work_minutes ?? 0);
const studyMin = computed(() => faith.todayRecord?.study_minutes ?? 0);
</script>

<style scoped>
.goal-panel { background: var(--color-surface); border-radius: 8px; padding: 12px; }
.goal-title { font-weight: 600; margin-bottom: 8px; }
.goal-item { font-size: 0.8rem; color: var(--color-text-muted); margin-bottom: 4px; }
.goal-item span { display: block; margin-bottom: 2px; }
.goal-bar { height: 8px; background: var(--color-bg); border-radius: 4px; overflow: hidden; }
.goal-fill { height: 100%; border-radius: 4px; }
.goal-fill.work { background: #ef4444; }
.goal-fill.study { background: #3b82f6; }
.goal-max { margin-top: 4px; font-size: 0.75rem; color: var(--color-text-muted); }
</style>
