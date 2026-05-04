<template>
  <div class="floating-widget" @dblclick="openMain">
    <div class="circle">
      <span class="lv">Lv{{ faith.faithStatus?.current_level ?? 1 }}</span>
      <span class="title">{{ faith.faithStatus?.level_title ?? '见习牛马' }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useFaithStore } from '@/stores/faith';
import { safeInvoke } from '@/api/mock-invoke';

const faith = useFaithStore();

onMounted(async () => { await faith.init(); });

async function openMain() { await safeInvoke('show_main_window'); }
</script>

<style scoped>
.floating-widget { width: 80px; height: 80px; display: flex; align-items: center; justify-content: center; }
.circle { width: 60px; height: 60px; border-radius: 50%; border: 2px solid var(--color-primary); display: flex; flex-direction: column; align-items: center; justify-content: center; background: var(--color-bg); cursor: pointer; }
.lv { font-size: 0.9rem; font-weight: 700; color: var(--color-primary); }
.title { font-size: 0.5rem; color: var(--color-text-muted); }
</style>
