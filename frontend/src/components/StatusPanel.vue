<template>
  <div class="status-panel" v-if="faith.faithStatus">
    <div class="level-badge">
      <span class="level-num">Lv{{ faith.faithStatus.current_level }}</span>
      <span class="level-title">{{ faith.faithStatus.level_title }}</span>
    </div>
    <div class="progress-section">
      <div v-if="faith.faithStatus.next_threshold !== null" class="progress-bar">
        <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
      </div>
      <div v-else class="max-level">已达最高等级</div>
      <span class="progress-text" v-if="faith.faithStatus.next_threshold !== null">
        {{ formatNumber(faith.faithStatus.progress_to_next ?? 0) }} / {{ formatNumber(faith.faithStatus.next_threshold - (faith.faithStatus.cumulative_faith - (faith.faithStatus.progress_to_next ?? 0))) }}
      </span>
    </div>
    <div class="armor-bar">
      <div class="armor-fill" :style="{ width: armorPercent + '%' }"></div>
      <span class="armor-text">护甲: {{ formatNumber(faith.faithStatus.armor) }} / {{ formatNumber(faith.faithStatus.total_armor) }}</span>
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
  // Find lower threshold
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
.status-panel { background: var(--color-surface); border-radius: 8px; padding: 12px; }
.level-badge { display: flex; flex-direction: column; align-items: center; margin-bottom: 8px; }
.level-num { font-size: 1.5rem; font-weight: 700; color: var(--color-primary); }
.level-title { font-size: 0.85rem; color: var(--color-text-muted); }
.progress-section { margin-bottom: 8px; }
.progress-bar, .armor-bar { height: 12px; background: var(--color-bg); border-radius: 6px; overflow: hidden; position: relative; }
.progress-fill { height: 100%; background: var(--color-primary); border-radius: 6px; transition: width 0.3s; }
.armor-fill { height: 100%; background: var(--color-success); border-radius: 6px; transition: width 0.3s; }
.progress-text, .armor-text { font-size: 0.7rem; color: var(--color-text-muted); position: absolute; left: 50%; top: 50%; transform: translate(-50%, -50%); white-space: nowrap; }
.max-level { text-align: center; color: var(--color-primary); font-size: 0.85rem; padding: 4px; }
.armor-bar { margin-top: 4px; }
</style>
