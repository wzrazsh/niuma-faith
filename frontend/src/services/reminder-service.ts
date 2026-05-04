const REMINDER_INTERVAL_MS = 60000;
let reminderTimer: number | null = null;

export function startReminderService() {
  if (reminderTimer) return;
  reminderTimer = window.setInterval(() => {
    // Check reminders - in full implementation would iterate kanban cards
  }, REMINDER_INTERVAL_MS);
}

export function stopReminderService() {
  if (reminderTimer) { window.clearInterval(reminderTimer); reminderTimer = null; }
}
