import type { Task, TaskCompleteResult, DailyStats } from '@/types';
import { safeInvoke } from './mock-invoke';
const UID = 'default_user';

export function invoke_create_task(title: string, description: string, category: string, estimatedMinutes: number, date?: string, recurrenceKind?: string): Promise<Task> {
  return safeInvoke('create_task', { userId: UID, title, description, category, estimatedMinutes, date: date || null, recurrenceKind: recurrenceKind || null });
}

export function invoke_get_tasks_by_date(date: string, status?: string): Promise<Task[]> {
  return safeInvoke('get_tasks_by_date', { userId: UID, date, status: status || null });
}

export function invoke_get_tasks(status?: string): Promise<Task[]> {
  return safeInvoke('get_tasks', { userId: UID, status: status || null });
}

export function invoke_get_task(id: string): Promise<Task | null> {
  return safeInvoke('get_task', { id });
}

export function invoke_update_task(id: string, title?: string, description?: string, estimatedMinutes?: number, actualMinutes?: number, notes?: string, status?: string): Promise<Task> {
  return safeInvoke('update_task', { id, title: title || null, description: description || null, estimatedMinutes: estimatedMinutes ?? null, actualMinutes: actualMinutes ?? null, notes: notes || null, status: status || null });
}

export function invoke_complete_task(id: string, actualMinutes: number): Promise<TaskCompleteResult> {
  return safeInvoke('complete_task', { id, actualMinutes });
}

export function invoke_abandon_task(id: string): Promise<Task> {
  return safeInvoke('abandon_task', { id });
}

export function invoke_delete_task(id: string): Promise<boolean> {
  return safeInvoke('delete_task', { id });
}

export function invoke_start_task(id: string): Promise<Task> {
  return safeInvoke('start_task', { id });
}

export function invoke_pause_task(id: string): Promise<Task> {
  return safeInvoke('pause_task', { id });
}

export function invoke_resume_task(id: string): Promise<Task> {
  return safeInvoke('resume_task', { id });
}

export function invoke_end_task(id: string): Promise<Task> {
  return safeInvoke('end_task', { id });
}

export function invoke_set_task_recurrence(id: string, kind: string): Promise<Task> {
  return safeInvoke('set_task_recurrence', { id, kind });
}

export function invoke_get_daily_stats(date: string): Promise<DailyStats | null> {
  return safeInvoke('get_daily_stats', { userId: UID, date });
}
