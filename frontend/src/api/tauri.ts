import type { FaithStatus, DailyRecord, User } from '@/types';
import { safeInvoke } from './mock-invoke';
const UID = 'default_user';

export function invoke_get_status(): Promise<FaithStatus> {
  return safeInvoke('get_status', { userId: UID });
}

export function invoke_get_today_record(): Promise<DailyRecord | null> {
  return safeInvoke('get_today_record', { userId: UID });
}

export function invoke_get_or_create_user(): Promise<User & { armor: number; total_armor: number }> {
  return safeInvoke('get_or_create_user');
}

export function invoke_check_in(workMinutes: number, studyMinutes: number, breakCount: number, leaveRecord: number, closeRecord: number): Promise<FaithStatus> {
  return safeInvoke('check_in', { userId: UID, workMinutes, studyMinutes, breakCount, leaveRecord, closeRecord });
}
