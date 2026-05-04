export interface KanbanColumn {
  id: string;
  title: string;
  order: number;
  taskIds: string[];
  isCustom: boolean;
}

export interface ProcessBinding {
  appName: string;
  autoStart: boolean;
  autoPause: boolean;
}

export interface KanbanCard {
  taskId: string;
  columnId: string;
  orderInColumn: number;
  processBinding?: ProcessBinding;
  reminder?: {
    time: string;
    enabled: boolean;
  };
}

export interface BoardConfig {
  columns: KanbanColumn[];
}
