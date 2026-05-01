import { safeInvoke } from "./mock-invoke";
import type { Task, TaskCompleteResult, DailyStats, TaskStatus } from "@/types";

const DEFAULT_USER_ID = "default_user";

export async function invoke_create_task(
  title: string,
  description: string,
  category: string,
  estimated_minutes: number,
  date?: string
): Promise<Task> {
  return safeInvoke<Task>("create_task", {
    userId: DEFAULT_USER_ID,
    title,
    description,
    category,
    estimatedMinutes: estimated_minutes,
    date: date ?? null,
  });
}

export async function invoke_get_tasks(status?: TaskStatus): Promise<Task[]> {
  return safeInvoke<Task[]>("get_tasks", {
    userId: DEFAULT_USER_ID,
    status: status ?? null,
  });
}

export async function invoke_get_tasks_by_date(
  date: string,
  status?: TaskStatus
): Promise<Task[]> {
  return safeInvoke<Task[]>("get_tasks_by_date", {
    userId: DEFAULT_USER_ID,
    date,
    status: status ?? null,
  });
}

export async function invoke_get_task(id: string): Promise<Task | null> {
  return safeInvoke<Task | null>("get_task", { id });
}

export async function invoke_update_task(
  id: string,
  title?: string,
  description?: string,
  estimated_minutes?: number,
  actual_minutes?: number,
  notes?: string,
  status?: string
): Promise<Task> {
  return safeInvoke<Task>("update_task", {
    id,
    title: title ?? null,
    description: description ?? null,
    estimatedMinutes: estimated_minutes ?? null,
    actualMinutes: actual_minutes ?? null,
    notes: notes ?? null,
    status: status ?? null,
  });
}

export async function invoke_complete_task(
  id: string,
  actual_minutes: number
): Promise<TaskCompleteResult> {
  return safeInvoke<TaskCompleteResult>("complete_task", {
    id,
    actualMinutes: actual_minutes,
  });
}

export async function invoke_start_task(id: string): Promise<Task> {
  return safeInvoke<Task>("start_task", { id });
}

export async function invoke_pause_task(id: string): Promise<Task> {
  return safeInvoke<Task>("pause_task", { id });
}

export async function invoke_resume_task(id: string): Promise<Task> {
  return safeInvoke<Task>("resume_task", { id });
}

export async function invoke_end_task(id: string): Promise<Task> {
  return safeInvoke<Task>("end_task", { id });
}

export async function invoke_abandon_task(id: string): Promise<Task> {
  return safeInvoke<Task>("abandon_task", { id });
}

export async function invoke_delete_task(id: string): Promise<boolean> {
  return safeInvoke<boolean>("delete_task", { id });
}

export async function invoke_get_daily_stats(date: string): Promise<DailyStats> {
  return safeInvoke<DailyStats>("get_daily_stats", {
    userId: DEFAULT_USER_ID,
    date,
  });
}
