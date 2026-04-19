import { createRouter, createWebHashHistory } from 'vue-router';
import MainView from './components/MainView.vue';
import FloatingWidget from './components/FloatingWidget.vue';
import TasksPage from './components/TasksPage.vue';

const routes = [
  { path: '/', component: MainView },
  { path: '/floating', component: FloatingWidget },
  { path: '/tasks', component: TasksPage },
  { path: '/:pathMatch(.*)*', redirect: '/' },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
