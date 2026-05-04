import { safeInvoke } from '@/api/mock-invoke';
import { useKanbanStore } from '@/stores/kanban';

const POLL_INTERVAL_MS = 3000;
let pollTimer: number | null = null;

export async function checkProcess(appName: string): Promise<boolean> {
  return safeInvoke<boolean>('is_process_running', { appName });
}

export function startPolling() {
  if (pollTimer) return;
  pollTimer = window.setInterval(async () => {
    const kanban = useKanbanStore();
    for (const [_, card] of kanban.cards) {
      if (card.processBinding) {
        const running = await checkProcess(card.processBinding.appName);
        // Auto start/pause would go here
      }
    }
  }, POLL_INTERVAL_MS);
}

export function stopPolling() {
  if (pollTimer) {
    window.clearInterval(pollTimer);
    pollTimer = null;
  }
}
