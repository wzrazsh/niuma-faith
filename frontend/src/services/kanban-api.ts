// frontend/src/services/kanban-api.ts
import type { KanbanColumn, BoardConfig, KanbanCard, ProcessBinding } from '@/types/kanban';

const STORAGE_KEY = 'kanban-board-config';

export const kanbanApi = {
  async getBoardConfig(): Promise<BoardConfig> {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        return JSON.parse(stored) as BoardConfig;
      } catch {
        console.warn('Failed to parse board config, using defaults');
      }
    }
    // 返回默认配置
    return {
      columns: [
        { id: 'todo', title: '待办', order: 0, taskIds: [], isCustom: false, createdAt: new Date().toISOString() },
        { id: 'doing', title: '进行中', order: 1, taskIds: [], isCustom: false, createdAt: new Date().toISOString() },
        { id: 'paused', title: '暂停中', order: 2, taskIds: [], isCustom: false, createdAt: new Date().toISOString() },
        { id: 'done', title: '已完成', order: 3, taskIds: [], isCustom: false, createdAt: new Date().toISOString() },
      ]
    };
  },

  async saveBoardConfig(config: BoardConfig): Promise<void> {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
  },

  async moveCard(
    taskId: string,
    fromColumnId: string,
    toColumnId: string,
    newOrder: number,
    columns: KanbanColumn[]
  ): Promise<KanbanColumn[]> {
    const updatedColumns = columns.map(col => {
      if (col.id === fromColumnId) {
        return { ...col, taskIds: col.taskIds.filter(id => id !== taskId) };
      }
      if (col.id === toColumnId) {
        const newTaskIds = [...col.taskIds];
        newTaskIds.splice(newOrder, 0, taskId);
        return { ...col, taskIds: newTaskIds };
      }
      return col;
    });
    return updatedColumns;
  },

  async bindProcess(taskId: string, binding: ProcessBinding): Promise<void> {
    const key = `kanban-card-${taskId}`;
    const stored = localStorage.getItem(key);
    const card: KanbanCard = stored ? JSON.parse(stored) : { task: { id: taskId } as any, columnId: '', orderInColumn: 0 };
    card.processBinding = binding;
    localStorage.setItem(key, JSON.stringify(card));
  },

  async unbindProcess(taskId: string): Promise<void> {
    const key = `kanban-card-${taskId}`;
    const stored = localStorage.getItem(key);
    if (stored) {
      const card: KanbanCard = JSON.parse(stored);
      delete card.processBinding;
      localStorage.setItem(key, JSON.stringify(card));
    }
  }
};
