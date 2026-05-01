<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useTaskStore } from "@/stores/task";
import { useFaithStore } from "@/stores/faith";

const taskStore = useTaskStore();
const faithStore = useFaithStore();

const WORK_TARGET_MINUTES = 480;
const STUDY_TARGET_MINUTES = 480;
const DAILY_FAITH_MAX = 1000;

onMounted(async () => {
  const today = new Date().toISOString().slice(0, 10);
  await taskStore.fetchDailyStats(today);
});

const workProgress = computed(() => {
  const record = faithStore.todayRecord;
  const workMin = record?.work_minutes ?? 0;
  return Math.min(100, (workMin / WORK_TARGET_MINUTES) * 100);
});

const studyProgress = computed(() => {
  const record = faithStore.todayRecord;
  const studyMin = record?.study_minutes ?? 0;
  return Math.min(100, (studyMin / STUDY_TARGET_MINUTES) * 100);
});

const workMinutes = computed(() => faithStore.todayRecord?.work_minutes ?? 0);
const studyMinutes = computed(() => faithStore.todayRecord?.study_minutes ?? 0);

const todayTotalFaith = computed(() => faithStore.todayRecord?.total_faith ?? 0);

function formatMin(min: number): string {
  const h = Math.floor(min / 60);
  const m = min % 60;
  return m > 0 ? `${h}h${m}m` : `${h}h`;
}
</script>

<template>
  <section class="daily-goal-panel">
    <h3 class="panel-title">今日目标</h3>

    <div class="goal-item">
      <div class="goal-label-row">
        <span class="goal-icon">💼</span>
        <span class="goal-label">工作</span>
        <span class="goal-count">{{ formatMin(workMinutes) }} / 8h</span>
      </div>
      <div class="goal-track">
        <div
          class="goal-fill survival"
          :style="{ width: workProgress + '%' }"
        ></div>
      </div>
    </div>

    <div class="goal-item">
      <div class="goal-label-row">
        <span class="goal-icon">📚</span>
        <span class="goal-label">学习</span>
        <span class="goal-count">{{ formatMin(studyMinutes) }} / 8h</span>
      </div>
      <div class="goal-track">
        <div
          class="goal-fill progress"
          :style="{ width: studyProgress + '%' }"
        ></div>
      </div>
    </div>

    <div v-if="taskStore.dailyStats" class="bonus-summary">
      <span class="bonus-label">任务加成</span>
      <span class="bonus-value">
        <span v-if="taskStore.dailyStats.task_bonus_work > 0" class="bonus-tag work">工作 +{{ taskStore.dailyStats.task_bonus_work }}</span>
        <span v-if="taskStore.dailyStats.task_bonus_study > 0" class="bonus-tag study">学习 +{{ taskStore.dailyStats.task_bonus_study }}</span>
        <span v-if="taskStore.dailyStats.task_bonus_work === 0 && taskStore.dailyStats.task_bonus_study === 0" class="bonus-none">无</span>
      </span>
    </div>

    <div class="faith-max-hint">
      每日信仰上限: {{ DAILY_FAITH_MAX }} · 今日已获: {{ todayTotalFaith }}
    </div>
  </section>
</template>

<style scoped>
.daily-goal-panel {
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

.goal-item {
  margin-bottom: 14px;
}

.goal-label-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
}

.goal-icon {
  font-size: 0.875rem;
}

.goal-label {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  flex: 1;
}

.goal-count {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  font-variant-numeric: tabular-nums;
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
  transition: width 0.5s ease;
}

.goal-fill.survival { background: var(--color-survival); }
.goal-fill.progress { background: var(--color-progress); }

.bonus-summary {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 14px;
  padding-top: 14px;
  border-top: 1px solid var(--color-border);
}

.bonus-label {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.bonus-value {
  display: flex;
  gap: 6px;
}

.bonus-tag {
  font-size: 0.75rem;
  padding: 2px 8px;
  border-radius: 6px;
  font-weight: 500;
}

.bonus-tag.work {
  background: var(--color-survival);
  color: #1a1a24;
}

.bonus-tag.study {
  background: var(--color-progress);
  color: #1a1a24;
}

.bonus-none {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.faith-max-hint {
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px solid var(--color-border);
  font-size: 0.75rem;
  color: var(--color-text-muted);
  text-align: center;
}
</style>
