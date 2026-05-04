<template>
  <div class="faith-dash" v-if="faith.todayRecord">
    <div class="faith-total">今日信仰: {{ faith.todayRecord.total_faith }} / 1000</div>
    <div class="faith-breakdown">
      <div>生存信仰 {{ faith.todayRecord.survival_faith }} (工作 {{ faith.todayRecord.work_minutes }}分钟)</div>
      <div>精进信仰 {{ faith.todayRecord.progress_faith }} (学习 {{ faith.todayRecord.study_minutes }}分钟)</div>
      <div>戒律信仰 {{ faith.todayRecord.discipline_faith }} (专注{{ faith.todayRecord.discipline_a }}/离岗{{ faith.todayRecord.discipline_b }}/闭环{{ faith.todayRecord.discipline_c }})</div>
    </div>
    <div class="faith-bonus">
      任务加成: work +{{ faith.todayRecord.task_bonus_work }} / study +{{ faith.todayRecord.task_bonus_study }}
    </div>
    <div class="tasks-done">已完成任务: {{ faith.todayRecord.tasks_completed }}</div>
  </div>
  <div class="faith-dash empty" v-else>
    暂无今日记录
  </div>
</template>

<script setup lang="ts">
import { useFaithStore } from '@/stores/faith';
const faith = useFaithStore();
</script>

<style scoped>
.faith-dash { background: var(--color-surface); border-radius: 8px; padding: 12px; }
.faith-total { font-size: 1.1rem; font-weight: 600; color: var(--color-primary); margin-bottom: 8px; }
.faith-breakdown { font-size: 0.8rem; color: var(--color-text-muted); display: flex; flex-direction: column; gap: 2px; }
.faith-bonus { font-size: 0.8rem; color: var(--color-success); margin-top: 4px; }
.tasks-done { font-size: 0.8rem; color: var(--color-text-muted); margin-top: 4px; }
.empty { color: var(--color-text-muted); font-size: 0.85rem; text-align: center; }
</style>
