export interface User {
  id: string;
  nickname: string;
  cumulative_faith: number;
  current_level: number;
  armor_points: number;
  created_at: string;
  updated_at: string;
}

export interface DailyRecord {
  id: number | null;
  user_id: string;
  date: string;
  work_minutes: number;
  study_minutes: number;
  survival_faith: number;
  progress_faith: number;
  discipline_faith: number;
  total_faith: number;
  task_bonus_work: number;
  task_bonus_study: number;
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

export interface FaithStatus {
  user_id: string;
  cumulative_faith: number;
  current_level: number;
  level_title: string;
  progress_to_next: number | null;
  next_threshold: number | null;
  today: DailyRecord | null;
  armor: number;
  total_armor: number;
}

export type TaskCategory = 'work' | 'study' | 'other';
export type TaskStatus = 'running' | 'paused' | 'completed' | 'abandoned';
export type RecurrenceKind = 'none' | 'daily';
export type TaskType = 'daily' | 'project';

export interface Task {
  id: string;
  user_id: string;
  date: string;
  title: string;
  description: string;
  category: TaskCategory;
  estimated_minutes: number;
  actual_minutes: number;
  status: TaskStatus;
  notes: string;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  duration_seconds: number;
  ai_summary: string | null;
  updated_at: string;
  recurrence_kind: RecurrenceKind;
  template_id: string | null;
  task_type: TaskType;
  source_tool: string | null;
  tool_session_id: string | null;
}

export interface TaskCompleteResult {
  task: Task;
  bonus_faith: number;
  bonus_category: TaskCategory;
}

export interface ProcessInfo {
  pid: number;
  name: string;
  status: string;
}

export interface DailyStats {
  date: string;
  work_minutes: number;
  study_minutes: number;
  survival_faith: number;
  progress_faith: number;
  discipline_faith: number;
  total_faith: number;
  task_bonus_work: number;
  task_bonus_study: number;
  tasks_completed: number;
  cumulative_faith: number;
}

export interface FaithTransaction {
  id: number | null;
  user_id: string;
  ts: string;
  delta: number;
  armor_delta: number;
  kind: string;
  ref_id: string | null;
  message: string;
}
