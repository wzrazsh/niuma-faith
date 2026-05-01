// Types matching the Rust backend (src-tauri/src/domain/models.rs and task.rs)

export interface User {
  id: string;
  nickname: string;
  cumulative_faith: number;
  current_level: number;
  armor: number;
  total_armor: number;
  created_at: string;
  updated_at: string;
}

export interface DisciplineInput {
  break_count: number;
  leave_record: number; // 0: 无, 1: 已解释, 2: 未解释
  close_record: number; // 0: 未完成, 1: 已完成
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

/**
 * FaithStatus — matches the Rust FaithStatus struct exactly.
 * Returned by check_in and get_status commands.
 */
export interface FaithStatus {
  user_id: string;
  cumulative_faith: number;
  current_level: number;
  level_title: string;
  progress_to_next: number;
  next_threshold: number | null;
  today: DailyRecord | null;
  armor: number;
  total_armor: number;
}

// --- Task types ---

export type TaskCategory = 'work' | 'study' | 'other';
export type TaskStatus = 'running' | 'paused' | 'completed' | 'abandoned';

export interface Task {
  id: string;
  user_id: string;
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
}

export interface TaskCompleteResult {
  task: Task;
  bonus_faith: number;
  bonus_category: TaskCategory;
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

export interface ProcessInfo {
  pid: number;
  name: string;
  status: string;
}
