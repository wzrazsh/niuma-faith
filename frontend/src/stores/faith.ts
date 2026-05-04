import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { FaithStatus, User, DailyRecord } from '@/types';
import { invoke_get_status, invoke_get_or_create_user, invoke_check_in } from '@/api/tauri';

export const useFaithStore = defineStore('faith', () => {
  const faithStatus = ref<FaithStatus | null>(null);
  const user = ref<User | null>(null);
  const todayRecord = ref<DailyRecord | null>(null);
  const loading = ref(false);

  async function init() {
    loading.value = true;
    try {
      user.value = await invoke_get_or_create_user();
      faithStatus.value = await invoke_get_status();
      todayRecord.value = faithStatus.value?.today ?? null;
    } finally {
      loading.value = false;
    }
  }

  async function refreshStatus() {
    faithStatus.value = await invoke_get_status();
    todayRecord.value = faithStatus.value?.today ?? null;
  }

  async function checkIn(workMinutes: number, studyMinutes: number, breakCount: number, leaveRecord: number, closeRecord: number) {
    faithStatus.value = await invoke_check_in(workMinutes, studyMinutes, breakCount, leaveRecord, closeRecord);
    todayRecord.value = faithStatus.value.today;
  }

  return { faithStatus, user, todayRecord, loading, init, refreshStatus, checkIn };
});
