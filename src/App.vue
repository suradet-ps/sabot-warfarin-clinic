<script setup lang="ts">
import { onMounted } from 'vue'
import AppSidebar from '#/components/layout/AppSidebar.vue'
import AppHeader from '#/components/layout/AppHeader.vue'
import { useSyncStore } from '#/stores/sync'

const syncStore = useSyncStore()

onMounted(async () => {
  console.log('[App] mounted')

  setTimeout(() => {
    const splash = document.getElementById('splash-overlay')
    if (splash) {
      splash.classList.add('splash-fade-out')
      setTimeout(() => splash.remove(), 300)
    }
  }, 1500)

  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const mainWindow = getCurrentWindow()
    await mainWindow.show()
    await mainWindow.setFocus()
  } catch (e) {
    console.warn('Window show skipped:', e)
  }

  try {
    await syncStore.refreshAll()
  } catch (error) {
    console.error('Failed to refresh sync status:', error)
  }
  syncStore.startAutoSync()
})
</script>

<template>
  <div class="app-shell">
    <AppSidebar />
    <div class="app-main">
      <AppHeader />
      <main class="app-content">
        <RouterView />
      </main>
    </div>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  height: 100vh;
  overflow: hidden;
  background: var(--color-canvas);
}
.app-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.app-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-xl);
}
</style>
