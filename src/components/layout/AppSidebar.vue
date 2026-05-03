<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { BarChart3, Heart, Search, Settings, Users } from 'lucide-vue-next'
import { useAlertStore } from '#/stores/alerts'
import { useSettingsStore } from '#/stores/settings'

const route = useRoute()
const alertStore = useAlertStore()
const settingsStore = useSettingsStore()

const navItems = [
  { name: 'screening', label: 'คัดกรอง', icon: Search, path: '/screening' },
  { name: 'active', label: 'ผู้ป่วยทั้งหมด', icon: Users, path: '/active' },
  { name: 'reports', label: 'รายงาน', icon: BarChart3, path: '/reports' },
  { name: 'settings', label: 'ตั้งค่า', icon: Settings, path: '/settings' },
]

const totalAlerts = computed(() => alertStore.criticalCount + alertStore.warningCount)

onMounted(() => {
  void Promise.all([alertStore.fetchAlerts(), settingsStore.loadSettings()])
})
</script>

<template>
  <nav class="sidebar">
    <div class="sidebar-logo"><Heart :size="24" class="sidebar-logo-icon" /><div class="sidebar-logo-text"><span class="sidebar-logo-title">วาร์ฟาริน</span><span class="sidebar-logo-sub">คลินิก</span></div></div>
    <ul class="sidebar-nav">
      <li v-for="item in navItems" :key="item.name"><RouterLink :to="item.path" class="sidebar-nav-item" :class="{ active: route.path.startsWith(item.path) }"><component :is="item.icon" :size="20" /><span>{{ item.label }}</span><span v-if="item.name === 'active' && totalAlerts > 0" class="sidebar-badge">{{ totalAlerts }}</span></RouterLink></li>
    </ul>
    <div class="sidebar-footer"><span class="micro footer-text">{{ settingsStore.hospitalName }}</span></div>
  </nav>
</template>

<style scoped>
.sidebar { width: 15rem; min-width: 15rem; background: var(--color-primary); display: flex; flex-direction: column; padding: var(--spacing-xl) 0; }
.sidebar-logo { display: flex; align-items: center; gap: var(--spacing-sm); padding: 0 var(--spacing-xl) var(--spacing-xxl); }
.sidebar-logo-icon { color: var(--color-brand-coral); }
.sidebar-logo-text { display: flex; flex-direction: column; }
.sidebar-logo-title { font-size: var(--typography-heading-5-size); font-weight: var(--typography-heading-5-weight); color: var(--color-on-dark); }
.sidebar-logo-sub,.footer-text { color: var(--color-on-dark-muted); }
.sidebar-nav { list-style: none; flex: 1; display: flex; flex-direction: column; gap: var(--spacing-xxs); padding: 0 var(--spacing-sm); }
.sidebar-nav-item { display: flex; align-items: center; gap: var(--spacing-sm); padding: var(--spacing-sm) var(--spacing-md); border-radius: var(--rounded-full); color: var(--color-on-dark-muted); text-decoration: none; }
.sidebar-nav-item.active { background: color-mix(in srgb, var(--color-on-dark) 12%, transparent); color: var(--color-on-dark); }
.sidebar-badge { margin-left: auto; padding: 0 var(--spacing-xs); border-radius: var(--rounded-full); background: var(--color-brand-red-dark); color: var(--color-on-dark); }
.sidebar-footer { padding: var(--spacing-xl); border-top: 1px solid color-mix(in srgb, var(--color-on-dark) 12%, transparent); margin-top: var(--spacing-xl); }
</style>
