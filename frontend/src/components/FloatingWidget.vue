<template>
  <div class="widget">
    <div class="widget-bg"></div>
    <div class="widget-inner">
      <div class="widget-level">Lv.{{ level }}</div>
      <div class="widget-divider"></div>
      <div class="widget-faith">{{ faithStr }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useFaithStore } from '@/stores/faith';
import { formatNumber } from '@/utils/format';

const faith = useFaithStore();
const level = ref(1);
const faithStr = ref('0');

let timer: ReturnType<typeof setInterval>;

onMounted(async () => {
  await faith.init();
  level.value = faith.faithStatus?.current_level ?? 1;
  faithStr.value = formatNumber(faith.faithStatus?.cumulative_faith ?? 0);
  timer = setInterval(async () => {
    await faith.refreshStatus();
    level.value = faith.faithStatus?.current_level ?? 1;
    faithStr.value = formatNumber(faith.faithStatus?.cumulative_faith ?? 0);
  }, 5000);
});

onUnmounted(() => clearInterval(timer));
</script>

<style scoped>
.widget {
  width: 100px;
  height: 100px;
  border-radius: 50%;
  position: relative;
  overflow: hidden;
  cursor: grab;
  user-select: none;
}

.widget-bg {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background: conic-gradient(from 0deg, var(--color-surface), var(--color-primary-dim), var(--color-surface));
  animation: spin 8s linear infinite;
}

.widget-inner {
  position: absolute;
  inset: 4px;
  border-radius: 50%;
  background: var(--color-bg);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  border: 1px solid rgba(255, 215, 0, 0.15);
}

.widget-level {
  font-family: var(--font-display);
  font-size: 1rem;
  font-weight: 900;
  color: var(--color-primary);
  text-shadow: 0 0 8px var(--color-primary-glow);
}

.widget-divider {
  width: 20px;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--color-primary-dim), transparent);
}

.widget-faith {
  font-size: 0.65rem;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
