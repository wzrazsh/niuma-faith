import { createRouter, createWebHashHistory } from 'vue-router';

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', component: () => import('./components/Dashboard.vue') },
    { path: '/kanban', component: () => import('./components/KanbanPage.vue') },
    { path: '/floating', component: () => import('./components/FloatingWidget.vue') },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
});

export default router;
