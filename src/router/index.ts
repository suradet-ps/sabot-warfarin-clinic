import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', redirect: '/screening' },
    { path: '/screening', name: 'screening', component: () => import('#/views/ScreeningView.vue') },
    { path: '/active', name: 'active', component: () => import('#/views/ActiveView.vue') },
    { path: '/patient/:hn', name: 'patient-detail', component: () => import('#/views/PatientDetailView.vue') },
    { path: '/slip/:visitId', name: 'slip', component: () => import('#/views/SlipView.vue') },
    { path: '/reports', name: 'reports', component: () => import('#/views/ReportsView.vue') },
    { path: '/settings', name: 'settings', component: () => import('#/views/SettingsView.vue') },
  ],
})

export default router
