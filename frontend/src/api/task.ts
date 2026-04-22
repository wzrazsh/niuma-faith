import { invoke } from "@tauri-apps/api/core";
import type { Task, TaskCompleteResult, DailyStats, TaskStatus } from "@/types";

const DEFAULT_USER_ID = "default_user";

export async function invoke_create_task(
  title: string,
  description: string,
  category: string,
  estimated_minutes: number
): Promise<Task> {
  return invoke<Task>("create_task", {
    userId: DEFAULT_USER_ID,
    title,
    description,
    category,
    estimatedMinutes: estimated_minutes,
  });
}

export async function invoke_get_tasks(status?: TaskStatus): Promise<Task[]> {
  return invoke<Task[]>("get_tasks", {
    userId: DEFAULT_USER_ID,
    status: status ?? null,
  });
}

export async function invoke_get_task(id: string): Promise<Task | null> {
  return invoke<Task | null>("get_task", { id });
}

export async function invoke_update_task(
  id: string,
  title?: string,
  description?: string,
  estimated_minutes?: number,
  notes?: string
): Promise<Task> {
  return invoke<Task>("update_task", {
    id,
    title: title ?? null,
    description: description ?? null,
    estimatedMinutes: estimated_minutes ?? null,
    notes: notes ?? null,
  });
}

export async function invoke_complete_task(
  id: string,
  actual_minutes: number
): Promise<TaskCompleteResult> {
  return invoke<TaskCompleteResult>("complete_task", {
    id,
    actualMinutes: actual_minutes,
  });
}

export async function invoke_start_task(id: string): Promise<Task> {
  return invoke<Task>("start_task", { id });
}

export async function invoke_pause_task(id: string): Promise<Task> {
  return invoke<Task>("pause_task", { id });
}

export async function invoke_resume_task(id: string): Promise<Task> {
  return invoke<Task>("resume_task", { id });
}

export async function invoke_end_task(id: string): Promise<Task> {
  return invoke<Task>("end_task", { id });
}

export async function invoke_abandon_task(id: string): Promise<Task> {
  return invoke<Task>("abandon_task", { id });
}

export async function invoke_delete_task(id: string): Promise<boolean> {
  return invoke<boolean>("delete_task", { id });
}

export async function invoke_get_daily_stats(date: string): Promise<DailyStats> {
  return invoke<DailyStats>("get_daily_stats", {
    userId: DEFAULT_USER_ID,
    date,
  });
}
