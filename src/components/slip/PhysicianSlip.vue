<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useSettingsStore } from '#/stores/settings'
import type { PatientDetail } from '#/types/patient'
import type { WfVisit } from '#/types/visit'
import { calculateAge, doseDayKeys, doseDayLabels, formatThaiDate, normalizeDoseSchedule } from '#/utils/clinic'

const props = defineProps<{ visit: WfVisit; patient: PatientDetail; ttr: number | null }>()

const settingsStore = useSettingsStore()
const hospitalName = ref('โรงพยาบาลสระโบสถ์')

onMounted(() => {
  void settingsStore.loadSettings()
  hospitalName.value = settingsStore.hospitalName || hospitalName.value
})

const info = computed(() => props.patient.hosxpInfo)
const p = computed(() => props.patient.patient)
const age = computed(() => (info.value ? calculateAge(info.value.birthday) : null))
const currentDoseSchedule = computed(() => normalizeDoseSchedule(props.visit.doseDetail))
const newDoseSchedule = computed(() => normalizeDoseSchedule(props.visit.newDoseDetail))
const last3Inr = computed(() => [...(props.patient.inrHistory ?? [])].sort((a, b) => b.date.localeCompare(a.date)).slice(0, 3))

const adherenceLabel: Record<string, string> = {
  good: 'ดี',
  fair: 'พอใช้',
  poor: 'ไม่ดี',
}

function ttrClass(v: number | null): string {
  if (v === null) return ''
  if (v >= 65) return 'ttr-good'
  if (v >= 50) return 'ttr-warn'
  return 'ttr-bad'
}
</script>

<template>
  <div class="slip-sheet card">
    <div class="slip-header">
      <div>
        <strong class="h5">คลินิกวาร์ฟาริน</strong>
        <p class="caption" style="color: var(--color-slate)">{{ hospitalName }}</p>
      </div>
      <p class="caption" style="color: var(--color-slate)">{{ formatThaiDate(visit.visitDate) }}</p>
    </div>

    <div class="slip-divider" />

    <div class="slip-patient-grid">
      <div><span class="caption label">ชื่อ-สกุล</span><span class="body-sm">{{ [info?.pname, info?.fname, info?.lname].filter(Boolean).join(' ') || '-' }}</span></div>
      <div><span class="caption label">HN</span><span class="body-sm">{{ patient.patient.hn }}</span></div>
      <div><span class="caption label">อายุ</span><span class="body-sm">{{ age ?? '-' }} ปี</span></div>
      <div><span class="caption label">เพศ</span><span class="body-sm">{{ info?.sex === 'M' ? 'ชาย' : info?.sex === 'F' ? 'หญิง' : '-' }}</span></div>
      <div><span class="caption label">ข้อบ่งชี้</span><span class="body-sm">{{ p.indication || '-' }}</span></div>
      <div><span class="caption label">เป้าหมาย INR</span><span class="body-sm">{{ p.targetInrLow.toFixed(1) }}–{{ p.targetInrHigh.toFixed(1) }}</span></div>
    </div>

    <div class="slip-divider" />

    <div class="inr-display">
      <span class="caption label">ผล INR วันนี้</span>
      <span class="inr-big">{{ visit.inrValue?.toFixed(1) ?? '-' }}</span>
    </div>

    <div v-if="last3Inr.length" class="inr-history">
      <span class="caption label">ย้อนหลัง 3 ครั้ง:</span>
      <div class="inr-hist-row">
        <span v-for="r in last3Inr" :key="r.date" class="caption">
          {{ formatThaiDate(r.date) }}: <strong>{{ r.value.toFixed(1) }}</strong>
        </span>
      </div>
    </div>

    <div class="slip-divider" />

    <div class="dose-section">
      <p class="caption label">ขนาดยาเดิม: {{ visit.currentDoseMgday?.toFixed(1) ?? '-' }} mg/วัน</p>
      <div class="dose-grid">
        <div v-for="k in doseDayKeys" :key="k" class="dose-chip">
          <span class="caption">{{ doseDayLabels[k] }}</span>
          <span class="body-sm">{{ currentDoseSchedule[k] }}</span>
        </div>
      </div>
      <p class="caption label" style="margin-top: var(--spacing-sm)">ขนาดยาใหม่ที่แนะนำ: {{ visit.newDoseMgday?.toFixed(1) ?? '-' }} mg/วัน</p>
      <div class="dose-grid">
        <div v-for="k in doseDayKeys" :key="k" class="dose-chip">
          <span class="caption">{{ doseDayLabels[k] }}</span>
          <span class="body-sm-medium">{{ newDoseSchedule[k] }}</span>
        </div>
      </div>
    </div>

    <div class="slip-divider" />

    <div class="slip-stats-grid">
      <div>
        <span class="caption label">TTR (6 เดือน)</span>
        <span :class="['body-sm-medium', ttrClass(ttr)]">{{ ttr != null ? ttr.toFixed(0) + '%' : '-' }}</span>
      </div>
      <div>
        <span class="caption label">การรับประทานยา</span>
        <span class="body-sm">{{ adherenceLabel[visit.adherence ?? ''] ?? '-' }}</span>
      </div>
    </div>

    <div class="slip-divider" />

    <div class="slip-appt-grid">
      <div>
        <span class="caption label">นัดครั้งต่อไป</span>
        <span class="body-sm">{{ formatThaiDate(visit.nextAppointment) }}</span>
      </div>
      <div>
        <span class="caption label">ตรวจ INR ครั้งต่อไป</span>
        <span class="body-sm">{{ formatThaiDate(visit.nextInrDue) }}</span>
      </div>
    </div>

    <div class="signature-section">
      <div class="signature-box">
        <span class="caption label">หมายเหตุ/คำแนะนำ</span>
        <div class="signature-line" />
        <div class="signature-line" />
      </div>
      <div class="signature-box">
        <span class="caption label">แพทย์ผู้สั่งยา</span>
        <div class="signature-line" style="margin-top: var(--spacing-xl)" />
        <p class="caption" style="color: var(--color-stone)">{{ visit.physician || '_______________' }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.slip-sheet { max-width: 560px; margin: 0 auto; display: flex; flex-direction: column; gap: var(--spacing-md); padding: var(--spacing-xxl); }
.slip-header { display: flex; justify-content: space-between; align-items: flex-start; }
.slip-divider { height: 1px; background: var(--color-hairline); margin: var(--spacing-xs) 0; }
.slip-patient-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--spacing-xs) var(--spacing-xl); }
.slip-patient-grid > div { display: flex; flex-direction: column; gap: 2px; }
.label { color: var(--color-slate); display: block; }
.inr-display { display: flex; flex-direction: column; gap: var(--spacing-xs); }
.inr-big { font-size: 2.5rem; font-weight: 700; line-height: 1; color: var(--color-primary); }
.inr-history { display: flex; flex-direction: column; gap: var(--spacing-xs); }
.inr-hist-row { display: flex; gap: var(--spacing-lg); flex-wrap: wrap; }
.dose-section { display: flex; flex-direction: column; gap: var(--spacing-xs); }
.dose-grid { display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); gap: var(--spacing-xs); margin-top: var(--spacing-xs); }
.dose-chip { display: flex; flex-direction: column; align-items: center; padding: var(--spacing-xs); border: 1px solid var(--color-hairline-soft); border-radius: var(--rounded-md); gap: 2px; }
.slip-stats-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--spacing-md); }
.slip-stats-grid > div { display: flex; flex-direction: column; gap: 2px; }
.slip-appt-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--spacing-md); }
.slip-appt-grid > div { display: flex; flex-direction: column; gap: 2px; }
.signature-section { display: grid; grid-template-columns: 1fr 1fr; gap: var(--spacing-xl); margin-top: var(--spacing-md); }
.signature-box { display: flex; flex-direction: column; gap: var(--spacing-sm); padding: var(--spacing-sm); border: 1px solid var(--color-hairline-soft); border-radius: var(--rounded-md); }
.signature-line { height: 1px; background: var(--color-hairline); margin: var(--spacing-lg) 0 var(--spacing-xs); }
.ttr-good { color: var(--color-success-accent); }
.ttr-warn { color: var(--color-brand-coral); }
.ttr-bad { color: var(--color-brand-red-dark); }
@media print {
  .slip-sheet { border-color: var(--color-hairline); border-radius: 0; box-shadow: none; color: var(--color-ink); background: var(--color-canvas); }
}
</style>
