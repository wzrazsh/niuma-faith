<template>
  <div class="calendar">
    <div class="cal-header">
      <button class="cal-nav" @click="prevMonth">◀</button>
      <span class="cal-title">{{ year }}年 {{ month + 1 }}月</span>
      <button class="cal-nav" @click="nextMonth">▶</button>
    </div>
    <div class="cal-weekdays">
      <span v-for="d in weekDays" :key="d">{{ d }}</span>
    </div>
    <div class="cal-grid">
      <div v-for="(day, i) in calendarDays" :key="i"
        class="cal-day"
        :class="{ today: day.isToday, selected: day.date === selectedDate, other: !day.currentMonth }"
        @click="selectDay(day)">
        <span class="cal-day-num">{{ day.day }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

const emit = defineEmits<{ select: [date: string] }>();

const weekDays = ['日', '一', '二', '三', '四', '五', '六'];
const currentMonth = ref(new Date());
const selectedDate = ref<string>(new Date().toISOString().slice(0, 10));

const year = computed(() => currentMonth.value.getFullYear());
const month = computed(() => currentMonth.value.getMonth());

const calendarDays = computed(() => {
  const y = year.value, m = month.value;
  const firstDay = new Date(y, m, 1).getDay();
  const daysInMonth = new Date(y, m + 1, 0).getDate();
  const daysInPrev = new Date(y, m, 0).getDate();
  const today = new Date().toISOString().slice(0, 10);
  const result: any[] = [];
  for (let i = firstDay - 1; i >= 0; i--) {
    const d = daysInPrev - i;
    const date = `${y}-${String(m).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
    result.push({ day: d, date, currentMonth: false, isToday: date === today });
  }
  for (let d = 1; d <= daysInMonth; d++) {
    const date = `${y}-${String(m + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
    result.push({ day: d, date, currentMonth: true, isToday: date === today });
  }
  const remaining = 42 - result.length;
  for (let d = 1; d <= remaining; d++) {
    const nm = m + 2 > 12 ? 1 : m + 2;
    const ny = m + 2 > 12 ? y + 1 : y;
    const date = `${ny}-${String(nm).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
    result.push({ day: d, date, currentMonth: false, isToday: false });
  }
  return result;
});

function prevMonth() {
  currentMonth.value = new Date(year.value, month.value - 1, 1);
}

function nextMonth() {
  currentMonth.value = new Date(year.value, month.value + 1, 1);
}

function selectDay(day: any) {
  selectedDate.value = day.date;
  emit('select', day.date);
}
</script>

<style scoped>
.calendar {
  background: var(--color-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  padding: 14px;
}

.cal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.cal-nav {
  width: 28px;
  height: 28px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.7rem;
  color: var(--color-text-muted);
  background: transparent;
  border-radius: 50%;
}

.cal-nav:hover {
  background: var(--color-surface-hover);
  color: var(--color-text);
}

.cal-title {
  font-family: var(--font-display);
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-text);
}

.cal-weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  text-align: center;
  color: var(--color-text-dim);
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.02em;
  margin-bottom: 6px;
  text-transform: uppercase;
}

.cal-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
}

.cal-day {
  text-align: center;
  padding: 4px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
}

.cal-day-num {
  font-size: 0.82rem;
  font-weight: 500;
  display: block;
  line-height: 1.8;
}

.cal-day:hover {
  background: var(--color-surface-hover);
}

.cal-day:hover .cal-day-num {
  color: var(--color-primary);
}

.cal-day.other {
  color: var(--color-text-dim);
  opacity: 0.4;
}

.cal-day.today .cal-day-num {
  color: var(--color-primary);
  font-weight: 700;
}

.cal-day.today {
  border: 1px solid var(--color-primary-dim);
}

.cal-day.selected {
  background: linear-gradient(135deg, var(--color-primary-glow), rgba(255, 215, 0, 0.05));
  border: 1px solid var(--color-primary);
}

.cal-day.selected .cal-day-num {
  color: var(--color-primary);
  font-weight: 700;
}
</style>
