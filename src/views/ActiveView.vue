<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRouter } from 'vue-router'
import { AlertTriangle, Search, Users } from 'lucide-vue-next'
import PatientRow from '#/components/active/PatientRow.vue'
import VisitFormPanel from '#/components/visit/VisitFormPanel.vue'
import { useAlertStore } from '#/stores/alerts'
import { useReviewStore } from '#/stores/review'
import type { ActivePatientSummary } from '#/types/patient'

const alertStore = useAlertStore()
const reviewStore = useReviewStore()
const router = useRouter()
const visitPanelOpen = ref(false)
const selectedHn = ref<string>('')
const summaries = ref<ActivePatientSummary[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const searchQuery = ref('')

const criticalAlerts = computed(() => alertStore.alerts.filter((a) => a.severity === 'critical'))

const filteredSummaries = computed(() => {
  if (!searchQuery.value.trim()) return summaries.value
  const query = searchQuery.value.toLowerCase()
  return summaries.value.filter((s) => {
    const hn = s.patient.hn.toLowerCase()
    const fname = s.hosxpInfo?.fname?.toLowerCase() ?? ''
    const lname = s.hosxpInfo?.lname?.toLowerCase() ?? ''
    return hn.includes(query) || fname.includes(query) || lname.includes(query)
  })
})

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
  void reviewStore.fetchPendingCount()
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
          <span class="body-sm">ผู้ป่วย <strong>{{ filteredSummaries.length }}</strong> ราย</span>
        </div>
      </div>
      <div class="search-box">
        <Search :size="16" class="search-icon" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="ค้นหา HN, ชื่อ, สกุล"
          class="search-input"
        />
      </div>
    </div>

    <div v-if="loading" class="card loading-state body-sm">กำลังโหลด...</div>
    <div v-else-if="error" class="card card-feature-coral">{{ error }}</div>
    <div v-else class="table-wrap card">
      <table class="table">
        <thead>
          <tr>
            <th>HN / ชื่อ-นามสกุล</th>
            <th>INR ล่าสุด</th>
            <th>ขนาดยา (mg/สัปดาห์)</th>
            <th>TTR</th>
            <th>นัดต่อไป</th>
            <th>การแจ้งเตือน</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="filteredSummaries.length === 0">
            <td colspan="7" class="empty-cell">{{ searchQuery ? 'ไม่พบผู้ป่วยที่ค้นหา' : 'ยังไม่มีผู้ป่วยในคลินิก' }}</td>
          </tr>
          <PatientRow
            v-for="summary in filteredSummaries"
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
.search-box { display: flex; align-items: center; gap: var(--spacing-xs); background: var(--color-canvas); border: 1px solid var(--color-hairline-soft); border-radius: var(--rounded-md); padding: var(--spacing-sm) var(--spacing-md); }
.search-icon { color: var(--color-stone); flex-shrink: 0; }
.search-input { border: none; outline: none; background: transparent; font-size: var(--typography-body-sm-size); color: var(--color-ink); width: 200px; }
.search-input::placeholder { color: var(--color-stone); }
.loading-state, .empty-cell {
  padding: var(--spacing-xxl);
  text-align: center;
  color: var(--color-slate);
}
.table-wrap { padding: 0; overflow-x: auto; }
.table-wrap .table { min-width: 980px; }
</style>
