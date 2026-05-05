<template>
  <div class="faith-dash" v-if="faith.todayRecord">
    <div class="faith-header">今日信仰</div>
    <div class="faith-total">
      <span class="faith-amount">{{ faith.todayRecord.total_faith }}</span>
      <span class="faith-max">/ 1000</span>
    </div>
    <div class="faith-source">
      <div class="source-item work">
        <span class="source-dot"></span>
        <span>生存 {{ faith.todayRecord.survival_faith }}</span>
        <span class="source-detail">{{ faith.todayRecord.work_minutes }}分钟</span>
      </div>
      <div class="source-item study">
        <span class="source-dot"></span>
        <span>精进 {{ faith.todayRecord.progress_faith }}</span>
        <span class="source-detail">{{ faith.todayRecord.study_minutes }}分钟</span>
      </div>
      <div class="source-item discipline">
        <span class="source-dot"></span>
        <span>戒律 {{ faith.todayRecord.discipline_faith }}</span>
        <span class="source-detail">专注{{ faith.todayRecord.discipline_a }}/离岗{{ faith.todayRecord.discipline_b }}/闭环{{ faith.todayRecord.discipline_c }}</span>
      </div>
    </div>
    <div v-if="faith.todayRecord.task_bonus_work || faith.todayRecord.task_bonus_study" class="task-bonus">
      <span class="bonus-label">任务加成</span>
      <span class="bonus-values">
        <span v-if="faith.todayRecord.task_bonus_work" class="bonus-work">工作 +{{ faith.todayRecord.task_bonus_work }}</span>
        <span v-if="faith.todayRecord.task_bonus_study" class="bonus-study">学习 +{{ faith.todayRecord.task_bonus_study }}</span>
      </span>
    </div>
    <div class="tasks-completed">
      已完成 {{ faith.todayRecord.tasks_completed }} 个任务
    </div>
  </div>
  <div class="faith-dash empty" v-else>
    <div class="empty-icon">◈</div>
    <span>暂无今日记录</span>
  </div>
</template>

<script setup lang="ts">
import { useFaithStore } from '@/stores/faith';
const faith = useFaithStore();
</script>

<style scoped>
.faith-dash {
  background: var(--color-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.faith-header {
  font-family: var(--font-display);
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-text-muted);
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.faith-total {
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.faith-amount {
  font-family: var(--font-display);
  font-size: 1.8rem;
  font-weight: 900;
  color: var(--color-primary);
  text-shadow: 0 0 16px var(--color-primary-glow);
  line-height: 1;
}

.faith-max {
  font-size: 0.85rem;
  color: var(--color-text-dim);
  font-family: var(--font-mono);
}

.faith-source {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.source-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.8rem;
  color: var(--color-text-muted);
}

.source-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.source-item.work .source-dot { background: var(--color-work); }
.source-item.study .source-dot { background: var(--color-study); }
.source-item.discipline .source-dot { background: var(--color-primary); }

.source-detail {
  margin-left: auto;
  font-size: 0.72rem;
  color: var(--color-text-dim);
  font-family: var(--font-mono);
}

.task-bonus {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: var(--color-bg);
  border-radius: var(--radius-sm);
}

.bonus-label {
  font-size: 0.75rem;
  color: var(--color-text-dim);
}

.bonus-values {
  display: flex;
  gap: 8px;
  margin-left: auto;
  font-size: 0.78rem;
  font-weight: 600;
}

.bonus-work { color: var(--color-work); }
.bonus-study { color: var(--color-study); }

.tasks-completed {
  font-size: 0.75rem;
  color: var(--color-text-dim);
  text-align: center;
}

.empty {
  align-items: center;
  gap: 8px;
  padding: 20px;
}

.empty-icon {
  font-size: 1.2rem;
  color: var(--color-text-dim);
  opacity: 0.5;
}
</style>
