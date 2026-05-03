<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { X } from 'lucide-vue-next'
import DayDoseTable from '#/components/visit/DayDoseTable.vue'
import DoseOptionsPanel from '#/components/visit/DoseOptionsPanel.vue'
import type { DispensingRecord } from '#/types/dispensing'
import type { InrRecord } from '#/types/inr'
import type { PatientDetail } from '#/types/patient'
import type { DoseSuggestion, VisitInput, WfVisit } from '#/types/visit'
import type { RegimenOption, AvailablePills } from '@/types/dose'
import { useDoseCalculator } from '@/composables/useDoseCalculator'
import {
  aggregateDispensingByVisit,
  dateInputToday,
  formatThaiDate,
  emptyDoseSchedule,
  mergeDoseSchedules,
  normalizeDoseSchedule,
  scheduleAverageDose,
  scheduleWeeklyTotal,
} from '#/utils/clinic'

const { generateDoseOptions, DEFAULT_AVAILABLE_PILLS } = useDoseCalculator()

const props = defineProps<{ hn: string }>()
const modelValue = defineModel<boolean>({ default: false })
const emit = defineEmits<{ (e: 'saved', visitId: number): void }>()

const saving = ref(false)
const error = ref<string | null>(null)
const loadingSuggestion = ref(false)

const visitDate = ref(dateInputToday())
const inrValue = ref<number | null>(null)
const inrSource = ref<'lab_order' | 'lab_app_order' | 'manual'>('lab_order')
const currentDoseMgday = ref<number | null>(null)
const currentDoseDetail = ref(emptyDoseSchedule())
const newDoseMgday = ref<number | null>(null)
const newDoseDetail = ref(emptyDoseSchedule())
const nextAppointment = ref('')
const nextInrDue = ref('')
const physician = ref('')
const adherence = ref<'good' | 'fair' | 'poor'>('good')
const notes = ref('')
const suggestion = ref<DoseSuggestion | null>(null)
const targetLow = ref(2.0)
const targetHigh = ref(3.0)
const latestHosxpDispense = ref<DispensingRecord | null>(null)
const latestHosxpVisit = ref<ReturnType<typeof aggregateDispensingByVisit>[number] | null>(null)
const currentDoseSource = ref<'visit' | 'hosxp' | 'manual'>('manual')
const currentDoseSourceText = ref('')
const doseOptions = ref<RegimenOption[]>([])
const selectedDoseOptionIndex = ref<number | null>(null)
const loadingDoseOptions = ref(false)
const doseOptionsError = ref<string | null>(null)
const availablePills = ref<AvailablePills>({ ...DEFAULT_AVAILABLE_PILLS })
const allowHalf = ref(true)
const specialDayPattern = ref<'fri-sun' | 'mon-wed-fri'>('fri-sun')

const sideEffectOptions = [
  { key: 'bleeding_gums', label: 'เหงือกเลือดออก' },
  { key: 'bruising', label: 'เลือดออกใต้ผิว' },
  { key: 'blood_urine', label: 'เลือดออกในปัสสาวะ' },
  { key: 'blood_stool', label: 'เลือดออกในอุจจาระ' },
  { key: 'nausea', label: 'คลื่นไส้' },
  { key: 'hair_loss', label: 'ผมร่วง' },
  { key: 'other', label: 'อื่นๆ' },
]
const selectedSideEffects = ref<string[]>([])

async function loadDefaults() {
  try {
    const [latestInr, visits, patientData] = await Promise.all([
      invoke<InrRecord | null>('get_latest_inr', { hn: props.hn }),
      invoke<WfVisit[]>('get_visit_history', { hn: props.hn }),
      invoke<PatientDetail>('get_patient_detail', { hn: props.hn }),
    ])
    if (latestInr) {
      inrValue.value = latestInr.value
      inrSource.value = latestInr.source as typeof inrSource.value ?? 'lab_order'
    }
    const lastVisit = visits[0]
    const aggregatedVisits = aggregateDispensingByVisit(patientData.dispensingHistory ?? [])
    latestHosxpVisit.value = aggregatedVisits.find((visit) => visit.mgPerWeek > 0) ?? null
    latestHosxpDispense.value = latestHosxpVisit.value?.items[0] ?? null
    if (lastVisit) {
      currentDoseMgday.value = lastVisit.newDoseMgday ?? lastVisit.currentDoseMgday ?? null
      currentDoseDetail.value = normalizeDoseSchedule(lastVisit.newDoseDetail ?? lastVisit.doseDetail)
      currentDoseSource.value = 'visit'
      currentDoseSourceText.value = 'ใช้ขนาดยาจาก visit ล่าสุดที่บันทึกในคลินิก'
    } else if (latestHosxpVisit.value) {
      applyHosxpDose(latestHosxpVisit.value)
    } else {
      currentDoseSource.value = 'manual'
      currentDoseSourceText.value = 'ไม่พบขนาดยาเดิมที่คำนวณได้จากระบบ กรุณากรอกเอง'
    }
    targetLow.value = patientData.patient.targetInrLow
    targetHigh.value = patientData.patient.targetInrHigh
  } catch {
    // non-critical
  }
}

function applyHosxpDose(visit: NonNullable<typeof latestHosxpVisit.value>) {
  currentDoseDetail.value = mergeDoseSchedules(visit.combinedSchedule)
  currentDoseMgday.value = visit.mgPerDayAverage
  currentDoseSource.value = 'hosxp'
  currentDoseSourceText.value = `ดึงจาก HosXP วันที่ ${formatThaiDate(visit.vstdate)}${visit.usageTextSummary !== '-' ? `: ${visit.usageTextSummary}` : ''}`
}

async function fetchSuggestion() {
  if (currentDoseMgday.value === null || inrValue.value === null) return
  loadingSuggestion.value = true
  loadingDoseOptions.value = true
  doseOptionsError.value = null
  doseOptions.value = []
  selectedDoseOptionIndex.value = null

  try {
    suggestion.value = await invoke<DoseSuggestion>('suggest_dose', {
      currentDose: currentDoseMgday.value,
      currentInr: inrValue.value,
      targetLow: targetLow.value,
      targetHigh: targetHigh.value,
    })
    if (suggestion.value) {
      newDoseMgday.value = suggestion.value.suggestedDoseMgday

      const weeklyDose = suggestion.value.suggestedDoseMgday * 7
      const daysUntilAppointment = nextAppointment.value
        ? Math.max(1, Math.ceil((new Date(nextAppointment.value).getTime() - new Date().getTime()) / (1000 * 60 * 60 * 24)))
        : 28
      const startDayOfWeek = new Date().getDay()

      const options = await generateDoseOptions(
        weeklyDose,
        availablePills.value,
        allowHalf.value,
        specialDayPattern.value,
        daysUntilAppointment,
        startDayOfWeek
      )
      doseOptions.value = options

      if (options.length === 0) {
        doseOptionsError.value = 'ไม่พบตัวเลือกที่เหมาะสม ลองปรับการตั้งค่ายา'
      }
    }
  } catch (e) {
    doseOptionsError.value = String(e)
  } finally {
    loadingSuggestion.value = false
    loadingDoseOptions.value = false
  }
}

function handleSelectDoseOption(index: number) {
  selectedDoseOptionIndex.value = index
  const option = doseOptions.value[index]

  const dayMap: Record<number, string> = {
    0: 'mon', 1: 'tue', 2: 'wed', 3: 'thu', 4: 'fri', 5: 'sat', 6: 'sun',
  }

  const newDetail = emptyDoseSchedule()
  for (const day of option.weekly_schedule) {
    const dayKey = dayMap[day.day_index]
    if (dayKey && dayKey in newDetail) {
      newDetail[dayKey as keyof typeof newDetail] = day.is_stop_day ? 0 : day.total_dose
    }
  }

  newDoseDetail.value = newDetail
  newDoseMgday.value = option.weekly_dose_actual / 7
}

const currentDoseAvg = computed(() => scheduleAverageDose(currentDoseDetail.value))
const currentDoseWeek = computed(() => scheduleWeeklyTotal(currentDoseDetail.value))
const newDoseAvg = computed(() => scheduleAverageDose(newDoseDetail.value))
const newDoseWeek = computed(() => scheduleWeeklyTotal(newDoseDetail.value))

watch(currentDoseDetail, () => {
  currentDoseMgday.value = currentDoseAvg.value
}, { deep: true })
watch(newDoseDetail, () => {
  newDoseMgday.value = newDoseAvg.value
}, { deep: true })

async function handleSubmit() {
  saving.value = true
  error.value = null
  try {
    const input: VisitInput = {
      hn: props.hn,
      visitDate: visitDate.value,
      inrValue: inrValue.value ?? undefined,
      inrSource: inrSource.value,
      currentDoseMgday: currentDoseMgday.value ?? undefined,
      doseDetail: currentDoseDetail.value,
      newDoseMgday: newDoseMgday.value ?? undefined,
      newDoseDetail: newDoseDetail.value,
      doseChanged: newDoseMgday.value !== currentDoseMgday.value,
      nextAppointment: nextAppointment.value || undefined,
      nextInrDue: nextInrDue.value || undefined,
      physician: physician.value || undefined,
      adherence: adherence.value,
      sideEffects: selectedSideEffects.value,
      notes: notes.value || undefined,
    }
    const visitId = await invoke<number>('save_visit', { visit: input })
    emit('saved', visitId)
    modelValue.value = false
  } catch (e) {
    error.value = String(e)
  } finally {
    saving.value = false
  }
}

watch(() => modelValue.value, (open) => {
  if (open) void loadDefaults()
})
onMounted(() => { if (modelValue.value) void loadDefaults() })
</script>

<template>
  <Teleport to="body">
    <div v-if="modelValue" class="panel-overlay" @click.self="modelValue = false">
      <div class="visit-panel card">
        <div class="panel-header">
          <h3 class="h4">บันทึกการเยี่ยม</h3>
          <button class="btn btn-ghost" @click="modelValue = false"><X :size="18" /></button>
        </div>

        <div v-if="error" class="card card-feature-coral body-sm" style="padding: var(--spacing-md)">{{ error }}</div>

        <div class="panel-body">
          <div class="form-row">
            <label class="form-field">
              <span class="caption label">วันที่</span>
              <input class="input" type="date" v-model="visitDate" />
            </label>
            <label class="form-field">
              <span class="caption label">ค่า INR</span>
              <input class="input" type="number" step="0.1" v-model.number="inrValue" />
            </label>
          </div>

          <div class="form-section">
            <p class="caption label">ตารางยาเดิม (mg/วัน)</p>
            <div class="dose-source-card">
              <div>
                <p class="body-sm-medium">{{ currentDoseSource === 'hosxp' ? 'ขนาดยาเดิมจาก HosXP' : currentDoseSource === 'visit' ? 'ขนาดยาเดิมจาก visit ล่าสุด' : 'กรอกขนาดยาเดิมเอง' }}</p>
                <p class="caption dose-source-text">{{ currentDoseSourceText }}</p>
              </div>
              <button
                v-if="latestHosxpVisit && currentDoseSource !== 'hosxp'"
                type="button"
                class="btn btn-secondary btn-compact"
                @click="applyHosxpDose(latestHosxpVisit)"
              >
                ใช้ค่าจาก HosXP ล่าสุด
              </button>
            </div>
            <DayDoseTable v-model="currentDoseDetail" />
            <p class="caption" style="color: var(--color-slate)">เฉลี่ย: {{ currentDoseAvg.toFixed(2) }} mg/วัน | รวม {{ currentDoseWeek.toFixed(1) }} mg/week</p>
            <p v-if="latestHosxpVisit?.parseNotes.length" class="caption dose-warning">{{ latestHosxpVisit.parseNotes.join(' | ') }}</p>
          </div>

          <div class="suggestion-row">
            <button class="btn btn-secondary" @click="fetchSuggestion" :disabled="loadingSuggestion || inrValue === null">
              {{ loadingSuggestion ? 'กำลังคำนวณ...' : 'คำนวณขนาดยา' }}
            </button>
            <span v-if="suggestion" class="body-sm">
              แนะนำ: <strong>{{ suggestion.suggestedDoseMgday.toFixed(1) }} mg/วัน</strong> &mdash; {{ suggestion.recommendation }}
            </span>
          </div>

          <div v-if="doseOptions.length > 0 || loadingDoseOptions || doseOptionsError" class="dose-options-section">
            <p class="caption label">ตัวเลือกตารางการกินยา</p>
            <DoseOptionsPanel
              :options="doseOptions"
              :selected-index="selectedDoseOptionIndex"
              :loading="loadingDoseOptions"
              @select="handleSelectDoseOption"
            />
            <p v-if="doseOptionsError" class="caption" style="color: var(--color-brand-coral)">{{ doseOptionsError }}</p>
            <p v-if="selectedDoseOptionIndex !== null" class="body-sm-medium" style="color: var(--color-success-accent)">
              ✓ เลือกตัวเลือก {{ selectedDoseOptionIndex + 1 }} แล้ว ตารางยาใหม่จะถูกอัพเดทด้านล่าง
            </p>
          </div>

          <div class="form-section">
            <p class="caption label">ตารางยาใหม่ (mg/วัน)</p>
            <DayDoseTable v-model="newDoseDetail" />
            <p class="caption" style="color: var(--color-slate)">เฉลี่ย: {{ newDoseAvg.toFixed(2) }} mg/วัน | รวม {{ newDoseWeek.toFixed(1) }} mg/week</p>
          </div>

          <div class="form-row">
            <label class="form-field">
              <span class="caption label">นัดครั้งต่อไป</span>
              <input class="input" type="date" v-model="nextAppointment" />
            </label>
            <label class="form-field">
              <span class="caption label">ตรวจ INR ครั้งต่อไป</span>
              <input class="input" type="date" v-model="nextInrDue" />
            </label>
            <label class="form-field">
              <span class="caption label">แพทย์</span>
              <input class="input" v-model="physician" />
            </label>
          </div>

          <div class="form-section">
            <p class="caption label">การรับประทานยา</p>
            <div class="radio-group">
              <label v-for="opt in [['good', 'ดี'], ['fair', 'พอใช้'], ['poor', 'ไม่ดี']]" :key="opt[0]" class="radio-label">
                <input type="radio" :value="opt[0]" v-model="adherence" />
                {{ opt[1] }}
              </label>
            </div>
          </div>

          <div class="form-section">
            <p class="caption label">อาการไม่พึงประสงค์</p>
            <div class="checkbox-grid">
              <label v-for="se in sideEffectOptions" :key="se.key" class="checkbox-label">
                <input type="checkbox" :value="se.key" v-model="selectedSideEffects" />
                {{ se.label }}
              </label>
            </div>
          </div>

          <label class="form-field">
            <span class="caption label">หมายเหตุ</span>
            <textarea class="input" rows="3" v-model="notes" />
          </label>
        </div>

        <div class="panel-footer">
          <button class="btn btn-ghost" @click="modelValue = false">ยกเลิก</button>
          <button class="btn btn-primary" @click="handleSubmit" :disabled="saving">
            {{ saving ? 'กำลังบันทึก...' : 'บันทึก' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.panel-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.35); z-index: 50;
  display: flex; align-items: stretch; justify-content: flex-end;
}
.visit-panel {
  width: min(760px, 100vw); height: 100vh; display: flex; flex-direction: column;
  border-radius: var(--rounded-xl) 0 0 var(--rounded-xl); overflow: hidden;
}
.panel-header {
  display: flex; justify-content: space-between; align-items: center;
  padding: var(--spacing-xl); border-bottom: 1px solid var(--color-hairline-soft);
}
.panel-body { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: var(--spacing-lg); padding: var(--spacing-xl); }
.panel-footer { display: flex; justify-content: flex-end; gap: var(--spacing-md); padding: var(--spacing-xl); border-top: 1px solid var(--color-hairline-soft); }
.form-row { display: grid; grid-template-columns: 180px 120px; gap: var(--spacing-md); }
.form-row .input { height: 44px; }
.form-field { display: flex; flex-direction: column; gap: var(--spacing-xs); }
.dose-source-card {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  border: 1px solid var(--color-hairline-soft);
  border-radius: var(--rounded-xl);
  background: var(--color-teal-light);
}
.dose-source-text { color: var(--color-slate); }
.dose-warning { color: var(--color-brand-coral); }
.btn-compact { padding-inline: var(--spacing-md); white-space: nowrap; }
.form-section { display: flex; flex-direction: column; gap: var(--spacing-sm); }
.label { color: var(--color-slate); }
.suggestion-row { display: flex; align-items: center; gap: var(--spacing-md); flex-wrap: wrap; }
.dose-options-section { display: flex; flex-direction: column; gap: var(--spacing-sm); margin-top: var(--spacing-sm); }
.radio-group { display: flex; gap: var(--spacing-lg); }
.radio-label { display: flex; align-items: center; gap: var(--spacing-xs); cursor: pointer; }
.checkbox-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--spacing-xs); }
.checkbox-label { display: flex; align-items: center; gap: var(--spacing-xs); cursor: pointer; }
</style>
