import type { BoardConfig } from '@/types/kanban';

const STORAGE_KEY = 'kanban-board-config';

export function getBoardConfig(): BoardConfig {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (raw) return JSON.parse(raw);
  return {
    columns: [
      { id: 'todo', title: '待办', order: 0, taskIds: [], isCustom: false },
      { id: 'inprogress', title: '进行中', order: 1, taskIds: [], isCustom: false },
      { id: 'paused', title: '暂停中', order: 2, taskIds: [], isCustom: false },
      { id: 'done', title: '已完成', order: 3, taskIds: [], isCustom: false },
    ],
  };
}

export function saveBoardConfig(config: BoardConfig) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
}
