import { safeInvoke } from "@/api/mock-invoke";
import type { ProcessInfo } from "@/types";

export const processDetector = {
  async isRunning(appName: string): Promise<boolean> {
    return safeInvoke<boolean>("is_process_running", { appName });
  },

  async listProcesses(appName: string): Promise<ProcessInfo[]> {
    return safeInvoke<ProcessInfo[]>("list_processes", { appName });
  },

  startPolling(
    appName: string,
    intervalMs: number,
    onStatusChange: (running: boolean) => void
  ): () => void {
    let lastStatus: boolean | null = null;

    const check = async () => {
      try {
        const running = await this.isRunning(appName);
        if (running !== lastStatus) {
          lastStatus = running;
          onStatusChange(running);
        }
      } catch (e) {
        console.error("Process poll error:", e);
      }
    };

    check();

    const id = window.setInterval(check, intervalMs);
    return () => clearInterval(id);
  },
};
