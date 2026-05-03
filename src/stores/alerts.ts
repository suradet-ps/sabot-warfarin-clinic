import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PatientAlert } from '#/types/alert'

export const useAlertStore = defineStore('alerts', () => {
  const alerts = ref<PatientAlert[]>([])
  const loading = ref(false)

  const criticalCount = computed(() => alerts.value.filter(a => a.severity === 'critical').length)
  const warningCount = computed(() => alerts.value.filter(a => a.severity === 'warning').length)

  async function fetchAlerts() {
    loading.value = true
    try {
      alerts.value = await invoke<PatientAlert[]>('get_patient_alerts')
    } catch {
      alerts.value = []
    } finally {
      loading.value = false
    }
  }

  function getAlertsForPatient(hn: string) {
    return alerts.value.filter(a => a.hn === hn)
  }

  return { alerts, loading, criticalCount, warningCount, fetchAlerts, getAlertsForPatient }
})
