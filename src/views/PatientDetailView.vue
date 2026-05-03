<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRoute, useRouter } from 'vue-router'
import { Activity, CalendarDays, FileClock, FilePenLine, Pill, ShieldAlert } from 'lucide-vue-next'
import AdverseEventList from '#/components/patient/AdverseEventList.vue'
import AppointmentTimeline from '#/components/patient/AppointmentTimeline.vue'
import DispensingTable from '#/components/patient/DispensingTable.vue'
import DoseCalculatorPanel from '#/components/patient/DoseCalculatorPanel.vue'
import InrTrendChart from '#/components/patient/InrTrendChart.vue'
import StatusChangeModal from '#/components/patient/StatusChangeModal.vue'
import VisitList from '#/components/patient/VisitList.vue'
import TtrBadge from '#/components/active/TtrBadge.vue'
import StatusBadge from '#/components/shared/StatusBadge.vue'
import VisitFormPanel from '#/components/visit/VisitFormPanel.vue'
import type { PatientDetail } from '#/types/patient'
import type { WfVisit } from '#/types/visit'
import { calculateAge, formatThaiDate, patientFullName, sexLabel } from '#/utils/clinic'

const route = useRoute()
const router = useRouter()
const hn = route.params.hn as string

type TabKey = 'inr' | 'visits' | 'dispensing' | 'appointments' | 'adverse'
const activeTab = ref<TabKey>('inr')
const tabs: { key: TabKey; label: string; icon: unknown }[] = [
  { key: 'inr', label: 'INR', icon: Activity },
  { key: 'visits', label: 'ประวัติการทำคลินิก', icon: FileClock },
  { key: 'dispensing', label: 'ประวัติยา', icon: Pill },
  { key: 'appointments', label: 'นัดหมาย', icon: CalendarDays },
  { key: 'adverse', label: 'เหตุการณ์', icon: ShieldAlert },
]

const patientDetail = ref<PatientDetail | null>(null)
const visits = ref<WfVisit[]>([])
const ttr = ref<number | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const visitPanelOpen = ref(false)
const statusModalOpen = ref(false)

async function loadPatient() {
  loading.value = true
  error.value = null
  try {
    const [detail, visitList, ttrValue] = await Promise.all([
      invoke<PatientDetail>('get_patient_detail', { hn }),
      invoke<WfVisit[]>('get_visit_history', { hn }),
      invoke<number | null>('calculate_ttr', { hn, windowDays: 180 }),
    ])
    patientDetail.value = detail
    visits.value = visitList
    ttr.value = ttrValue
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

const age = computed(() => calculateAge(patientDetail.value?.hosxpInfo?.birthday))
const fullName = computed(() => patientFullName(patientDetail.value?.hosxpInfo))

async function onVisitSaved(visitId: number) {
  await loadPatient()
  void router.push(`/slip/${visitId}`)
}

onMounted(() => { void loadPatient() })
</script>

<template>
  <div class="patient-detail">
    <div v-if="loading" class="body-sm" style="color: var(--color-stone); padding: var(--spacing-xxl)">
      กำลังโหลด...
    </div>
    <div v-else-if="error" class="card card-feature-coral body-sm" style="padding: var(--spacing-md)">
      {{ error }}
    </div>

    <template v-else-if="patientDetail">
      <section class="card header-card">
        <div class="header-main">
          <div>
            <h2 class="h3">{{ fullName }}</h2>
            <p class="body-sm header-meta">
              HN: {{ hn }} &nbsp;|&nbsp;
              อายุ: {{ age ?? '-' }} ปี &nbsp;|&nbsp;
              {{ sexLabel(patientDetail.hosxpInfo?.sex) }} &nbsp;|&nbsp;
              {{ patientDetail.hosxpInfo?.phone || '-' }}
            </p>
          </div>
          <div class="header-badges">
            <StatusBadge :status="patientDetail.patient.status" />
            <TtrBadge :ttr="ttr" />
          </div>
        </div>

        <div class="header-grid">
          <div class="header-item">
            <span class="caption header-label">ข้อบ่งชี้</span>
            <span class="body-sm-medium">{{ patientDetail.patient.indication || '-' }}</span>
          </div>
          <div class="header-item">
            <span class="caption header-label">เป้าหมาย INR</span>
            <span class="body-sm-medium">
              {{ patientDetail.patient.targetInrLow.toFixed(1) }}–{{ patientDetail.patient.targetInrHigh.toFixed(1) }}
            </span>
          </div>
          <div class="header-item">
            <span class="caption header-label">ลงทะเบียน</span>
            <span class="body-sm-medium">{{ formatThaiDate(patientDetail.patient.enrolledAt) }}</span>
          </div>
          <div class="header-item">
            <span class="caption header-label">ที่อยู่</span>
            <span class="body-sm-medium">{{ patientDetail.hosxpInfo?.addrpart || '-' }}</span>
          </div>
        </div>

        <div class="header-actions">
          <button type="button" class="btn btn-secondary" @click="statusModalOpen = true">
            เปลี่ยนสถานะ
          </button>
          <button type="button" class="btn btn-primary" @click="visitPanelOpen = true">
            <FilePenLine :size="16" /> + บันทึกการทำคลินิก
          </button>
        </div>
      </section>

      <div class="tab-bar">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          :class="['tab-pill', { active: activeTab === tab.key }]"
          @click="activeTab = tab.key"
        >
          <component :is="tab.icon" :size="14" />
          {{ tab.label }}
        </button>
      </div>

      <div class="tab-content">
        <template v-if="activeTab === 'inr'">
          <InrTrendChart
            :inr-records="patientDetail.inrHistory ?? []"
            :target-low="patientDetail.patient.targetInrLow"
            :target-high="patientDetail.patient.targetInrHigh"
          />
          <DoseCalculatorPanel
            :hn="hn"
            :target-low="patientDetail.patient.targetInrLow"
            :target-high="patientDetail.patient.targetInrHigh"
          />
        </template>

        <VisitList v-else-if="activeTab === 'visits'" :visits="visits" :hn="hn" />

        <DispensingTable
          v-else-if="activeTab === 'dispensing'"
          :records="patientDetail.dispensingHistory ?? []"
        />

        <AppointmentTimeline v-else-if="activeTab === 'appointments'" :hn="hn" />

        <AdverseEventList v-else-if="activeTab === 'adverse'" :hn="hn" />
      </div>
    </template>

    <div v-else class="card">
      <p class="body-sm">ไม่พบข้อมูลผู้ป่วย HN: {{ hn }}</p>
    </div>

    <VisitFormPanel v-model="visitPanelOpen" :hn="hn" @saved="onVisitSaved" />

    <StatusChangeModal
      v-if="statusModalOpen && patientDetail"
      v-model="statusModalOpen"
      :hn="hn"
      :current-status="patientDetail.patient.status"
      @saved="loadPatient"
    />
  </div>
</template>

<style scoped>
.patient-detail { display: flex; flex-direction: column; gap: var(--spacing-xl); }
.header-card { display: flex; flex-direction: column; gap: var(--spacing-lg); }
.header-main { display: flex; justify-content: space-between; align-items: flex-start; gap: var(--spacing-md); }
.header-meta { color: var(--color-slate); margin-top: var(--spacing-xs); }
.header-badges { display: flex; align-items: center; gap: var(--spacing-sm); flex-shrink: 0; }
.header-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: var(--spacing-md); }
.header-item { display: flex; flex-direction: column; gap: 2px; }
.header-label { color: var(--color-slate); }
.header-actions { display: flex; gap: var(--spacing-md); justify-content: flex-end; }
.tab-bar { display: flex; gap: var(--spacing-xs); flex-wrap: wrap; }
.tab-pill {
  display: flex; align-items: center; gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-lg);
  border-radius: var(--rounded-full);
  border: 1px solid var(--color-hairline);
  background: transparent;
  cursor: pointer; font-size: 0.875rem; color: var(--color-slate);
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}
.tab-pill:hover { background: var(--color-surface-raised); }
.tab-pill.active { background: var(--color-primary); color: var(--color-on-primary); border-color: var(--color-primary); }
.tab-content { display: flex; flex-direction: column; gap: var(--spacing-xl); }
</style>
