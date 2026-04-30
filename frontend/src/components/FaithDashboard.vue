<script setup lang="ts">
import { onMounted, computed } from "vue";
import { useFaithStore } from "@/stores/faith";

const faithStore = useFaithStore();

onMounted(async () => {
  await faithStore.fetchStatus();
});

const faithStatus = computed(() => faithStore.faithStatus);
const todayRecord = computed(() => faithStatus.value?.today ?? null);
</script>

<template>
  <section class="faith-dashboard">
    <h3 class="panel-title">今日信仰汇总</h3>

    <div v-if="todayRecord" class="faith-breakdown">
      <div class="breakdown-row">
        <span class="row-label">生存信仰</span>
        <span class="row-value survival">{{ todayRecord.survival_faith }}</span>
      </div>

      <div class="breakdown-row">
        <span class="row-label">精进信仰</span>
        <span class="row-value progress">{{ todayRecord.progress_faith }}</span>
      </div>

      <div class="breakdown-row">
        <span class="row-label">戒律信仰</span>
        <span class="row-value discipline">{{ todayRecord.discipline_faith }}</span>
      </div>

      <div class="row-divider"></div>

      <div class="breakdown-row total-row">
        <span class="row-label">今日总计</span>
        <span class="row-value total">{{ todayRecord.total_faith }}</span>
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

.panel-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 16px;
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
