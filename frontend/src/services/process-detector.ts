import { safeInvoke } from '@/api/mock-invoke';
import { useKanbanStore } from '@/stores/kanban';
import { useTaskStore } from '@/stores/task';

const POLL_INTERVAL_MS = 3000;
let pollTimer: number | null = null;

export async function checkProcess(appName: string): Promise<boolean> {
  return safeInvoke<boolean>('is_process_running', { appName });
}

export function startPolling() {
  if (pollTimer) return;
  pollTimer = window.setInterval(async () => {
    const kanban = useKanbanStore();
    const taskStore = useTaskStore();
    for (const [_, card] of kanban.cards) {
      if (card.processBinding) {
        const running = await checkProcess(card.processBinding.appName);
        const task = taskStore.tasks.find(t => t.id === card.taskId);
        if (!task) continue;

        if (running && card.processBinding.autoStart && task.status !== 'running') {
          try {
            await taskStore.startTask(card.taskId);
            kanban.startTimer(card.taskId);
          } catch (e) {
            console.error('[process-detector] autoStart failed:', e);
          }
        }

        if (!running && card.processBinding.autoPause && task.status === 'running') {
          try {
            await taskStore.pauseTask(card.taskId);
            kanban.stopTimer(card.taskId);
          } catch (e) {
            console.error('[process-detector] autoPause failed:', e);
          }
        }
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
