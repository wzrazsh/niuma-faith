import { createRouter, createWebHashHistory } from 'vue-router';
import Dashboard from './components/Dashboard.vue';
import FloatingWidget from './components/FloatingWidget.vue';

const routes = [
  { path: '/', component: Dashboard },
  { path: '/floating', component: FloatingWidget },
  { path: '/:pathMatch(.*)*', redirect: '/' },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
