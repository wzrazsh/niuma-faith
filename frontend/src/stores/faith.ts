import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { FaithStatus, DailyRecord, User } from "@/types";
import {
  invoke_get_status,
  invoke_get_today_record,
  invoke_get_or_create_user,
} from "@/api/tauri";

export const useFaithStore = defineStore("faith", () => {
  // State
  const faithStatus = ref<FaithStatus | null>(null);
  const user = ref<User | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // Computed shortcuts
  const todayRecord = computed<DailyRecord | null>(() => faithStatus.value?.today ?? null);

  const todayFaith = computed(() => {
    const rec = todayRecord.value;
    if (rec) {
      return {
        survival_faith: rec.survival_faith,
        progress_faith: rec.progress_faith,
        discipline_faith: rec.discipline_faith,
        total_faith: rec.total_faith,
      };
    }
    return { survival_faith: 0, progress_faith: 0, discipline_faith: 0, total_faith: 0 };
  });

  const currentLevel = computed(() => ({
    level: faithStatus.value?.current_level ?? 1,
    title: faithStatus.value?.level_title ?? "见习牛马",
    cumulative_threshold: 0, // not directly available; use next_threshold for interval
  }));

  const nextLevelThreshold = computed(() => faithStatus.value?.next_threshold ?? null);

  const progressToNext = computed(() => faithStatus.value?.progress_to_next ?? 0);

  const cumulativeFaith = computed(() => faithStatus.value?.cumulative_faith ?? 0);

  // Actions
  async function init() {
    try {
      isLoading.value = true;
      error.value = null;
      const u = await invoke_get_or_create_user();
      user.value = u;
      await fetchStatus();
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function fetchStatus() {
    try {
      isLoading.value = true;
      error.value = null;
      const result = await invoke_get_status();
      faithStatus.value = result;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function fetchTodayRecord() {
    try {
      const rec = await invoke_get_today_record();
      if (faithStatus.value) {
        faithStatus.value = { ...faithStatus.value, today: rec };
      }
    } catch (e) {
      error.value = String(e);
    }
  }

  async function ensureUser() {
    try {
      user.value = await invoke_get_or_create_user();
    } catch (e) {
      error.value = String(e);
    }
  }

  return {
    // State
    faithStatus,
    user,
    isLoading,
    error,
    // Computed
    todayRecord,
    todayFaith,
    currentLevel,
    nextLevelThreshold,
    progressToNext,
    cumulativeFaith,
    // Actions
    init,
    fetchStatus,
    fetchTodayRecord,
    ensureUser,
  };
});
