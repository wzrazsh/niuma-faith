import type { DailyRecord, FaithStatus, ProcessInfo, Task, TaskCompleteResult, User } from '@/types';

const DEFAULT_USER_ID = 'default_user';

// ============= Level thresholds (v2.0 ×10) =============
const LEVEL_THRESHOLDS = [0, 15000, 40000, 80000, 135000, 205000, 290000, 395000, 520000, 665000, 825000, 945000, 1025000, 1070000, 1095000];
const LEVEL_TITLES = ['', '见习牛马', '工位信徒', '初级供奉者', '稳定产出者', '自律门徒', '双修学徒', '工时祭司', '苦修执行官', '连轴修行者', '钢铁牛马', '卷力使徒', '精进主教', '福报传道者', '31日苦修士', '牛马圣徒'];

function getLevel(cumulative: number): { level: number; title: string } {
  for (let i = LEVEL_THRESHOLDS.length - 1; i >= 0; i--) {
    if (cumulative >= LEVEL_THRESHOLDS[i]) return { level: i + 1, title: LEVEL_TITLES[i + 1] || '' };
  }
  return { level: 1, title: '见习牛马' };
}

function calcArmor(level: number): number {
  if (level <= 1) return 0;
  if (level <= 5) return 2000;
  if (level <= 10) return 4000;
  return 6000;
}

function todayStr(): string {
  return new Date().toISOString().slice(0, 10);
}

function nowStr(): string {
  return new Date().toISOString();
}

// localStorage keys
const STORAGE_USER = 'mock-user';
const STORAGE_FAITH = 'mock-faith';
const STORAGE_TASKS = 'mock-tasks';

function loadUser(): User {
  const data = localStorage.getItem(STORAGE_USER);
  if (data) return JSON.parse(data);
  const user: User = {
    id: DEFAULT_USER_ID, nickname: '牛马信徒', cumulative_faith: 0,
    current_level: 1, armor_points: 0, created_at: nowStr(), updated_at: nowStr(),
  };
  saveUser(user);
  return user;
}

function saveUser(user: User) {
  localStorage.setItem(STORAGE_USER, JSON.stringify(user));
}

function loadFaithRecords(): Record<string, DailyRecord> {
  const data = localStorage.getItem(STORAGE_FAITH);
  return data ? JSON.parse(data) : {};
}

function saveFaithRecords(records: Record<string, DailyRecord>) {
  localStorage.setItem(STORAGE_FAITH, JSON.stringify(records));
}

function loadTasks(): Task[] {
  const data = localStorage.getItem(STORAGE_TASKS);
  return data ? JSON.parse(data) : [];
}

function saveTasks(tasks: Task[]) {
  localStorage.setItem(STORAGE_TASKS, JSON.stringify(tasks));
}

function calcSurvivalFaith(minutes: number): number {
  const hours = Math.floor(minutes / 60);
  if (hours >= 8) return 400;
  if (hours >= 6) return 300;
  if (hours >= 4) return 200;
  if (hours >= 2) return 100;
  return 0;
}

function calcProgressFaith(minutes: number): number {
  return calcSurvivalFaith(minutes);
}

function calcDiscipline(breakCount: number, leaveRecord: number, closeRecord: number): [number, number, number, number] {
  let a = 0;
  if (breakCount <= 2) a = 80; else if (breakCount <= 4) a = 40;
  let b = 0;
  if (leaveRecord === 0) b = 60; else if (leaveRecord === 1) b = 30;
  const c = closeRecord >= 1 ? 60 : 0;
  return [a + b + c, a, b, c];
}

function calcTaskBonus(category: string, actualMinutes: number): number {
  const hours = Math.max(1, Math.floor(actualMinutes / 60));
  return (category === 'work' || category === 'study') ? hours * 5 : hours * 2;
}

interface MockHandlers {
  [key: string]: (args: any) => Promise<any>;
}

export const handlers: MockHandlers = {
  async get_or_create_user() {
    const user = loadUser();
    return { ...user, armor: user.armor_points, total_armor: calcArmor(user.current_level) };
  },

  async get_status(args: { userId: string }) {
    const user = loadUser();
    const faithRecords = loadFaithRecords();
    const todayKey = `${args.userId}_${todayStr()}`;
    const todayRecord = faithRecords[todayKey] || null;
    const levelInfo = getLevel(user.cumulative_faith);
    const nextIdx = levelInfo.level;
    const progress = nextIdx < LEVEL_THRESHOLDS.length ? LEVEL_THRESHOLDS[nextIdx] - user.cumulative_faith : null;
    const nextThreshold = nextIdx < LEVEL_THRESHOLDS.length ? LEVEL_THRESHOLDS[nextIdx] : null;
    return {
      user_id: user.id,
      cumulative_faith: user.cumulative_faith,
      current_level: user.current_level,
      level_title: levelInfo.title,
      progress_to_next: progress,
      next_threshold: nextThreshold,
      today: todayRecord ? { ...todayRecord, id: todayRecord.id || null } : null,
      armor: user.armor_points,
      total_armor: calcArmor(user.current_level),
    } as FaithStatus;
  },

  async check_in(args: { userId: string; workMinutes: number; studyMinutes: number; breakCount: number; leaveRecord: number; closeRecord: number }) {
    const user = loadUser();
    const records = loadFaithRecords();
    const date = todayStr();
    const key = `${args.userId}_${date}`;
    const oldRecord = records[key];
    const oldTotal = oldRecord ? oldRecord.total_faith : 0;

    const survival = calcSurvivalFaith(args.workMinutes);
    const progress = calcProgressFaith(args.studyMinutes);
    const [discipline, a, b, c] = calcDiscipline(args.breakCount, args.leaveRecord, args.closeRecord);
    const taskBonusWork = oldRecord ? oldRecord.task_bonus_work : 0;
    const taskBonusStudy = oldRecord ? oldRecord.task_bonus_study : 0;
    const tasksCompleted = oldRecord ? oldRecord.tasks_completed : 0;
    const total = survival + progress + discipline + taskBonusWork + taskBonusStudy;

    const record: DailyRecord = {
      id: oldRecord?.id ?? null, user_id: args.userId, date,
      work_minutes: args.workMinutes, study_minutes: args.studyMinutes,
      survival_faith: survival, progress_faith: progress, discipline_faith: discipline,
      total_faith: total, task_bonus_work: taskBonusWork, task_bonus_study: taskBonusStudy,
      break_count: args.breakCount, leave_record: args.leaveRecord, close_record: args.closeRecord,
      discipline_a: a, discipline_b: b, discipline_c: c,
      tasks_completed: tasksCompleted, created_at: nowStr(), updated_at: nowStr(),
    };
    records[key] = record;
    saveFaithRecords(records);

    const delta = total - oldTotal;
    user.cumulative_faith += delta;
    const levelInfo = getLevel(user.cumulative_faith);
    user.current_level = levelInfo.level;
    user.armor_points = calcArmor(user.current_level);
    user.updated_at = nowStr();
    saveUser(user);

    const nextIdx = levelInfo.level;
    const nextThreshold = nextIdx < LEVEL_THRESHOLDS.length ? LEVEL_THRESHOLDS[nextIdx] : null;
    const nextProgress = nextIdx < LEVEL_THRESHOLDS.length ? LEVEL_THRESHOLDS[nextIdx] - user.cumulative_faith : null;

    return {
      user_id: user.id, cumulative_faith: user.cumulative_faith,
      current_level: user.current_level, level_title: levelInfo.title,
      progress_to_next: nextProgress, next_threshold: nextThreshold,
      today: record,
      armor: user.armor_points, total_armor: calcArmor(user.current_level),
    } as FaithStatus;
  },

  async get_today_record(args: { userId: string }) {
    const records = loadFaithRecords();
    const key = `${args.userId}_${todayStr()}`;
    return records[key] || null;
  },

  async is_process_running() { return false; },
  async list_processes() { return [] as ProcessInfo[]; },

  async create_task(args: any) {
    if (args.estimatedMinutes <= 0) throw 'estimated_minutes must be > 0';
    const tasks = loadTasks();
    const now = nowStr();
    const date = args.date || todayStr();
    const rec = args.recurrenceKind || 'none';
    const task: Task = {
      id: `task-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      user_id: args.userId || DEFAULT_USER_ID,
      date,
      title: args.title,
      description: args.description || '',
      category: args.category || 'work',
      estimated_minutes: args.estimatedMinutes,
      actual_minutes: 0,
      status: 'paused',
      notes: '',
      created_at: now,
      started_at: null,
      completed_at: null,
      duration_seconds: 0,
      ai_summary: null,
      updated_at: now,
      recurrence_kind: rec as any,
      template_id: null,
      task_type: 'daily',
      source_tool: null,
      tool_session_id: null,
    };
    tasks.push(task);
    saveTasks(tasks);
    return task;
  },

  async get_tasks_by_date(args: { userId: string; date: string; status: string | null }) {
    let tasks = loadTasks().filter(t => t.user_id === args.userId && t.date === args.date);
    if (args.status) tasks = tasks.filter(t => t.status === args.status);
    // Synthesize virtual instances from daily templates
    const today = todayStr();
    if (args.date >= today && (!args.status || args.status === 'paused')) {
      const templates = loadTasks().filter(t => t.user_id === args.userId && t.recurrence_kind === 'daily' && !t.template_id);
      for (const tpl of templates) {
        if (tpl.date === args.date) continue;
        const exists = tasks.some(t => t.template_id === tpl.id);
        if (exists) continue;
        tasks.push({
          id: `daily:${tpl.id}:${args.date}`,
          user_id: tpl.user_id, date: args.date, title: tpl.title, description: tpl.description,
          category: tpl.category, estimated_minutes: tpl.estimated_minutes, actual_minutes: 0,
          status: 'paused', notes: '', created_at: tpl.created_at, started_at: null,
          completed_at: null, duration_seconds: 0, ai_summary: null, updated_at: tpl.updated_at,
          recurrence_kind: 'none', template_id: tpl.id, task_type: 'daily',
          source_tool: null, tool_session_id: null,
        });
      }
    }
    return tasks;
  },

  async get_tasks(args: { userId: string; status: string | null }) {
    let tasks = loadTasks().filter(t => t.user_id === args.userId);
    if (args.status) tasks = tasks.filter(t => t.status === args.status);
    return tasks;
  },

  async get_task(args: { id: string }) {
    return loadTasks().find(t => t.id === args.id) || null;
  },

  async update_task(args: any) {
    const tasks = loadTasks();
    const idx = tasks.findIndex(t => t.id === args.id);
    if (idx === -1) throw 'Task not found';
    const t = tasks[idx];
    // History protection
    if (t.date < todayStr()) throw 'cannot modify historical task';
    if (t.task_type === 'project') throw 'project task cannot be modified via UI';
    if (args.title !== undefined) t.title = args.title;
    if (args.description !== undefined) t.description = args.description;
    if (args.category !== undefined) t.category = args.category;
    if (args.estimatedMinutes !== undefined) {
      if (args.estimatedMinutes <= 0) throw 'estimated_minutes must be > 0';
      t.estimated_minutes = args.estimatedMinutes;
    }
    if (t.status === 'completed' || t.status === 'abandoned') throw 'task is already in terminal state';
    if (args.actualMinutes !== undefined) t.actual_minutes = args.actualMinutes;
    if (args.notes !== undefined) t.notes = args.notes;
    if (args.status !== undefined) t.status = args.status;
    t.updated_at = nowStr();
    saveTasks(tasks);
    return t;
  },

  async complete_task(args: { id: string; actualMinutes: number }) {
    if (args.id.startsWith('daily:')) throw 'cannot complete virtual task';
    if (args.actualMinutes < 0) throw 'actual_minutes must be >= 0';
    const tasks = loadTasks();
    const idx = tasks.findIndex(t => t.id === args.id);
    if (idx === -1) throw 'Task not found';
    const t = tasks[idx];
    if (t.status === 'completed' || t.status === 'abandoned') throw 'task is already in terminal state';
    if (t.date < todayStr()) throw 'cannot modify historical task';
    if (t.task_type === 'project') throw 'project task cannot be modified via UI';
    const bonus = calcTaskBonus(t.category, args.actualMinutes);
    t.status = 'completed';
    t.actual_minutes = args.actualMinutes;
    t.completed_at = nowStr();
    t.updated_at = nowStr();
    saveTasks(tasks);
    // Apply bonus to daily record
    const user = loadUser();
    const records = loadFaithRecords();
    const date = t.date || todayStr();
    const key = `${t.user_id}_${date}`;
    const old = records[key];
    const oldBW = old?.task_bonus_work ?? 0;
    const oldBS = old?.task_bonus_study ?? 0;
    const record: DailyRecord = {
      id: old?.id ?? null, user_id: t.user_id, date,
      work_minutes: old?.work_minutes ?? 0, study_minutes: old?.study_minutes ?? 0,
      survival_faith: old?.survival_faith ?? 0, progress_faith: old?.progress_faith ?? 0,
      discipline_faith: old?.discipline_faith ?? 0,
      total_faith: (old?.total_faith ?? 0) + bonus,
      task_bonus_work: t.category === 'work' ? oldBW + bonus : oldBW,
      task_bonus_study: t.category === 'study' ? oldBS + bonus : oldBS,
      break_count: old?.break_count ?? 0, leave_record: old?.leave_record ?? 0, close_record: old?.close_record ?? 0,
      discipline_a: old?.discipline_a ?? 0, discipline_b: old?.discipline_b ?? 0, discipline_c: old?.discipline_c ?? 0,
      tasks_completed: (old?.tasks_completed ?? 0) + 1,
      created_at: old?.created_at ?? nowStr(), updated_at: nowStr(),
    };
    records[key] = record;
    saveFaithRecords(records);
    user.cumulative_faith += bonus;
    const lvl = getLevel(user.cumulative_faith);
    user.current_level = lvl.level;
    user.armor_points = calcArmor(user.current_level);
    user.updated_at = nowStr();
    saveUser(user);
    return { task: t, bonus_faith: bonus, bonus_category: t.category } as TaskCompleteResult;
  },

  async abandon_task(args: { id: string }) {
    if (args.id.startsWith('daily:')) throw 'cannot abandon virtual task';
    const tasks = loadTasks();
    const t = tasks.find(t => t.id === args.id);
    if (!t) throw 'Task not found';
    if (t.status === 'completed' || t.status === 'abandoned') throw 'task is already in terminal state';
    if (t.date < todayStr()) throw 'cannot modify historical task';
    if (t.task_type === 'project') throw 'project task cannot be modified via UI';
    t.status = 'abandoned';
    t.updated_at = nowStr();
    saveTasks(tasks);
    return t;
  },

  async delete_task(args: { id: string }) {
    if (args.id.startsWith('daily:')) return true;
    const tasks = loadTasks();
    const idx = tasks.findIndex(t => t.id === args.id);
    if (idx === -1) throw 'Task not found';
    const t = tasks[idx];
    if (t.date < todayStr()) throw 'cannot modify historical task';
    if (t.task_type === 'project') throw 'project task cannot be modified via UI';
    tasks.splice(idx, 1);
    saveTasks(tasks);
    return true;
  },

  async start_task(args: { id: string }) {
    let realId = args.id;
    if (args.id.startsWith('daily:')) {
      const parts = args.id.split(':');
      const tplId = parts[1];
      const date = parts[2];
      const tasks = loadTasks();
      const existing = tasks.find(t => t.template_id === tplId && t.date === date);
      if (existing) { realId = existing.id; }
      else {
        const tpl = tasks.find(t => t.id === tplId);
        if (!tpl) throw 'Template not found';
        const now = nowStr();
        const newTask: Task = {
          id: `task-${Date.now()}-${Math.random().toString(36).slice(2)}`,
          user_id: tpl.user_id, date, title: tpl.title, description: tpl.description,
          category: tpl.category, estimated_minutes: tpl.estimated_minutes, actual_minutes: 0,
          status: 'paused', notes: '', created_at: now, started_at: null,
          completed_at: null, duration_seconds: 0, ai_summary: null, updated_at: now,
          recurrence_kind: 'none', template_id: tpl.id, task_type: 'daily',
          source_tool: null, tool_session_id: null,
        };
        realId = newTask.id;
        tasks.push(newTask);
        saveTasks(tasks);
      }
    }
    const tasks = loadTasks();
    const t = tasks.find(t => t.id === realId);
    if (!t) throw 'Task not found';
    if (t.status === 'completed' || t.status === 'abandoned') throw 'task is already in terminal state';
    if (t.status === 'running') return t;
    t.status = 'running';
    t.started_at = nowStr();
    t.updated_at = nowStr();
    saveTasks(tasks);
    return t;
  },

  async pause_task(args: { id: string }) {
    const tasks = loadTasks();
    const t = tasks.find(t => t.id === args.id);
    if (!t) throw 'Task not found';
    // Simulate session settlement: calculate elapsed time and accumulate to daily record
    if (t.started_at && t.status === 'running') {
      const elapsedMs = Date.now() - new Date(t.started_at).getTime();
      const elapsedSecs = Math.max(0, Math.floor(elapsedMs / 1000));
      t.duration_seconds = (t.duration_seconds || 0) + elapsedSecs;
      const minutes = Math.floor(elapsedSecs / 60);
      if (minutes > 0) {
        const user = loadUser();
        const records = loadFaithRecords();
        const date = t.date || todayStr();
        const key = `${t.user_id}_${date}`;
        const old = records[key];
        const oldWork = old?.work_minutes ?? 0;
        const oldStudy = old?.study_minutes ?? 0;
        const [newWork, newStudy] = t.category === 'study'
          ? [oldWork, oldStudy + minutes]
          : [oldWork + minutes, oldStudy];
        const survival = calcSurvivalFaith(newWork);
        const progress = calcProgressFaith(newStudy);
        const [discipline, da, db, dc] = calcDiscipline(
          old?.break_count ?? 0, old?.leave_record ?? 0, old?.close_record ?? 0,
        );
        const oldBW = old?.task_bonus_work ?? 0;
        const oldBS = old?.task_bonus_study ?? 0;
        const total = survival + progress + discipline + oldBW + oldBS;
        const oldTotal = old?.total_faith ?? 0;
        const record: DailyRecord = {
          id: old?.id ?? null, user_id: t.user_id, date,
          work_minutes: newWork, study_minutes: newStudy,
          survival_faith: survival, progress_faith: progress, discipline_faith: discipline,
          total_faith: total, task_bonus_work: oldBW, task_bonus_study: oldBS,
          break_count: old?.break_count ?? 0, leave_record: old?.leave_record ?? 0,
          close_record: old?.close_record ?? 0,
          discipline_a: da, discipline_b: db, discipline_c: dc,
          tasks_completed: old?.tasks_completed ?? 0,
          created_at: old?.created_at ?? nowStr(), updated_at: nowStr(),
        };
        records[key] = record;
        saveFaithRecords(records);
        const delta = total - oldTotal;
        if (delta !== 0) {
          user.cumulative_faith += delta;
          const lvl = getLevel(user.cumulative_faith);
          user.current_level = lvl.level;
          user.armor_points = calcArmor(user.current_level);
          user.updated_at = nowStr();
          saveUser(user);
        }
      }
    }
    t.status = 'paused';
    t.updated_at = nowStr();
    saveTasks(tasks);
    return t;
  },

  async resume_task(args: { id: string }) {
    const tasks = loadTasks();
    const t = tasks.find(t => t.id === args.id);
    if (!t) throw 'Task not found';
    if (t.status === 'completed' || t.status === 'abandoned') throw 'task is already in terminal state';
    if (t.status === 'running') return t;
    t.status = 'running';
    t.started_at = nowStr();
    t.updated_at = nowStr();
    saveTasks(tasks);
    return t;
  },

  async end_task(args: { id: string }) {
    // Delegate to pause_task first to accumulate session time, then mark completed
    const paused = await this.pause_task(args);
    const tasks = loadTasks();
    const t = tasks.find(t => t.id === paused.id);
    if (!t) throw 'Task not found';
    t.status = 'completed';
    t.completed_at = nowStr();
    t.updated_at = nowStr();
    saveTasks(tasks);
    return t;
  },

  async set_task_recurrence(args: { id: string; kind: string }) {
    if (args.id.startsWith('daily:')) throw 'cannot set recurrence on virtual instance';
    const tasks = loadTasks();
    const t = tasks.find(t => t.id === args.id);
    if (!t) throw 'Task not found';
    t.recurrence_kind = args.kind as any;
    t.updated_at = nowStr();
    saveTasks(tasks);
    return t;
  },

  async get_daily_stats(args: { userId: string; date: string }) {
    const records = loadFaithRecords();
    const key = `${args.userId}_${args.date}`;
    const record = records[key];
    if (!record) return null;
    const user = loadUser();
    return { ...record, cumulative_faith: user.cumulative_faith };
  },

  async get_project_task(args: { sessionId: string }) {
    return loadTasks().find(t => t.tool_session_id === args.sessionId) || null;
  },

  async get_project_tasks(args: { userId: string }) {
    return loadTasks().filter(t => t.user_id === args.userId && t.task_type === 'project' && (t.status === 'running' || t.status === 'paused'));
  },

  async open_floating_widget() {},
  async close_floating_widget() {},
  async show_main_window() {},
};

export async function safeInvoke<T>(command: string, args: Record<string, any> = {}): Promise<T> {
  try {
    if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke(command, args);
    }
    if (handlers[command]) {
      return await handlers[command](args) as T;
    }
    throw new Error(`Unknown command: ${command}`);
  } catch (e: any) {
    console.error(`[safeInvoke] ${command} failed:`, typeof e === 'string' ? e : e?.message || e);
    throw e;
  }
}
