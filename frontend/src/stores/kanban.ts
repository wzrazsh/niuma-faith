import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { KanbanColumn, KanbanCard, BoardConfig, SwimlaneGroup } from '@/types/kanban';
import type { Task } from '@/types';
import { useTaskStore } from './task';

const DEFAULT_COLUMNS: KanbanColumn[] = [
  { id: 'todo', title: '待办', order: 0, taskIds: [], isCustom: false },
  { id: 'inprogress', title: '进行中', order: 1, taskIds: [], isCustom: false },
  { id: 'paused', title: '暂停中', order: 2, taskIds: [], isCustom: false },
  { id: 'done', title: '已完成', order: 3, taskIds: [], isCustom: false },
];

const SWIMLANE_CATEGORIES: { id: string; label: string }[] = [
  { id: 'work', label: '工作' },
  { id: 'study', label: '学习' },
  { id: 'other', label: '其他' },
];

function loadConfig(): BoardConfig {
  const raw = localStorage.getItem('kanban-board-config');
  if (raw) return JSON.parse(raw);
  return { columns: DEFAULT_COLUMNS };
}

function saveConfig(config: BoardConfig) {
  localStorage.setItem('kanban-board-config', JSON.stringify(config));
}

export const useKanbanStore = defineStore('kanban', () => {
  const columns = ref<KanbanColumn[]>([]);
  const cards = ref<Map<string, KanbanCard>>(new Map());
  const activeTimers = ref<Map<string, number>>(new Map());
  const isLoading = ref(false);

  const taskMap = computed(() => {
    const taskStore = useTaskStore();
    const map: Record<string, Task> = {};
    for (const t of taskStore.tasks) {
      map[t.id] = t;
    }
    return map;
  });

  function mapStatusToColumn(status: string): string {
    switch (status) {
      case 'running': return 'inprogress';
      case 'paused': return 'paused';
      case 'completed':
      case 'abandoned': return 'done';
      default: return 'todo';
    }
  }

  async function loadBoard() {
    isLoading.value = true;
    try {
      const config = loadConfig();
      columns.value = config.columns;
      const taskStore = useTaskStore();
      await taskStore.loadTasksByDate(new Date().toISOString().slice(0, 10));

      const existingTaskIds = new Set<string>();
      for (const col of columns.value) {
        for (const id of col.taskIds) {
          existingTaskIds.add(id);
        }
      }

      cards.value.clear();
      for (const col of columns.value) {
        col.taskIds = col.taskIds.filter(id => taskStore.tasks.some(t => t.id === id));
        for (let i = 0; i < col.taskIds.length; i++) {
          const id = col.taskIds[i];
          cards.value.set(id, { taskId: id, columnId: col.id, orderInColumn: i });
        }
      }

      for (const task of taskStore.tasks) {
        if (existingTaskIds.has(task.id)) continue;
        const colId = mapStatusToColumn(task.status);
        const col = columns.value.find(c => c.id === colId);
        if (col) {
          col.taskIds.push(task.id);
          cards.value.set(task.id, { taskId: task.id, columnId: colId, orderInColumn: col.taskIds.length - 1 });
        }
      }
      saveConfig({ columns: columns.value });
    } catch (e: any) {
      console.error('[kanban] loadBoard failed:', e);
    }
    isLoading.value = false;
  }

  function columnCards(columnId: string): KanbanCard[] {
    const col = columns.value.find(c => c.id === columnId);
    if (!col) return [];
    return col.taskIds.map(id => cards.value.get(id)).filter(Boolean) as KanbanCard[];
  }

  function columnSwimlanes(columnId: string): SwimlaneGroup[] {
    const tasks = columnCards(columnId);
    const taskStore = useTaskStore();
    const groups: SwimlaneGroup[] = SWIMLANE_CATEGORIES.map(cat => ({
      categoryId: cat.id,
      label: cat.label,
      cards: [],
    }));
    for (const card of tasks) {
      const task = taskStore.tasks.find(t => t.id === card.taskId);
      if (!task) continue;
      const group = groups.find(g => g.categoryId === task.category);
      if (group) group.cards.push(card);
    }
    return groups.filter(g => g.cards.length > 0);
  }

  function moveCard(cardId: string, targetColumnId: string, targetIndex: number) {
    const card = cards.value.get(cardId);
    if (!card) return;
    const sourceCol = columns.value.find(c => c.id === card.columnId);
    const targetCol = columns.value.find(c => c.id === targetColumnId);
    if (!sourceCol || !targetCol) return;
    sourceCol.taskIds = sourceCol.taskIds.filter(id => id !== cardId);
    targetCol.taskIds.splice(targetIndex, 0, cardId);
    card.columnId = targetColumnId;
    cards.value.set(cardId, card);
    saveConfig({ columns: columns.value });
  }

  function addCard(columnId: string, taskId: string) {
    const col = columns.value.find(c => c.id === columnId);
    if (!col) return;
    if (col.taskIds.includes(taskId)) return;
    col.taskIds.push(taskId);
    cards.value.set(taskId, { taskId, columnId, orderInColumn: col.taskIds.length - 1 });
    saveConfig({ columns: columns.value });
  }

  function startTimer(cardId: string) {
    const existing = activeTimers.value.get(cardId);
    if (existing) window.clearInterval(existing);
    const interval = window.setInterval(() => {}, 1000);
    activeTimers.value.set(cardId, interval);
  }

  function stopTimer(cardId: string) {
    const interval = activeTimers.value.get(cardId);
    if (interval) { window.clearInterval(interval); activeTimers.value.delete(cardId); }
  }

  function addColumn(title: string) {
    const col: KanbanColumn = {
      id: `col-${Date.now()}`,
      title,
      order: columns.value.length,
      taskIds: [],
      isCustom: true,
    };
    columns.value.push(col);
    saveConfig({ columns: columns.value });
  }

  function removeColumn(id: string) {
    const col = columns.value.find(c => c.id === id);
    if (col?.isCustom) {
      columns.value = columns.value.filter(c => c.id !== id);
      saveConfig({ columns: columns.value });
    }
  }

  function resetToDefault() {
    cards.value.clear();
    activeTimers.value.forEach((_, key) => stopTimer(key));
    columns.value = DEFAULT_COLUMNS.map(c => ({ ...c, taskIds: [...c.taskIds] }));
    saveConfig({ columns: columns.value });
    loadBoard();
  }

  return { columns, cards, activeTimers, isLoading, taskMap, columnCards, columnSwimlanes, loadBoard, moveCard, addCard, startTimer, stopTimer, addColumn, removeColumn, resetToDefault };
});
