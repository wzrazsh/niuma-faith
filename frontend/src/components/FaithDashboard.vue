<script setup lang="ts">
import { onMounted, computed } from "vue";
import { useFaithStore } from "@/stores/faith";
import { formatNumber } from "@/utils/format";

const faithStore = useFaithStore();
const DAILY_MAX = 1000;

onMounted(async () => {
  await faithStore.fetchStatus();
});

const faithStatus = computed(() => faithStore.faithStatus);
const todayRecord = computed(() => faithStatus.value?.today ?? null);

const todayTotalFaith = computed(() => todayRecord.value?.total_faith ?? 0);

const dailyMaxPercent = computed(() => {
  return Math.min(100, (todayTotalFaith.value / DAILY_MAX) * 100);
});
</script>

<template>
  <section class="faith-dashboard">
    <div class="panel-header">
      <h3 class="panel-title">今日信仰汇总</h3>
      <span class="daily-max-indicator">
        今日信仰: <strong>{{ formatNumber(todayTotalFaith) }}</strong> / {{ formatNumber(DAILY_MAX) }}
      </span>
    </div>

    <div class="daily-max-track">
      <div
        class="daily-max-fill"
        :style="{ width: dailyMaxPercent + '%' }"
      ></div>
    </div>

    <div v-if="todayRecord" class="faith-breakdown">
      <div class="breakdown-row">
        <span class="row-label">生存信仰</span>
        <span class="row-value survival">{{ formatNumber(todayRecord.survival_faith) }}</span>
      </div>

      <div class="breakdown-row">
        <span class="row-label">精进信仰</span>
        <span class="row-value progress">{{ formatNumber(todayRecord.progress_faith) }}</span>
      </div>

      <div class="breakdown-row">
        <span class="row-label">戒律信仰</span>
        <span class="row-value discipline">{{ formatNumber(todayRecord.discipline_faith) }}</span>
      </div>

      <div class="row-divider"></div>

      <div class="breakdown-row total-row">
        <span class="row-label">今日总计</span>
        <span class="row-value total">{{ formatNumber(todayRecord.total_faith) }}</span>
      </div>

      <div class="task-summary">
        <span class="task-summary-label">完成任务</span>
        <span class="task-summary-value">{{ todayRecord.tasks_completed }} 个</span>
      </div>
    </div>

    <p v-else class="no-record">今日暂无任务记录</p>
  </section>
</template>

<style scoped>
.faith-dashboard {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 16px;
  padding: 20px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.panel-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.daily-max-indicator {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.daily-max-indicator strong {
  color: var(--color-primary);
  font-weight: 700;
}

.daily-max-track {
  height: 4px;
  background: var(--color-bg);
  border-radius: 2px;
  overflow: hidden;
  margin-bottom: 18px;
}

.daily-max-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--color-primary-dark), var(--color-primary));
  border-radius: 2px;
  transition: width 0.6s ease;
}

.faith-breakdown {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.breakdown-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.row-label {
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.row-value {
  font-size: 1rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  display: flex;
  align-items: center;
  gap: 4px;
}

.row-value.survival { color: var(--color-survival); }
.row-value.progress { color: var(--color-progress); }
.row-value.discipline { color: var(--color-discipline); }
.row-value.total { color: var(--color-primary); font-size: 1.125rem; }

.bonus-inline {
  font-size: 0.8125rem;
  font-weight: 600;
  opacity: 0.8;
}

.row-divider {
  height: 1px;
  background: var(--color-border);
  margin: 4px 0;
}

.total-row {
  padding-top: 4px;
}

.task-summary {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid var(--color-border);
}

.task-summary-label {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.task-summary-value {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-discipline);
}

.no-record {
  text-align: center;
  color: var(--color-text-muted);
  font-size: 0.875rem;
  padding: 16px 0;
}
</style>
