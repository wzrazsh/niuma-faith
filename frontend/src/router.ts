import { createRouter, createWebHashHistory } from 'vue-router';
import Dashboard from './components/Dashboard.vue';
import FloatingWidget from './components/FloatingWidget.vue';
import KanbanPage from './components/KanbanPage.vue';

const routes = [
  { path: '/', component: Dashboard },
  { path: '/kanban', component: KanbanPage },
  { path: '/floating', component: FloatingWidget },
  { path: '/:pathMatch(.*)*', redirect: '/' },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
