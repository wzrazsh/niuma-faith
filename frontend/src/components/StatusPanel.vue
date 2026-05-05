<template>
  <div class="status-panel" v-if="faith.faithStatus">
    <div class="level-badge">
      <div class="level-ring">
        <span class="level-num">Lv.{{ faith.faithStatus.current_level }}</span>
      </div>
      <span class="level-title">{{ faith.faithStatus.level_title }}</span>
    </div>

    <div class="stat-section">
      <div class="stat-label">信仰积累</div>
      <div v-if="faith.faithStatus.next_threshold !== null" class="progress-track">
        <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
      </div>
      <div v-else class="max-level">已达最高等级</div>
      <span class="stat-value" v-if="faith.faithStatus.next_threshold !== null">
        {{ formatNumber(faith.faithStatus.cumulative_faith) }}
        <span class="stat-divider">/</span>
        <span class="stat-target">{{ formatNumber(faith.faithStatus.next_threshold) }}</span>
      </span>
    </div>

    <div class="armor-section">
      <div class="armor-header">
        <span class="stat-label">护甲值</span>
        <span class="armor-value">{{ formatNumber(faith.faithStatus.armor) }} / {{ formatNumber(faith.faithStatus.total_armor) }}</span>
      </div>
      <div class="armor-track">
        <div class="armor-fill" :style="{ width: armorPercent + '%' }"></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useFaithStore } from '@/stores/faith';
import { formatNumber } from '@/utils/format';

const faith = useFaithStore();

const progressPercent = computed(() => {
  if (!faith.faithStatus || faith.faithStatus.next_threshold === null) return 100;
  const current = faith.faithStatus.cumulative_faith;
  const upper = faith.faithStatus.next_threshold;
  const lower = upper === 15000 ? 0 : upper > 15000 ? (faith.faithStatus.cumulative_faith - (faith.faithStatus.progress_to_next ?? 0)) : 0;
  const pos = current - lower;
  const range = upper - lower;
  return range > 0 ? Math.min(100, (pos / range) * 100) : 100;
});

const armorPercent = computed(() => {
  if (!faith.faithStatus || faith.faithStatus.total_armor === 0) return 100;
  return (faith.faithStatus.armor / faith.faithStatus.total_armor) * 100;
});
</script>

<style scoped>
.status-panel {
  background: var(--color-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  position: relative;
  overflow: hidden;
}

.status-panel::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--color-primary), transparent);
  opacity: 0.2;
}

.level-badge {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.level-ring {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  background: conic-gradient(from 0deg, var(--color-primary), var(--color-primary-dim), var(--color-primary));
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  animation: glow-pulse 3s ease-in-out infinite;
}

.level-ring::before {
  content: '';
  position: absolute;
  inset: 3px;
  border-radius: 50%;
  background: var(--color-surface);
}

.level-num {
  font-family: var(--font-display);
  font-size: 1.3rem;
  font-weight: 900;
  color: var(--color-primary);
  text-shadow: 0 0 12px var(--color-primary-glow);
  position: relative;
  z-index: 1;
}

.level-title {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  letter-spacing: 0.05em;
}

.stat-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.stat-label {
  font-size: 0.7rem;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-weight: 600;
}

.progress-track {
  height: 8px;
  background: var(--color-bg);
  border-radius: 4px;
  overflow: hidden;
  position: relative;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--color-primary-dim), var(--color-primary));
  border-radius: 4px;
  transition: width var(--transition-slow);
  position: relative;
}

.progress-fill::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.2), transparent);
  background-size: 200% 100%;
  animation: shimmer 2s ease-in-out infinite;
}

.stat-value {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}

.stat-divider {
  color: var(--color-text-dim);
  margin: 0 4px;
}

.stat-target {
  color: var(--color-primary-dim);
}

.max-level {
  text-align: center;
  color: var(--color-primary);
  font-size: 0.85rem;
  padding: 8px 0;
  font-family: var(--font-display);
}

.armor-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.armor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.armor-value {
  font-size: 0.75rem;
  color: var(--color-success);
  font-family: var(--font-mono);
}

.armor-track {
  height: 6px;
  background: var(--color-bg);
  border-radius: 3px;
  overflow: hidden;
}

.armor-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--color-success-dim), var(--color-success));
  border-radius: 3px;
  transition: width var(--transition-slow);
}
</style>
