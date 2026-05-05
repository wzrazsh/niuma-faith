<template>
  <nav v-if="$route.path !== '/floating'" class="nav-bar">
    <div class="nav-brand">
      <span class="nav-logo">✦</span>
      <span class="nav-title">牛马信仰</span>
    </div>
    <div class="nav-links">
      <router-link to="/">
        <span class="nav-icon">◈</span>
        <span>仪表盘</span>
      </router-link>
      <router-link to="/kanban">
        <span class="nav-icon">▣</span>
        <span>任务看板</span>
      </router-link>
    </div>
    <div class="nav-glow"></div>
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
  position: relative;
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border-subtle);
  padding: 0 20px;
  height: 44px;
  display: flex;
  align-items: center;
  gap: 24px;
  z-index: 10;
  overflow: hidden;
}

.nav-glow {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--color-primary), transparent);
  opacity: 0.3;
}

.nav-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-right: 8px;
}

.nav-logo {
  font-size: 1.1rem;
  color: var(--color-primary);
  text-shadow: 0 0 8px var(--color-primary-glow);
  animation: logo-pulse 2s ease-in-out infinite;
}

.nav-title {
  font-family: var(--font-display);
  font-size: 0.95rem;
  font-weight: 700;
  color: var(--color-text);
  letter-spacing: 0.08em;
}

.nav-links {
  display: flex;
  gap: 4px;
}

.nav-links a {
  text-decoration: none;
  color: var(--color-text-muted);
  font-size: 0.85rem;
  padding: 6px 14px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all var(--transition-fast);
  position: relative;
}

.nav-icon {
  font-size: 0.75rem;
  opacity: 0.6;
}

.nav-links a:hover {
  color: var(--color-text);
  background: rgba(255, 255, 255, 0.04);
}

.nav-links a.router-link-active {
  color: var(--color-primary);
  background: var(--color-primary-glow);
}

.nav-links a.router-link-active::after {
  content: '';
  position: absolute;
  bottom: -1px;
  left: 50%;
  transform: translateX(-50%);
  width: 60%;
  height: 2px;
  background: var(--color-primary);
  border-radius: 1px;
  box-shadow: 0 0 8px var(--color-primary-glow);
}

@keyframes logo-pulse {
  0%, 100% {
    text-shadow: 0 0 8px var(--color-primary-glow);
  }
  50% {
    text-shadow: 0 0 16px var(--color-primary-glow-strong), 0 0 24px var(--color-primary-glow);
  }
}
</style>
