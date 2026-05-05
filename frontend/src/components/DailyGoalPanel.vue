<template>
  <div class="goal-panel">
    <div class="goal-header">每日目标</div>
    <div class="goal-item">
      <div class="goal-label">
        <span class="goal-icon work-icon">◈</span>
        <span>工作进度</span>
        <span class="goal-stat">{{ workMin }} / 480 分钟</span>
      </div>
      <div class="goal-track">
        <div class="goal-fill work-fill" :style="{ width: Math.min(100, workMin / 480 * 100) + '%' }"></div>
      </div>
    </div>
    <div class="goal-item">
      <div class="goal-label">
        <span class="goal-icon study-icon">◇</span>
        <span>学习进度</span>
        <span class="goal-stat">{{ studyMin }} / 480 分钟</span>
      </div>
      <div class="goal-track">
        <div class="goal-fill study-fill" :style="{ width: Math.min(100, studyMin / 480 * 100) + '%' }"></div>
      </div>
    </div>
    <div class="goal-footer">
      信仰上限: <span class="faith-cap">1,000</span>
    </div>
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
.goal-panel {
  background: var(--color-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.goal-header {
  font-family: var(--font-display);
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-text-muted);
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.goal-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.goal-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.8rem;
  color: var(--color-text-muted);
}

.goal-icon {
  font-size: 0.7rem;
}

.work-icon { color: var(--color-work); }
.study-icon { color: var(--color-study); }

.goal-stat {
  margin-left: auto;
  font-size: 0.72rem;
  color: var(--color-text-dim);
  font-family: var(--font-mono);
}

.goal-track {
  height: 8px;
  background: var(--color-bg);
  border-radius: 4px;
  overflow: hidden;
}

.goal-fill {
  height: 100%;
  border-radius: 4px;
  transition: width var(--transition-slow);
  position: relative;
}

.work-fill {
  background: linear-gradient(90deg, var(--color-work), #e0406d);
}

.work-fill::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.15), transparent);
  background-size: 200% 100%;
  animation: shimmer 2.5s ease-in-out infinite;
}

.study-fill {
  background: linear-gradient(90deg, var(--color-study), #3b82f6);
}

.study-fill::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.15), transparent);
  background-size: 200% 100%;
  animation: shimmer 2.5s ease-in-out infinite;
  animation-delay: 0.5s;
}

.goal-footer {
  margin-top: 2px;
  font-size: 0.72rem;
  color: var(--color-text-dim);
  text-align: center;
}

.faith-cap {
  color: var(--color-primary);
  font-family: var(--font-mono);
}
</style>
