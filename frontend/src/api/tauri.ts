import { safeInvoke } from "./mock-invoke";
import type { FaithStatus, DailyRecord, User } from "@/types";

// MVP: single-user app — fixed default user ID
const DEFAULT_USER_ID = "default_user";

/**
 * Get the current user's faith status, including level progress.
 */
export async function invoke_get_status(): Promise<FaithStatus> {
  return safeInvoke<FaithStatus>("get_status", { userId: DEFAULT_USER_ID });
}

/**
 * Get today's check-in record (null if not checked in today).
 */
export async function invoke_get_today_record(): Promise<DailyRecord | null> {
  return safeInvoke<DailyRecord | null>("get_today_record", { userId: DEFAULT_USER_ID });
}

/**
 * Get or create the default user record.
 */
export async function invoke_get_or_create_user(): Promise<User> {
  return safeInvoke<User>("get_or_create_user");
}
