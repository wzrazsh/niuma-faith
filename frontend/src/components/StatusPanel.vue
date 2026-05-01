<script setup lang="ts">
import { computed } from "vue";
import { useFaithStore } from "@/stores/faith";
import { formatNumber } from "@/utils/format";

const store = useFaithStore();

const hasRecordToday = computed(() => store.todayRecord !== null);

// progress_to_next is faith needed to reach next level
const progressNeeded = computed(() => store.progressToNext ?? 0);
const cumulativeFaith = computed(() => store.cumulativeFaith ?? 0);
const nextThreshold = computed(() => store.nextLevelThreshold ?? null);

// Progress toward next level: 0..100
const percentToNext = computed(() => {
  if (!nextThreshold.value) return 100;
  const total = nextThreshold.value - (store.currentLevel?.cumulative_threshold ?? 0);
  if (total <= 0) return 100;
  const made = cumulativeFaith.value - (store.currentLevel?.cumulative_threshold ?? 0);
  return Math.min(100, Math.max(0, (made / total) * 100));
});

const armor = computed(() => store.faithStatus?.armor ?? 0);
const totalArmor = computed(() => store.faithStatus?.total_armor ?? 0);

const armorPercent = computed(() => {
  if (totalArmor.value <= 0) return 0;
  return Math.min(100, (armor.value / totalArmor.value) * 100);
});
</script>

<template>
  <section class="status-panel">
    <!-- Level Card -->
    <div v-if="store.faithStatus" class="level-card">
      <div class="level-badge">
        <span class="level-num">Lv.{{ store.currentLevel.level }}</span>
        <span class="level-title">{{ store.currentLevel.title }}</span>
      </div>

      <div class="progress-section">
        <div class="progress-header">
          <span class="progress-label">距下一级</span>
          <span class="progress-threshold">
            {{ formatNumber(cumulativeFaith) }}
            <span class="threshold-sep">/</span>
            {{ nextThreshold != null ? formatNumber(nextThreshold) : "MAX" }}
          </span>
        </div>
        <div class="progress-track">
          <div
            class="progress-fill"
            :style="{ width: percentToNext + '%' }"
          ></div>
        </div>
        <p v-if="progressNeeded > 0" class="progress-hint">
          再积累
          <strong>{{ formatNumber(progressNeeded) }}</strong>
          信仰可升至下一级
        </p>
        <p v-else-if="store.currentLevel.level < 15" class="progress-hint">
          继续积累即可升级
        </p>
        <p v-else class="progress-hint progress-max">
          已达到最高等级 · 牛马圣徒
        </p>
      </div>

      <!-- Armor Meter -->
      <div v-if="totalArmor > 0" class="armor-section">
        <div class="armor-header">
          <span class="armor-label">戒律护甲</span>
          <span class="armor-value">{{ formatNumber(armor) }} / {{ formatNumber(totalArmor) }}</span>
        </div>
        <div class="armor-track">
          <div
            class="armor-fill"
            :style="{ width: armorPercent + '%' }"
          ></div>
        </div>
      </div>
    </div>

    <div v-else-if="store.isLoading" class="status-loading">
      <p>加载中...</p>
    </div>

    <!-- Today's Faith Breakdown -->
    <div class="faith-breakdown">
      <h3 class="breakdown-title">今日信仰</h3>

      <div v-if="hasRecordToday" class="breakdown-list">
        <div class="breakdown-item">
          <span class="breakdown-dot survival"></span>
          <span class="breakdown-label">生存信仰</span>
          <span class="breakdown-value survival">
            {{ store.todayFaith.survival_faith }}
          </span>
        </div>
        <div class="breakdown-item">
          <span class="breakdown-dot progress"></span>
          <span class="breakdown-label">精进信仰</span>
          <span class="breakdown-value progress">
            {{ store.todayFaith.progress_faith }}
          </span>
        </div>
        <div class="breakdown-item">
          <span class="breakdown-dot discipline"></span>
          <span class="breakdown-label">戒律信仰</span>
          <span class="breakdown-value discipline">
            {{ store.todayFaith.discipline_faith }}
          </span>
        </div>
        <div class="breakdown-divider"></div>
        <div class="breakdown-item breakdown-total">
          <span class="breakdown-label">今日总计</span>
          <span class="breakdown-value total">
            {{ store.todayFaith.total_faith }}
          </span>
        </div>
      </div>

      <p v-else class="no-record-hint">
        今日尚未打卡，输入时长并提交即可积累信仰
      </p>
    </div>

    <!-- Cumulative total -->
    <div v-if="store.faithStatus" class="cumulative-stat">
      <span class="stat-label">累计信仰</span>
      <span class="stat-value">{{ formatNumber(cumulativeFaith) }}</span>
    </div>
  </section>
</template>

<style scoped>
.status-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Level Card */
.level-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 16px;
  padding: 20px 24px;
}

.level-badge {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin-bottom: 20px;
}

.level-num {
  font-size: 1.5rem;
  font-weight: 800;
  color: var(--color-primary);
}

.level-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
  letter-spacing: 2px;
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-label {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.progress-threshold {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  font-variant-numeric: tabular-nums;
}

.threshold-sep {
  margin: 0 2px;
  color: var(--color-border);
}

.progress-track {
  height: 8px;
  background: var(--color-bg);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--color-primary-dark), var(--color-primary));
  border-radius: 4px;
  transition: width 0.6s ease;
}

.progress-hint {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin-top: 4px;
}

.progress-hint strong {
  color: var(--color-primary);
}

.progress-max {
  color: var(--color-discipline);
}

/* Faith Breakdown */
.faith-breakdown {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 16px;
  padding: 20px 24px;
}

.breakdown-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 16px;
}

.breakdown-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.breakdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.breakdown-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.breakdown-dot.survival { background: var(--color-survival); }
.breakdown-dot.progress { background: var(--color-progress); }
.breakdown-dot.discipline { background: var(--color-discipline); }

.breakdown-label {
  flex: 1;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.breakdown-value {
  font-size: 1rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.breakdown-value.survival { color: var(--color-survival); }
.breakdown-value.progress { color: var(--color-progress); }
.breakdown-value.discipline { color: var(--color-discipline); }
.breakdown-value.total { color: var(--color-primary); font-size: 1.125rem; }

.breakdown-divider {
  height: 1px;
  background: var(--color-border);
  margin: 4px 0;
}

.breakdown-total .breakdown-label {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--color-text);
}

.no-record-hint {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  text-align: center;
  padding: 16px 0;
}

/* Cumulative */
.cumulative-stat {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 12px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stat-label {
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.stat-value {
  font-size: 1.125rem;
  font-weight: 700;
  color: var(--color-primary);
  font-variant-numeric: tabular-nums;
}

.status-loading {
  padding: 24px;
  text-align: center;
  color: var(--color-text-muted);
  background: var(--color-surface);
  border-radius: 16px;
}

/* Armor Meter */
.armor-section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 14px 24px;
}

.armor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.armor-label {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.armor-value {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  font-variant-numeric: tabular-nums;
}

.armor-track {
  height: 8px;
  background: var(--color-bg);
  border-radius: 4px;
  overflow: hidden;
}

.armor-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--color-discipline), #a78bfa);
  border-radius: 4px;
  transition: width 0.6s ease;
}
</style>
