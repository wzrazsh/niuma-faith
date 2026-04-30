// frontend/src/services/reminder-service.ts

class ReminderService {
  private intervalId: number | null = null;
  private reminders: Map<string, { time: string; taskTitle: string }> = new Map();

  start() {
    if (this.intervalId) return;
    
    // 每分钟检查一次
    this.intervalId = window.setInterval(() => {
      this.checkReminders();
    }, 60000);
  }

  stop() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  addReminder(taskId: string, time: string, taskTitle: string) {
    this.reminders.set(taskId, { time, taskTitle });
  }

  removeReminder(taskId: string) {
    this.reminders.delete(taskId);
  }

  private checkReminders() {
    const now = new Date();
    const currentTime = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;
    
    for (const [_, reminder] of this.reminders) {
      if (reminder.time === currentTime) {
        this.showNotification(reminder.taskTitle);
      }
    }
  }

  private async showNotification(taskTitle: string) {
    // 尝试使用 Tauri Notification API
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('show_notification', {
        title: '任务提醒',
        body: `任务「${taskTitle}」的截止时间已到！`
      });
    } catch {
      // 降级到浏览器通知
      if ('Notification' in window && Notification.permission === 'granted') {
        new Notification('任务提醒', {
          body: `任务「${taskTitle}」的截止时间已到！`
        });
      } else if ('Notification' in window && Notification.permission !== 'denied') {
        Notification.requestPermission();
      }
    }
  }
}

export const reminderService = new ReminderService();
