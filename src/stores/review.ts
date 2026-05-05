import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useReviewStore = defineStore('review', () => {
  const pendingCount = ref(0)

  async function fetchPendingCount() {
    try {
      pendingCount.value = await invoke<number>('get_pending_review_count')
    } catch (e) {
      console.error('failed to fetch pending review count', e)
    }
  }

  return {
    pendingCount,
    fetchPendingCount,
  }
})