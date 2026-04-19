<script setup lang="ts">
import { ref, computed } from "vue";
import { useFaithStore } from "@/stores/faith";
import type { DisciplineInput } from "@/types";

const store = useFaithStore();

const workHours = ref(0);
const studyHours = ref(0);
// Discipline inputs
const breakCount = ref(0);
const leaveRecord = ref(0);
const closeRecord = ref(1);
const submitted = ref(false);

const workMinutes = computed(() => Math.round(workHours.value * 60));
const studyMinutes = computed(() => Math.round(studyHours.value * 60));

const discipline = computed<DisciplineInput>(() => ({
  break_count: breakCount.value,
  leave_record: leaveRecord.value,
  close_record: closeRecord.value,
}));

const canSubmit = computed(() => (workMinutes.value > 0 || studyMinutes.value > 0) && !store.isLoading);

async function handleSubmit() {
  submitted.value = false;
  try {
    await store.checkIn(workMinutes.value, studyMinutes.value, discipline.value);
    submitted.value = true;
  } catch {
    submitted.value = false;
  }
}
</script>

<template>
  <section class="checkin-card">
    <h2 class="card-title">今日打卡</h2>

    <div class="form-group">
      <label class="form-label">
        <span class="label-icon">💼</span>
        工作时长
      </label>
      <div class="input-row">
        <input
          v-model.number="workHours"
          type="number"
          class="form-input"
          min="0"
          max="24"
          step="0.5"
          placeholder="0"
        />
        <span class="input-unit">小时</span>
      </div>
      <span class="input-hint">今日已记录 {{ workMinutes }} 分钟</span>
    </div>

    <div class="form-group">
      <label class="form-label">
        <span class="label-icon">📚</span>
        学习时长
      </label>
      <div class="input-row">
        <input
          v-model.number="studyHours"
          type="number"
          class="form-input"
          min="0"
          max="24"
          step="0.5"
          placeholder="0"
        />
        <span class="input-unit">小时</span>
      </div>
      <span class="input-hint">今日已记录 {{ studyMinutes }} 分钟</span>
    </div>

    <div class="form-group">
      <label class="form-label">
        <span class="label-icon">⚡</span>
        专注稳定（中断次数）
      </label>
      <div class="input-row">
        <input
          v-model.number="breakCount"
          type="number"
          class="form-input"
          min="0"
          max="20"
          step="1"
          placeholder="0"
        />
        <span class="input-unit">次</span>
      </div>
      <span class="input-hint">≤2次得8分，3-4次得4分，≥5次得0分</span>
    </div>

    <div class="form-group">
      <label class="form-label">
        <span class="label-icon">🚶</span>
        离岗纪律
      </label>
      <div class="input-row">
        <select v-model.number="leaveRecord" class="form-input">
          <option value="0">无离岗 / 已解释</option>
          <option value="1">有离岗已解释</option>
          <option value="2">长时间失联未解释</option>
        </select>
      </div>
      <span class="input-hint">无得6分，有解释得3分，未解释得0分</span>
    </div>

    <div class="form-group">
      <label class="form-label">
        <span class="label-icon">📝</span>
        记录闭环
      </label>
      <div class="input-row checkbox-row">
        <input
          v-model.number="closeRecord"
          type="checkbox"
          :true-value="1"
          :false-value="0"
          class="form-checkbox"
        />
        <span class="input-unit">已完成今日记录闭环</span>
      </div>
      <span class="input-hint">完成得6分，未完成得0分</span>
    </div>

    <button
      class="submit-btn"
      :disabled="!canSubmit"
      @click="handleSubmit"
    >
      {{ store.isLoading ? "提交中..." : "打卡信仰" }}
    </button>

    <p v-if="store.error" class="error-msg">{{ store.error }}</p>

    <div v-if="submitted && store.todayFaith" class="faith-result">
      <div class="result-row">
        <span class="result-label">生存信仰</span>
        <span class="result-value survival">+{{ store.todayFaith.survival_faith }}</span>
      </div>
      <div class="result-row">
        <span class="result-label">精进信仰</span>
        <span class="result-value progress">+{{ store.todayFaith.progress_faith }}</span>
      </div>
      <div class="result-row">
        <span class="result-label">戒律信仰</span>
        <span class="result-value discipline">+{{ store.todayFaith.discipline_faith }}</span>
      </div>
      <div class="result-divider"></div>
      <div class="result-row result-total">
        <span class="result-label">今日信仰</span>
        <span class="result-value total">+{{ store.todayFaith.total_faith }}</span>
      </div>
    </div>
  </section>
</template>

<style scoped>
.checkin-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 16px;
  padding: 24px;
}

.card-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 20px;
}

.form-group {
  margin-bottom: 20px;
}

.form-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.875rem;
  color: var(--color-text-muted);
  margin-bottom: 8px;
}

.label-icon {
  font-size: 1rem;
}

.input-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.form-input {
  flex: 1;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 12px 16px;
  font-size: 1.25rem;
  color: var(--color-text);
  width: 100%;
  transition: border-color 0.2s;
}

.form-input:focus {
  border-color: var(--color-primary);
}

.form-input::-webkit-inner-spin-button,
.form-input::-webkit-outer-spin-button {
  opacity: 1;
}

.input-unit {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  white-space: nowrap;
}

.input-hint {
  display: block;
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin-top: 4px;
  text-align: right;
}

.checkbox-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.form-checkbox {
  width: 20px;
  height: 20px;
  accent-color: var(--color-discipline);
}

.submit-btn {
  width: 100%;
  padding: 14px;
  background: var(--color-primary);
  color: #1a1a24;
  font-size: 1rem;
  font-weight: 700;
  border-radius: 10px;
  letter-spacing: 2px;
  transition: background 0.2s, transform 0.1s;
}

.submit-btn:hover:not(:disabled) {
  background: var(--color-primary-dark);
}

.submit-btn:active:not(:disabled) {
  transform: scale(0.98);
}

.submit-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.error-msg {
  margin-top: 12px;
  font-size: 0.8125rem;
  color: #e06040;
  text-align: center;
}

.faith-result {
  margin-top: 20px;
  padding: 16px;
  background: var(--color-bg);
  border-radius: 12px;
}

.result-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
}

.result-label {
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.result-value {
  font-size: 1rem;
  font-weight: 700;
}

.result-value.survival { color: var(--color-survival); }
.result-value.progress { color: var(--color-progress); }
.result-value.discipline { color: var(--color-discipline); }
.result-value.total { color: var(--color-primary); font-size: 1.25rem; }

.result-divider {
  height: 1px;
  background: var(--color-border);
  margin: 8px 0;
}

.result-total {
  padding-top: 8px;
}
</style>
