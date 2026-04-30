import { invoke } from "@tauri-apps/api/core";
import type { FaithStatus, DailyRecord, User } from "@/types";

// MVP: single-user app — fixed default user ID
const DEFAULT_USER_ID = "default_user";

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
