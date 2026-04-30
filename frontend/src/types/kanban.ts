// frontend/src/types/kanban.ts
import type { Task } from './index';

export interface KanbanColumn {
  id: string;
  title: string;
  order: number;
  taskIds: string[];
  isCustom: boolean;
  createdAt: string;
}

export interface ProcessBinding {
  appName: string;
  autoStart: boolean;
  autoPause: boolean;
}

export interface Reminder {
  time: string; // HH:mm format
  enabled: boolean;
}

export interface KanbanCard {
  task: Task;
  columnId: string;
  orderInColumn: number;
  processBinding?: ProcessBinding;
  reminder?: Reminder;
}

export interface KanbanState {
  columns: KanbanColumn[];
  cards: Map<string, KanbanCard>;
  activeTimers: Map<string, number>;
  isLoading: boolean;
}

export interface BoardConfig {
  columns: KanbanColumn[];
}
