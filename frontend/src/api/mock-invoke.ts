/**
 * Browser-mode mock for @tauri-apps/api/core invoke.
 * When running in `npm run dev` (not inside Tauri), uses localStorage with
 * the same interface as the Tauri backend commands.
 */

const STORAGE_TASKS = 'mock-tasks';
const STORAGE_FAITH = 'mock-faith';
const STORAGE_USER = 'mock-user';

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
  started_at: string | null;
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
  work_minutes: number;
  study_minutes: number;
  break_count: number;
  leave_record: number;
  close_record: number;
  discipline_a: number;
  discipline_b: number;
  discipline_c: number;
  tasks_completed: number;
  created_at: string;
  updated_at: string;
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

function loadUser(): any {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_USER) || 'null');
  } catch {
    return null;
  }
}

function saveUser(user: any): void {
  localStorage.setItem(STORAGE_USER, JSON.stringify(user));
}

const LEVEL_THRESHOLDS: number[] = [
  0, 100, 300, 600, 1000, 1500, 2100, 2800, 3600, 4500,
  5500, 6600, 7800, 9100, 10500, null as any,
];

function getNextThreshold(level: number): number | null {
  if (level >= 15 || level >= LEVEL_THRESHOLDS.length - 1) return null;
  return LEVEL_THRESHOLDS[level + 1] ?? null;
}

const LEVEL_TITLES: string[] = [
  '见习牛马', '初级牛马', '熟练牛马', '资深牛马', '精英牛马',
  '牛马专家', '牛马大师', '牛马宗师', '牛马圣手', '牛马尊者',
  '牛马王者', '牛马天尊', '牛马神话', '牛马传说', '牛马圣徒', '牛马之神',
];

function getTitleForLevel(level: number): string {
  return LEVEL_TITLES[Math.min(level, LEVEL_TITLES.length - 1)] ?? '见习牛马';
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
      status: 'paused',
      created_at: now,
      updated_at: now,
      completed_at: null,
      started_at: null,
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

  get_status(_args: any): any {
    const date = todayStr();
    const records = loadFaith();
    const record = records.find(r => r.date === date);
    const user = loadUser();
    const level = record?.level ?? 0;
    const currentFaith = user?.cumulative_faith ?? 0;
    const nextThreshold = getNextThreshold(level);
    return {
      user_id: user?.id ?? 'default_user',
      cumulative_faith: currentFaith,
      current_level: level,
      level_title: record?.title ?? getTitleForLevel(level),
      progress_to_next: nextThreshold !== null ? Math.max(0, (nextThreshold - currentFaith)) : 0,
      next_threshold: nextThreshold,
      today: record
        ? {
            id: null,
            user_id: user?.id ?? 'default_user',
            date: record.date,
            work_minutes: record.work_minutes ?? 0,
            study_minutes: record.study_minutes ?? 0,
            survival_faith: record.survial ?? 0,
            progress_faith: record.progress ?? 0,
            discipline_faith: record.discipline ?? 0,
            total_faith: record.total ?? 0,
            break_count: record.break_count ?? 0,
            leave_record: record.leave_record ?? 0,
            close_record: record.close_record ?? 0,
            discipline_a: 0,
            discipline_b: 0,
            discipline_c: 0,
            tasks_completed: record.tasks_completed ?? 0,
            created_at: record.created_at ?? date,
            updated_at: record.updated_at ?? date,
          }
        : null,
      armor: user?.armor ?? 0,
      total_armor: user?.total_armor ?? 100,
    };
  },

  get_today_record(_args: any): any | null {
    const date = todayStr();
    const records = loadFaith();
    const record = records.find(r => r.date === date);
    if (!record) return null;
    return {
      id: null,
      user_id: 'default_user',
      date: record.date,
      work_minutes: record.work_minutes ?? 0,
      study_minutes: record.study_minutes ?? 0,
      survival_faith: record.survial ?? 0,
      progress_faith: record.progress ?? 0,
      discipline_faith: record.discipline ?? 0,
      total_faith: record.total ?? 0,
      break_count: record.break_count ?? 0,
      leave_record: record.leave_record ?? 0,
      close_record: record.close_record ?? 0,
      discipline_a: 0,
      discipline_b: 0,
      discipline_c: 0,
      tasks_completed: record.tasks_completed ?? 0,
      created_at: record.created_at ?? date,
      updated_at: record.updated_at ?? date,
    };
  },

  get_or_create_user(_args: any): any {
    let user = loadUser();
    if (!user) {
      user = {
        id: 'default_user',
        name: '默认用户',
        cumulative_faith: 0,
        current_level: 0,
        armor: 0,
        total_armor: 100,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      saveUser(user);
    }
    return user;
  },

  start_task(args: any): MockTask | null {
    const tasks = loadTasks();
    const idx = tasks.findIndex(t => t.id === args.id);
    if (idx === -1) return null;
    tasks[idx].status = 'running';
    tasks[idx].started_at = new Date().toISOString();
    tasks[idx].updated_at = new Date().toISOString();
    saveTasks(tasks);
    return tasks[idx];
  },

  pause_task(args: any): MockTask | null {
    const tasks = loadTasks();
    const idx = tasks.findIndex(t => t.id === args.id);
    if (idx === -1) return null;
    const t = tasks[idx];
    if (t.started_at) {
      const started = new Date(t.started_at).getTime();
      const now = Date.now();
      t.actual_minutes += Math.round((now - started) / 60000);
    }
    t.status = 'paused';
    t.updated_at = new Date().toISOString();
    tasks[idx] = t;
    saveTasks(tasks);
    return t;
  },

  resume_task(args: any): MockTask | null {
    const tasks = loadTasks();
    const idx = tasks.findIndex(t => t.id === args.id);
    if (idx === -1) return null;
    tasks[idx].status = 'running';
    tasks[idx].started_at = new Date().toISOString();
    tasks[idx].updated_at = new Date().toISOString();
    saveTasks(tasks);
    return tasks[idx];
  },

  end_task(args: any): { task: MockTask; faith_update: any } {
    const tasks = loadTasks();
    const idx = tasks.findIndex(t => t.id === args.id);
    if (idx === -1) throw new Error(`Task not found: ${args.id}`);
    const t = tasks[idx];
    if (t.started_at) {
      const started = new Date(t.started_at).getTime();
      const now = Date.now();
      t.actual_minutes += Math.round((now - started) / 60000);
    }
    t.status = 'completed';
    t.updated_at = new Date().toISOString();
    t.completed_at = new Date().toISOString();
    tasks[idx] = t;
    saveTasks(tasks);
    return { task: t, faith_update: {} };
  },

  check_in(_args: any): any {
    const date = todayStr();
    const records = loadFaith();
    let record: MockFaithRecord | undefined = records.find(r => r.date === date);
    if (!record) {
      record = {
        date,
        check_in_done: true,
        survial: 0,
        progress: 0,
        discipline: 0,
        total: 0,
        level: 0,
        title: '见习牛马',
        work_minutes: 0,
        study_minutes: 0,
        break_count: 0,
        leave_record: 0,
        close_record: 0,
        discipline_a: 0,
        discipline_b: 0,
        discipline_c: 0,
        tasks_completed: 0,
        created_at: date,
        updated_at: date,
      };
      records.push(record);
      saveFaith(records);
    } else {
      record.check_in_done = true;
      saveFaith(records);
    }
    // After if/else, record is always defined
    const r = record!;
    const user = loadUser();
    const level = r.level;
    const currentFaith = user?.cumulative_faith ?? 0;
    const nextThreshold = getNextThreshold(level);
    return {
      user_id: user?.id ?? 'default_user',
      cumulative_faith: currentFaith,
      current_level: level,
      level_title: r.title ?? getTitleForLevel(level),
      progress_to_next: nextThreshold !== null ? Math.max(0, (nextThreshold - currentFaith)) : 0,
      next_threshold: nextThreshold,
      today: {
        id: null,
        user_id: user?.id ?? 'default_user',
        date: r.date,
        work_minutes: r.work_minutes ?? 0,
        study_minutes: r.study_minutes ?? 0,
        survival_faith: r.survial ?? 0,
        progress_faith: r.progress ?? 0,
        discipline_faith: r.discipline ?? 0,
        total_faith: r.total ?? 0,
        break_count: r.break_count ?? 0,
        leave_record: r.leave_record ?? 0,
        close_record: r.close_record ?? 0,
        discipline_a: 0,
        discipline_b: 0,
        discipline_c: 0,
        tasks_completed: r.tasks_completed ?? 0,
        created_at: r.created_at ?? date,
        updated_at: r.updated_at ?? date,
      },
      armor: user?.armor ?? 0,
      total_armor: user?.total_armor ?? 100,
    };
  },

  get_daily_stats(_args: any): any {
    const date = todayStr();
    const records = loadFaith();
    const record = records.find(r => r.date === date);
    const user = loadUser();
    return {
      date,
      work_minutes: record?.work_minutes ?? 0,
      study_minutes: record?.study_minutes ?? 0,
      survival_faith: record?.survial ?? 0,
      progress_faith: record?.progress ?? 0,
      discipline_faith: record?.discipline ?? 0,
      total_faith: record?.total ?? 0,
      task_bonus_work: 0,
      task_bonus_study: 0,
      tasks_completed: record?.tasks_completed ?? 0,
      cumulative_faith: user?.cumulative_faith ?? 0,
    };
  },

  get_task_session(_args: any): any | null {
    const tasks = loadTasks();
    const running = tasks.find(t => t.status === 'running');
    if (!running || !running.started_at) return null;
    return {
      id: 1,
      task_id: running.id,
      start_ts: running.started_at,
      end_ts: null,
      seconds: Math.round((Date.now() - new Date(running.started_at).getTime()) / 1000),
    };
  },

  is_process_running(_args: any): boolean {
    return false;
  },

  list_processes(_args: any): any[] {
    return [];
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
