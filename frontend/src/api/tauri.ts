import { invoke } from "@tauri-apps/api/core";
import type { FaithStatus, DailyRecord, User, DisciplineInput } from "@/types";

// MVP: single-user app — fixed default user ID
const DEFAULT_USER_ID = "default_user";

/**
 * Submit daily check-in: work, study duration in minutes, and discipline input.
 * Triggers faith calculation on the Rust backend.
 */
export async function invoke_check_in(
  work_minutes: number,
  study_minutes: number,
  discipline: DisciplineInput
): Promise<FaithStatus> {
  return invoke<FaithStatus>("check_in", {
    userId: DEFAULT_USER_ID,
    workMinutes: work_minutes,
    studyMinutes: study_minutes,
    breakCount: discipline.break_count,
    leaveRecord: discipline.leave_record,
    closeRecord: discipline.close_record,
  });
}

/**
 * Get the current user's faith status, including level progress.
 */
export async function invoke_get_status(): Promise<FaithStatus> {
  return invoke<FaithStatus>("get_status", { userId: DEFAULT_USER_ID });
}

/**
 * Get today's check-in record (null if not checked in today).
 */
export async function invoke_get_today_record(): Promise<DailyRecord | null> {
  return invoke<DailyRecord | null>("get_today_record", { userId: DEFAULT_USER_ID });
}

/**
 * Get or create the default user record.
 */
export async function invoke_get_or_create_user(): Promise<User> {
  return invoke<User>("get_or_create_user");
}
