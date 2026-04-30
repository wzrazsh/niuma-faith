<script setup lang="ts">
import { ref, computed } from "vue";

const emit = defineEmits<{
  "update:modelValue": [date: string];
  "update:view": [view: "month" | "week" | "day"];
}>();

defineProps<{
  modelValue: string;
  view: "month" | "week" | "day";
}>();

// Use local date to match backend chrono::Local
const todayString = new Date().toLocaleDateString('en-CA'); // YYYY-MM-DD local

// --- Month view logic ---
const currentYear = ref(new Date().getFullYear());
const currentMonth = ref(new Date().getMonth()); // 0-indexed

const dayNames = ["日", "一", "二", "三", "四", "五", "六"];

const monthLabel = computed(() => {
  return `${currentYear.value}年${currentMonth.value + 1}月`;
});

const firstDayOfMonth = computed(() => {
  return new Date(currentYear.value, currentMonth.value, 1).getDay();
});

const daysInMonth = computed(() => {
  return new Date(currentYear.value, currentMonth.value + 1, 0).getDate();
});

const calendarCells = computed(() => {
  const cells: Array<{ date: string; day: number; isCurrentMonth: boolean }> = [];
  const totalSlots = Math.ceil((firstDayOfMonth.value + daysInMonth.value) / 7) * 7;
  for (let i = 0; i < totalSlots; i++) {
    const dayOfMonth = i - firstDayOfMonth.value + 1;
    const isCurrentMonth = dayOfMonth >= 1 && dayOfMonth <= daysInMonth.value;
    if (isCurrentMonth) {
      const m = String(currentMonth.value + 1).padStart(2, "0");
      const d = String(dayOfMonth).padStart(2, "0");
      cells.push({
        date: `${currentYear.value}-${m}-${d}`,
        day: dayOfMonth,
        isCurrentMonth: true,
      });
    } else {
      cells.push({ date: "", day: 0, isCurrentMonth: false });
    }
  }
  return cells;
});

function prevMonth() {
  if (currentMonth.value === 0) {
    currentMonth.value = 11;
    currentYear.value--;
  } else {
    currentMonth.value--;
  }
}

function nextMonth() {
  if (currentMonth.value === 11) {
    currentMonth.value = 0;
    currentYear.value++;
  } else {
    currentMonth.value++;
  }
}

function selectCell(cell: { date: string; isCurrentMonth: boolean }) {
  if (!cell.isCurrentMonth || !cell.date) return;
  emit("update:modelValue", cell.date);
}

// --- Week view logic ---
const weekStart = computed(() => {
  const today = new Date(todayString);
  const day = today.getDay();
  const diff = today.getDate() - day;
  return new Date(today.setDate(diff));
});

const weekDays = computed(() => {
  const days: Array<{ date: string; dayNum: number; dayName: string }> = [];
  const start = new Date(weekStart.value);
  for (let i = 0; i < 7; i++) {
    const d = new Date(start);
    d.setDate(start.getDate() + i);
    const dateStr = d.toISOString().split("T")[0];
    days.push({
      date: dateStr,
      dayNum: d.getDate(),
      dayName: dayNames[d.getDay()],
    });
  }
  return days;
});

// --- Shared helpers ---
function isToday(date: string) {
  return date === todayString;
}

function isPast(date: string) {
  return date < todayString;
}

function goToToday() {
  const t = new Date();
  currentYear.value = t.getFullYear();
  currentMonth.value = t.getMonth();
  emit("update:modelValue", todayString);
}
</script>

<template>
  <div class="calendar-view">
    <!-- Month View -->
    <div v-if="view === 'month'" class="month-view">
      <div class="cal-header">
        <button class="nav-btn" @click="prevMonth">&lt;</button>
        <span class="month-label">{{ monthLabel }}</span>
        <button class="nav-btn" @click="nextMonth">&gt;</button>
        <button class="today-btn" @click="goToToday">今天</button>
      </div>
      <div class="day-names">
        <span v-for="d in dayNames" :key="d" class="day-name">{{ d }}</span>
      </div>
      <div class="cal-grid">
        <div
          v-for="(cell, idx) in calendarCells"
          :key="idx"
          class="cal-cell"
          :class="{
            'other-month': !cell.isCurrentMonth,
            'is-today': isToday(cell.date),
            'is-past': isPast(cell.date),
          }"
          @click="selectCell(cell)"
        >
          <span v-if="cell.isCurrentMonth" class="cell-day">{{ cell.day }}</span>
          <span v-if="cell.isCurrentMonth && isPast(cell.date)" class="past-indicator">—</span>
        </div>
      </div>
    </div>

    <!-- Week View -->
    <div v-else-if="view === 'week'" class="week-view">
      <div class="week-grid">
        <div
          v-for="d in weekDays"
          :key="d.date"
          class="week-cell"
          :class="{ 'is-today': isToday(d.date), 'is-past': isPast(d.date) }"
          @click="emit('update:modelValue', d.date)"
        >
          <div class="week-day-name">{{ d.dayName }}</div>
          <div class="week-day-num">{{ d.dayNum }}</div>
        </div>
      </div>
    </div>

    <!-- Day View -->
    <div v-else class="day-view">
      <div class="day-header">{{ modelValue }}</div>
      <p class="day-placeholder">日视图 — 详情由父组件展示</p>
    </div>
  </div>
</template>

<style scoped>
.calendar-view {
  user-select: none;
}

.month-view {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.cal-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.month-label {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--color-text);
  flex: 1;
  text-align: center;
}

.nav-btn {
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 4px 10px;
  font-size: 0.875rem;
  color: var(--color-text);
  cursor: pointer;
}

.nav-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.today-btn {
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 4px 10px;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  cursor: pointer;
}

.day-names {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
  margin-bottom: 2px;
}

.day-name {
  text-align: center;
  font-size: 0.75rem;
  color: var(--color-text-muted);
  padding: 4px 0;
}

.cal-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
}

.cal-cell {
  aspect-ratio: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  cursor: pointer;
  position: relative;
  background: var(--color-bg);
  transition: background 0.15s;
  min-height: 36px;
}

.cal-cell:hover:not(.other-month) {
  background: var(--color-surface);
  border: 1px solid var(--color-primary);
}

.cal-cell.other-month {
  cursor: default;
  opacity: 0.3;
}

.cal-cell.is-today {
  border: 2px solid var(--color-primary);
  box-shadow: 0 0 6px var(--color-primary);
}

.cal-cell.is-past {
  cursor: not-allowed;
  opacity: 0.5;
}

.cal-cell.is-past:hover {
  background: var(--color-bg);
  border-color: var(--color-border);
}

.cell-day {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text);
}

.past-indicator {
  font-size: 0.625rem;
  color: var(--color-text-muted);
}

/* Week view */
.week-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 4px;
}

.week-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 10px 4px;
  border-radius: 10px;
  background: var(--color-bg);
  cursor: pointer;
  transition: all 0.15s;
  border: 2px solid transparent;
}

.week-cell:hover {
  border-color: var(--color-primary);
}

.week-cell.is-today {
  border-color: var(--color-primary);
  box-shadow: 0 0 6px var(--color-primary);
}

.week-cell.is-past {
  opacity: 0.5;
  cursor: not-allowed;
}

.week-day-name {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.week-day-num {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
}

/* Day view */
.day-view {
  padding: 16px;
}

.day-header {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 12px;
}

.day-placeholder {
  color: var(--color-text-muted);
  font-size: 0.875rem;
  text-align: center;
  padding: 24px;
}
</style>
