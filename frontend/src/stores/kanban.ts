// frontend/src/stores/kanban.ts
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { KanbanColumn, KanbanCard } from '@/types/kanban';
import { kanbanApi } from '@/services/kanban-api';

export const useKanbanStore = defineStore('kanban', () => {
  const columns = ref<KanbanColumn[]>([]);
  const cards = ref<Map<string, KanbanCard>>(new Map());
  const activeTimers = ref<Map<string, number>>(new Map());
  const timerIntervals = ref<Map<string, number>>(new Map());
  const isLoading = ref(false);

  const sortedColumns = computed(() => {
    return [...columns.value].sort((a, b) => a.order - b.order);
  });

  async function loadBoardConfig() {
    isLoading.value = true;
    try {
      const config = await kanbanApi.getBoardConfig();
      columns.value = config.columns;
    } finally {
      isLoading.value = false;
    }
  }

  async function saveBoardConfig() {
    const config = { columns: columns.value };
    await kanbanApi.saveBoardConfig(config);
  }

  async function moveCard(taskId: string, fromColumnId: string, toColumnId: string, newOrder: number) {
    columns.value = await kanbanApi.moveCard(taskId, fromColumnId, toColumnId, newOrder, columns.value);
    await saveBoardConfig();
  }

  function startTimer(taskId: string) {
    if (activeTimers.value.has(taskId)) return;
    
    activeTimers.value.set(taskId, Date.now());
    
    // 每秒更新一次
    const intervalId = window.setInterval(() => {
      activeTimers.value = new Map(activeTimers.value);
    }, 1000);
    
    timerIntervals.value.set(taskId, intervalId);
  }

  function stopTimer(taskId: string): number {
    const startTime = activeTimers.value.get(taskId);
    const intervalId = timerIntervals.value.get(taskId);
    
    if (intervalId) {
      clearInterval(intervalId);
      timerIntervals.value.delete(taskId);
    }
    
    if (startTime) {
      const elapsed = Date.now() - startTime;
      activeTimers.value.delete(taskId);
      return elapsed;
    }
    return 0;
  }

  function getElapsedTime(taskId: string): number {
    const startTime = activeTimers.value.get(taskId);
    if (startTime) {
      return Date.now() - startTime;
    }
    return 0;
  }

  function addColumn(title: string) {
    const newColumn: KanbanColumn = {
      id: `col-${Date.now()}`,
      title,
      order: columns.value.length,
      taskIds: [],
      isCustom: true,
      createdAt: new Date().toISOString()
    };
    columns.value.push(newColumn);
    saveBoardConfig();
  }

  function removeColumn(columnId: string) {
    const col = columns.value.find(c => c.id === columnId);
    if (!col || !col.isCustom) return;
    
    columns.value = columns.value.filter(c => c.id !== columnId);
    saveBoardConfig();
  }

  return {
    columns,
    cards,
    activeTimers,
    isLoading,
    sortedColumns,
    loadBoardConfig,
    saveBoardConfig,
    moveCard,
    startTimer,
    stopTimer,
    getElapsedTime,
    addColumn,
    removeColumn,
  };
});
