<template>
  <nav v-if="$route.path !== '/floating'" class="nav-bar">
    <router-link to="/">仪表盘</router-link>
    <router-link to="/kanban">任务看板</router-link>
  </nav>
  <router-view />
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useFaithStore } from '@/stores/faith';

const route = useRoute();

onMounted(async () => {
  if (route.path !== '/floating') {
    const faith = useFaithStore();
    await faith.init();
  }
});
</script>

<style scoped>
.nav-bar {
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  padding: 8px 16px;
  display: flex;
  gap: 16px;
}
.nav-bar a {
  text-decoration: none;
  color: var(--color-text-muted);
  font-size: 0.875rem;
  padding: 4px 12px;
  border-radius: 6px;
}
.nav-bar a:hover { background: var(--color-bg); color: var(--color-text); }
.nav-bar a.router-link-active { background: var(--color-primary); color: #1a1a24; font-weight: 600; }
</style>
