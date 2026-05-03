<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRouter } from 'vue-router'
import { AlertTriangle, FilePenLine, Users } from 'lucide-vue-next'
import PatientRow from '#/components/active/PatientRow.vue'
import VisitFormPanel from '#/components/visit/VisitFormPanel.vue'
import { useAlertStore } from '#/stores/alerts'
import type { ActivePatientSummary } from '#/types/patient'

const alertStore = useAlertStore()
const router = useRouter()
const visitPanelOpen = ref(false)
const selectedHn = ref<string>('')
const summaries = ref<ActivePatientSummary[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

const criticalAlerts = computed(() => alertStore.alerts.filter((a) => a.severity === 'critical'))

async function loadRows() {
  loading.value = true
  error.value = null
  try {
    summaries.value = await invoke<ActivePatientSummary[]>('get_active_patient_summaries')
    if (!selectedHn.value) {
      selectedHn.value = summaries.value[0]?.patient.hn ?? ''
    }
    void alertStore.fetchAlerts()
  } catch (invokeError) {
    error.value = String(invokeError)
  } finally {
    loading.value = false
  }
}

function openVisit(hn: string) {
  selectedHn.value = hn
  visitPanelOpen.value = true
}

async function handleSaved(visitId: number) {
  visitPanelOpen.value = false
  await loadRows()
  await router.push(`/slip/${visitId}`)
}

onMounted(() => {
  void loadRows()
})
</script>

<template>
  <div class="active-view">
    <div v-if="criticalAlerts.length" class="card card-feature-coral alert-banner">
      <div class="alert-banner-head">
        <div>
          <h2 class="heading-md">แจ้งเตือนวิกฤต {{ criticalAlerts.length }} รายการ</h2>
          <p class="body-sm">{{ criticalAlerts[0]?.message }}</p>
        </div>
        <AlertTriangle :size="20" />
      </div>
    </div>

    <div class="page-toolbar">
      <div class="stat-row">
        <div class="stat-chip card">
          <Users :size="16" class="stat-icon" />
          <span class="body-sm">ผู้ป่วย <strong>{{ summaries.length }}</strong> ราย</span>
        </div>
      </div>
      <button class="btn btn-primary" :disabled="!summaries.length" @click="openVisit(selectedHn || summaries[0]?.patient.hn || '')">
        <FilePenLine :size="16" /> บันทึกการเยี่ยม
      </button>
    </div>

    <div v-if="loading" class="card loading-state body-sm">กำลังโหลด...</div>
    <div v-else-if="error" class="card card-feature-coral">{{ error }}</div>
    <div v-else class="table-wrap card">
      <table class="table">
        <thead>
          <tr>
            <th>HN / ชื่อ-นามสกุล</th>
            <th>INR ล่าสุด</th>
            <th>ขนาดยา (mg/วัน)</th>
            <th>TTR</th>
            <th>นัดต่อไป</th>
            <th>การแจ้งเตือน</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="summaries.length === 0">
            <td colspan="7" class="empty-cell">ยังไม่มีผู้ป่วยในคลินิก</td>
          </tr>
          <PatientRow
            v-for="summary in summaries"
            :key="summary.patient.hn"
            :summary="summary"
            :alerts="alertStore.getAlertsForPatient(summary.patient.hn)"
            @open-visit="openVisit"
          />
        </tbody>
      </table>
    </div>

    <VisitFormPanel v-if="selectedHn" v-model="visitPanelOpen" :hn="selectedHn" @saved="handleSaved" />
  </div>
</template>

<style scoped>
.active-view {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xl);
}
.alert-banner-head,
.page-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--spacing-md);
}
.stat-row { display: flex; gap: var(--spacing-md); }
.stat-chip { display: flex; align-items: center; gap: var(--spacing-xs); }
.stat-icon { color: var(--color-slate); }
.loading-state, .empty-cell {
  padding: var(--spacing-xxl);
  text-align: center;
  color: var(--color-slate);
}
.table-wrap { padding: 0; overflow-x: auto; }
.table-wrap .table { min-width: 980px; }
</style>
