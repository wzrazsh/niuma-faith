<script setup lang="ts">
import { onMounted } from "vue";
import { useFaithStore } from "@/stores/faith";

const store = useFaithStore();

onMounted(async () => {
  await store.init();
});
</script>

<template>
  <div id="app">
    <nav class="main-nav">
      <router-link to="/" class="nav-link">仪表盘</router-link>
      <router-link to="/kanban" class="nav-link">任务看板</router-link>
    </nav>
    <router-view />
  </div>
</template>

<style scoped>
.main-nav {
  background: var(--color-surface);
  padding: 8px 16px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  gap: 16px;
}

.nav-link {
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.875rem;
  color: var(--color-text-muted);
  text-decoration: none;
  transition: all 0.15s;
}

.nav-link:hover {
  background: var(--color-bg);
  color: var(--color-text);
}

.nav-link.router-link-active {
  background: var(--color-primary);
  color: #1a1a24;
  font-weight: 600;
}
</style>
