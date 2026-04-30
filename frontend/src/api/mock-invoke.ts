/**
 * Browser-mode mock for @tauri-apps/api/core invoke.
 * When running in `npm run dev` (not inside Tauri), uses localStorage with
 * the same interface as the Tauri backend commands.
 */

const STORAGE_TASKS = 'mock-tasks';
const STORAGE_FAITH = 'mock-faith';

interface MockTask {
  id: string;
  user_id: string;
  title: string;
  description: string;
  category: string;
  estimated_minutes: number;
  actual_minutes: number;
  notes: string;
  date: string;
  status: string;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
}

interface MockFaithRecord {
  date: string;
  check_in_done: boolean;
  survial: number;
  progress: number;
  discipline: number;
  total: number;
  level: number;
  title: string;
}

let idCounter = Date.now();
function genId(): string {
  return `mock-${(idCounter++).toString(36)}`;
}

function todayStr(): string {
  return new Date().toLocaleDateString('en-CA');
}

function loadTasks(): MockTask[] {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_TASKS) || '[]');
  } catch {
    return [];
  }
}

function saveTasks(tasks: MockTask[]): void {
  localStorage.setItem(STORAGE_TASKS, JSON.stringify(tasks));
}

function loadFaith(): MockFaithRecord[] {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_FAITH) || '[]');
  } catch {
    return [];
  }
}

function saveFaith(records: MockFaithRecord[]): void {
  localStorage.setItem(STORAGE_FAITH, JSON.stringify(records));
}

const handlers: Record<string, Function> = {
  create_task(args: any): MockTask {
    const tasks = loadTasks();
    const now = new Date().toISOString();
    const task: MockTask = {
      id: genId(),
      user_id: args.userId,
      title: args.title,
      description: args.description ?? '',
      category: args.category ?? 'work',
      estimated_minutes: args.estimatedMinutes ?? 60,
      actual_minutes: 0,
      notes: '',
      date: args.date ?? todayStr(),
      status: 'pending',
      created_at: now,
      updated_at: now,
      completed_at: null,
    };
    tasks.push(task);
    saveTasks(tasks);
    return task;
  },

  get_tasks(args: any): MockTask[] {
    const tasks = loadTasks();
    if (args.status) {
      return tasks.filter(t => t.user_id === args.userId && t.status === args.status);
    }
    return tasks.filter(t => t.user_id === args.userId);
  },

  get_tasks_by_date(args: any): MockTask[] {
    const tasks = loadTasks();
    return tasks.filter(t => {
      if (t.user_id !== args.userId) return false;
      if (t.date !== args.date) return false;
      if (args.status && t.status !== args.status) return false;
      return true;
    });
  },

  get_task(args: any): MockTask | null {
    return loadTasks().find(t => t.id === args.id) ?? null;
  },

  update_task(args: any): MockTask {
    const tasks = loadTasks();
    const idx = tasks.findIndex(t => t.id === args.id);
    if (idx === -1) throw new Error(`Task not found: ${args.id}`);
    const t = tasks[idx];
    if (args.title !== undefined && args.title !== null) t.title = args.title;
    if (args.description !== undefined && args.description !== null) t.description = args.description;
    if (args.estimatedMinutes !== undefined && args.estimatedMinutes !== null) t.estimated_minutes = args.estimatedMinutes;
    if (args.actualMinutes !== undefined && args.actualMinutes !== null) t.actual_minutes = args.actualMinutes;
    if (args.notes !== undefined && args.notes !== null) t.notes = args.notes;
    if (args.status !== undefined && args.status !== null) t.status = args.status;
    t.updated_at = new Date().toISOString();
    tasks[idx] = t;
    saveTasks(tasks);
    return t;
  },

  complete_task(args: any): { task: MockTask; faith_update: any } {
    const tasks = loadTasks();
    const idx = tasks.findIndex(t => t.id === args.id);
    if (idx === -1) throw new Error(`Task not found: ${args.id}`);
    const t = tasks[idx];
    t.status = 'completed';
    t.actual_minutes = args.actualMinutes ?? t.actual_minutes;
    t.updated_at = new Date().toISOString();
    t.completed_at = new Date().toISOString();
    tasks[idx] = t;
    saveTasks(tasks);
    return { task: t, faith_update: {} };
  },

  abandon_task(args: any): MockTask {
    const tasks = loadTasks();
    const idx = tasks.findIndex(t => t.id === args.id);
    if (idx === -1) throw new Error(`Task not found: ${args.id}`);
    const t = tasks[idx];
    t.status = 'abandoned';
    t.updated_at = new Date().toISOString();
    tasks[idx] = t;
    saveTasks(tasks);
    return t;
  },

  delete_task(args: any): boolean {
    const tasks = loadTasks();
    const idx = tasks.findIndex(t => t.id === args.id);
    if (idx === -1) return false;
    tasks.splice(idx, 1);
    saveTasks(tasks);
    return true;
  },

  get_status(args: any): any {
    const date = todayStr();
    const records = loadFaith();
    const record = records.find(r => r.date === date);
    return {
      faith: {
        survial: record?.survial ?? 0,
        progress: record?.progress ?? 0,
        discipline: record?.discipline ?? 0,
        total: record?.total ?? 0,
      },
      level: record?.level ?? 0,
      title: record?.title ?? '新手牛马',
      checked_in: record?.check_in_done ?? false,
    };
  },

  get_today_record(args: any): any | null {
    const date = todayStr();
    const records = loadFaith();
    return records.find(r => r.date === date) ?? null;
  },

  get_or_create_user(args: any): any {
    return {
      id: 'default_user',
      name: '默认用户',
      created_at: new Date().toISOString(),
    };
  },
};

/**
 * Invoke a backend command, falling back to in-browser mock when Tauri is unavailable.
 */
export async function safeInvoke<T>(command: string, args: Record<string, any> = {}): Promise<T> {
  const handler = handlers[command];
  if (!handler) {
    throw new Error(`Mock: unknown command "${command}"`);
  }
  return handler(args) as T;
}
