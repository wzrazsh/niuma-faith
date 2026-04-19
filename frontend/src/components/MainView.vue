<script setup lang="ts">
import { useRouter } from "vue-router";
import StatusPanel from "@/components/StatusPanel.vue";
import CheckInForm from "@/components/CheckInForm.vue";

const router = useRouter();
</script>

<template>
  <div class="app-container">
    <header class="app-header">
      <h1 class="app-title">牛马信仰</h1>
      <p class="app-subtitle">每日精进，积累信仰之力</p>
    </header>

    <main class="app-main">
      <StatusPanel />
      <CheckInForm />
    </main>

    <footer class="app-footer">
      <button class="nav-link" @click="router.push('/tasks')">
        任务管理系统 →
      </button>
    </footer>
  </div>
</template>

<style scoped>
.app-container {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 24px 16px;
}

.app-header {
  text-align: center;
  margin-bottom: 32px;
}

.app-title {
  font-size: 2rem;
  font-weight: 700;
  color: var(--color-primary);
  letter-spacing: 4px;
}

.app-subtitle {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  margin-top: 6px;
  letter-spacing: 2px;
}

.app-main {
  width: 100%;
  max-width: 480px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.app-footer {
  margin-top: 24px;
}

.nav-link {
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 0.875rem;
  cursor: pointer;
  padding: 8px 16px;
  border-radius: 8px;
  transition: color 0.15s;
}

.nav-link:hover {
  color: var(--color-primary);
}
</style>
