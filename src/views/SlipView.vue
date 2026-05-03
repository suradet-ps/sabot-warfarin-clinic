<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRoute } from 'vue-router'
import { Printer } from 'lucide-vue-next'
import PhysicianSlip from '#/components/slip/PhysicianSlip.vue'
import type { InrRecord } from '#/types/inr'
import type { PatientDetail } from '#/types/patient'
import type { WfVisit } from '#/types/visit'

const route = useRoute()
const visitId = computed(() => Number(route.params.visitId))
const visit = ref<WfVisit | null>(null)
const patient = ref<PatientDetail | null>(null)
const ttr = ref<number | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

async function loadSlip() {
  if (!Number.isInteger(visitId.value) || visitId.value <= 0) {
    visit.value = null
    patient.value = null
    ttr.value = null
    error.value = 'visit id ไม่ถูกต้อง'
    loading.value = false
    return
  }

  loading.value = true
  error.value = null
  visit.value = null
  patient.value = null
  ttr.value = null
  try {
    visit.value = await invoke<WfVisit>('get_visit_by_id', { visitId: visitId.value })
    const [patientDetail, inrHistory, ttrValue] = await Promise.all([
      invoke<PatientDetail>('get_patient_detail', { hn: visit.value.hn }),
      invoke<InrRecord[]>('get_inr_history', { hn: visit.value.hn }),
      invoke<number | null>('calculate_ttr', { hn: visit.value.hn, windowDays: 182 }),
    ])
    patient.value = { ...patientDetail, inrHistory }
    ttr.value = ttrValue
  } catch (invokeError) {
    error.value = String(invokeError)
  } finally {
    loading.value = false
  }
}

function printSlip() {
  window.print()
}

watch(visitId, () => {
  void loadSlip()
}, { immediate: true })
</script>

<template>
  <div class="slip-view">
    <div class="slip-toolbar">
      <button type="button" class="btn btn-primary print-button" @click="printSlip"><Printer :size="16" /></button>
    </div>
    <div v-if="loading" class="card loading-card body-sm" style="padding: var(--spacing-xxl)">กำลังโหลด...</div>
    <div v-else-if="error" class="card card-feature-coral">{{ error }}</div>
    <PhysicianSlip v-else-if="visit && patient" :visit="(visit as WfVisit)" :patient="(patient as PatientDetail)" :ttr="ttr" />
  </div>
</template>

<style scoped>
.slip-view {
  max-width: 56rem;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.slip-toolbar {
  display: flex;
  justify-content: flex-end;
}

.loading-card {
  text-align: center;
  color: var(--color-slate);
}

@media print {
  .slip-toolbar {
    display: none;
  }
}
</style>
