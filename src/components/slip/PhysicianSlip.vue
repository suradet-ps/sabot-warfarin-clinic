<script setup lang="ts">
import { computed, ref } from 'vue'
import { useSettingsStore } from '#/stores/settings'
import RegimenOptionCard from '#/components/visit/RegimenOptionCard.vue'
import type { PatientDetail } from '#/types/patient'
import type { WfVisit } from '#/types/visit'
import { calculateAge, doseDayKeys, doseDayLabels, formatThaiDate, normalizeDoseSchedule, scheduleWeeklyTotal, scheduleAverageDose } from '#/utils/clinic'
import { createRegimenOptionSnapshot } from '#/utils/regimen'

const props = defineProps<{ visit: WfVisit; patient: PatientDetail; ttr: number | null }>()

const settingsStore = useSettingsStore()
const hospitalName = ref('โรงพยาบาลสระโบสถ์')

settingsStore.loadSettings().then(() => {
  hospitalName.value = settingsStore.hospitalName || hospitalName.value
})

const CHART_WIDTH = 400
const CHART_HEIGHT = 160
const PADDING = { top: 12, right: 32, bottom: 24, left: 12 }

type ChartPoint = { date: string; value: number; x: number; y: number }

const inrRecords = computed(() => {
  return [...(props.patient.inrHistory ?? [])]
    .sort((a, b) => a.date.localeCompare(b.date))
    .slice(-12)
})

const valueRange = computed(() => {
  const values = inrRecords.value.map(r => r.value)
  if (!values.length) return null
  const targetLow = props.patient.patient.targetInrLow
  const targetHigh = props.patient.patient.targetInrHigh
  const rawMin = Math.min(...values, targetLow, targetHigh)
  const rawMax = Math.max(...values, targetLow, targetHigh)
  const min = Math.max(0, Math.floor((rawMin - 0.4) * 2) / 2)
  const max = Math.ceil((rawMax + 0.4) * 2) / 2
  return { min, max: max === min ? max + 1 : max }
})

const plotWidth = CHART_WIDTH - PADDING.left - PADDING.right
const plotHeight = CHART_HEIGHT - PADDING.top - PADDING.bottom

function xForDate(date: string): number {
  const first = inrRecords.value[0]
  const last = inrRecords.value[inrRecords.value.length - 1]
  if (!first || !last) return PADDING.left
  const firstTime = new Date(first.date).getTime()
  const lastTime = new Date(last.date).getTime()
  const currentTime = new Date(date).getTime()
  const span = Math.max(lastTime - firstTime, 1)
  return PADDING.left + ((currentTime - firstTime) / span) * plotWidth
}

function yForValue(value: number): number {
  const range = valueRange.value
  if (!range) return PADDING.top
  return PADDING.top + ((range.max - value) / (range.max - range.min)) * plotHeight
}

const points = computed<ChartPoint[]>(() =>
  inrRecords.value.map(record => ({
    ...record,
    x: xForDate(record.date),
    y: yForValue(record.value),
  }))
)

function buildSmoothPath(series: ChartPoint[]): string {
  if (series.length === 0) return ''
  if (series.length === 1) return `M ${series[0].x} ${series[0].y}`
  if (series.length === 2) return `M ${series[0].x} ${series[0].y} L ${series[1].x} ${series[1].y}`
  let path = `M ${series[0].x} ${series[0].y}`
  for (let index = 0; index < series.length - 1; index += 1) {
    const p0 = series[index - 1] ?? series[index]
    const p1 = series[index]
    const p2 = series[index + 1]
    const p3 = series[index + 2] ?? p2
    const cp1x = p1.x + (p2.x - p0.x) / 6
    const cp1y = p1.y + (p2.y - p0.y) / 6
    const cp2x = p2.x - (p3.x - p1.x) / 6
    const cp2y = p2.y - (p3.y - p1.y) / 6
    path += ` C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${p2.x} ${p2.y}`
  }
  return path
}

const linePath = computed(() => buildSmoothPath(points.value))

const targetBand = computed(() => {
  const range = valueRange.value
  if (!range) return null
  const yTop = yForValue(props.patient.patient.targetInrHigh)
  const yBottom = yForValue(props.patient.patient.targetInrLow)
  return {
    y: Math.min(yTop, yBottom),
    height: Math.abs(yBottom - yTop),
  }
})

const yTicks = computed(() => {
  const range = valueRange.value
  if (!range) return []
  const step = Math.max(0.5, Math.ceil(((range.max - range.min) / 3) * 2) / 2)
  const ticks: number[] = []
  for (let value = range.min; value <= range.max + 0.001; value += step) {
    ticks.push(Number(value.toFixed(1)))
  }
  return ticks
})

const xTicks = computed(() => {
  const records = inrRecords.value
  if (!records.length) return []
  return records.filter((_, i) => i === 0 || i === records.length - 1 || i === Math.floor(records.length / 2)).map(record => ({
    label: new Date(record.date).toLocaleDateString('th-TH', { month: 'short', day: 'numeric' }),
    x: xForDate(record.date),
  }))
})

const info = computed(() => props.patient.hosxpInfo)
const p = computed(() => props.patient.patient)
const age = computed(() => (info.value ? calculateAge(info.value.birthday) : null))
const currentDoseSchedule = computed(() => normalizeDoseSchedule(props.visit.doseDetail))
const newDoseSchedule = computed(() => normalizeDoseSchedule(props.visit.newDoseDetail))
const selectedDoseOption = computed(() => props.visit.selectedDoseOption ?? createRegimenOptionSnapshot({
  schedule: props.visit.newDoseDetail,
  visitDate: props.visit.visitDate,
  nextAppointment: props.visit.nextAppointment,
}))

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
  <div class="slip-sheet">
    <div class="slip-header">
      <div class="slip-title">
        <strong>ใบนำส่งแพทย์</strong>
        <span class="slip-subtitle">คลินิกวาร์ฟาริน {{ hospitalName }}</span>
      </div>
      <div class="slip-date">{{ formatThaiDate(visit.visitDate) }}</div>
    </div>

    <table class="patient-table">
      <tbody>
        <tr>
          <td><span class="label">ชื่อ-สกุล</span>{{ [info?.pname, info?.fname, info?.lname].filter(Boolean).join(' ') || '-' }}</td>
          <td><span class="label">HN</span>{{ patient.patient.hn }}</td>
          <td><span class="label">อายุ</span>{{ age ?? '-' }} ปี</td>
          <td><span class="label">เพศ</span>{{ info?.sex === 'M' ? 'ชาย' : info?.sex === 'F' ? 'หญิง' : '-' }}</td>
        </tr>
        <tr>
          <td><span class="label">ข้อบ่งชี้</span>{{ p.indication || '-' }}</td>
          <td><span class="label">เป้าหมาย INR</span>{{ p.targetInrLow.toFixed(1) }}–{{ p.targetInrHigh.toFixed(1) }}</td>
          <td><span class="label">TTR 6 เดือน</span><span :class="ttrClass(ttr)">{{ ttr != null ? ttr.toFixed(0) + '%' : '-' }}</span></td>
          <td><span class="label">การรับประทานยา</span>{{ adherenceLabel[visit.adherence ?? ''] ?? '-' }}</td>
        </tr>
      </tbody>
    </table>

    <div class="inr-section">
      <div class="inr-today">
        <span class="label">ค่า INR วันนี้</span>
        <span class="inr-value">{{ visit.inrValue?.toFixed(2) ?? '-' }}</span>
      </div>
      <div class="inr-chart">
        <svg v-if="inrRecords.length" :viewBox="`0 0 ${CHART_WIDTH} ${CHART_HEIGHT}`" preserveAspectRatio="xMidYMid meet">
          <rect
            v-if="targetBand"
            :x="PADDING.left"
            :y="targetBand.y"
            :width="CHART_WIDTH - PADDING.left - PADDING.right"
            :height="targetBand.height"
            class="target-band"
          />
          <g class="grid">
            <line
              v-for="tick in yTicks"
              :key="`y-${tick}`"
              :x1="PADDING.left"
              :y1="yForValue(tick)"
              :x2="CHART_WIDTH - PADDING.right"
              :y2="yForValue(tick)"
            />
          </g>
          <g class="targets">
            <line
              :x1="PADDING.left"
              :y1="yForValue(patient.patient.targetInrHigh)"
              :x2="CHART_WIDTH - PADDING.right"
              :y2="yForValue(patient.patient.targetInrHigh)"
              class="target-line"
            />
            <line
              :x1="PADDING.left"
              :y1="yForValue(patient.patient.targetInrLow)"
              :x2="CHART_WIDTH - PADDING.right"
              :y2="yForValue(patient.patient.targetInrLow)"
              class="target-line"
            />
          </g>
          <path :d="linePath" class="trend-line" />
          <g class="points">
            <circle
              v-for="point in points"
              :key="`${point.date}-${point.value}`"
              :cx="point.x"
              :cy="point.y"
              r="3"
            />
          </g>
          <g class="x-axis">
            <template v-for="tick in xTicks" :key="`x-${tick.x}`">
              <line :x1="tick.x" :y1="PADDING.top" :x2="tick.x" :y2="CHART_HEIGHT - PADDING.bottom" class="axis-grid" />
              <text :x="tick.x" :y="CHART_HEIGHT - 4" text-anchor="middle">{{ tick.label }}</text>
            </template>
          </g>
          <g class="y-axis">
            <text
              v-for="tick in yTicks"
              :key="`label-${tick}`"
              :x="CHART_WIDTH - 4"
              :y="yForValue(tick) + 3"
              text-anchor="end"
            >
              {{ tick.toFixed(1) }}
            </text>
          </g>
        </svg>
        <span v-else class="no-data">ไม่มีข้อมูล</span>
      </div>
    </div>

    <div class="dose-section">
      <table class="dose-table">
        <thead>
          <tr>
            <th></th>
            <th v-for="k in doseDayKeys" :key="k">{{ doseDayLabels[k] }}</th>
            <th>เฉลี่ย/วัน</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td class="dose-label">ยาเดิม</td>
            <td v-for="k in doseDayKeys" :key="k">{{ currentDoseSchedule[k] || '-' }}</td>
            <td>{{ scheduleAverageDose(currentDoseSchedule).toFixed(1) }} mg</td>
          </tr>
          <tr class="new-dose-row">
            <td class="dose-label">ยาใหม่</td>
            <td v-for="k in doseDayKeys" :key="k">{{ newDoseSchedule[k] || '-' }}</td>
            <td>{{ scheduleAverageDose(newDoseSchedule).toFixed(1) }} mg</td>
          </tr>
        </tbody>
      </table>

      <div class="dose-instructions">
        <span class="label">วิธีกินยาที่เลือกจากหน้าบันทึกการทำคลินิก</span>
        <RegimenOptionCard :option="selectedDoseOption" label="การ์ดวิธีกินยา" selected />
      </div>
    </div>

    <div class="footer-section">
      <div class="appointment-info">
        <span class="label">นัดครั้งต่อไป:</span>
        <strong>{{ formatThaiDate(visit.nextAppointment) }}</strong>
      </div>
      <div class="signature-area">
        <div class="signature-box">
          <span class="label">หมายเหตุ/คำแนะนำ</span>
          <div class="sig-line" />
          <div class="sig-line" />
        </div>
        <div class="signature-box">
          <span class="label">ลายมือชื่อแพทย์</span>
          <div class="sig-line" />
          <span class="doctor-name">{{ visit.physician || '____________________' }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.slip-sheet {
  width: 100%;
  max-width: 800px;
  margin: 0 auto;
  padding: var(--spacing-xl);
  background: var(--color-canvas);
  font-family: var(--font-family-primary);
  color: var(--color-ink);
  box-sizing: border-box;
}

.slip-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-lg);
  padding-bottom: var(--spacing-md);
  border-bottom: 2px solid var(--color-primary);
}

.slip-title {
  display: flex;
  flex-direction: column;
}

.slip-title strong {
  font-size: var(--typography-heading-3-size);
  color: var(--color-primary);
}

.slip-subtitle {
  font-size: var(--typography-body-sm-size);
  color: var(--color-slate);
}

.slip-date {
  font-size: var(--typography-body-md-size);
  color: var(--color-ink);
}

.patient-table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: var(--spacing-lg);
}

.patient-table td {
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-hairline);
}

.patient-table .label {
  display: block;
  font-size: var(--typography-caption-size);
  color: var(--color-slate);
  margin-bottom: 2px;
}

.inr-section {
  display: flex;
  gap: var(--spacing-lg);
  align-items: center;
  margin-bottom: var(--spacing-lg);
  padding: var(--spacing-md);
  background: var(--color-surface);
  border-radius: var(--rounded-lg);
}

.inr-today {
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 120px;
}

.inr-today .label {
  font-size: var(--typography-caption-size);
  color: var(--color-slate);
}

.inr-value {
  font-size: 3rem;
  font-weight: 700;
  color: var(--color-primary);
  line-height: 1;
}

.inr-chart {
  flex: 1;
  height: 180px;
}

.inr-chart svg {
  width: 100%;
  height: 100%;
}

.inr-chart .no-data {
  display: grid;
  place-items: center;
  height: 100%;
  color: var(--color-stone);
  font-size: var(--typography-body-sm-size);
}

.inr-chart .target-band {
  fill: color-mix(in srgb, var(--color-success-accent) 15%, transparent);
}

.inr-chart .grid line {
  stroke: var(--color-hairline-soft);
  stroke-width: 1;
}

.inr-chart .axis-grid {
  stroke: var(--color-hairline-soft);
  stroke-width: 1;
}

.inr-chart .target-line {
  stroke: var(--color-success-accent);
  stroke-width: 1;
  stroke-dasharray: 4 4;
}

.inr-chart .trend-line {
  fill: none;
  stroke: var(--color-primary);
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.inr-chart .points circle {
  fill: var(--color-primary);
  stroke: var(--color-canvas);
  stroke-width: 1.5;
}

.inr-chart .x-axis text,
.inr-chart .y-axis text {
  fill: var(--color-slate);
  font-size: 10px;
}

.dose-section {
  margin-bottom: var(--spacing-lg);
}

.dose-table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: var(--spacing-md);
}

.dose-table th,
.dose-table td {
  padding: var(--spacing-sm);
  text-align: center;
  border: 1px solid var(--color-hairline);
}

.dose-table th {
  background: var(--color-surface);
  font-weight: 600;
  font-size: var(--typography-body-sm-size);
}

.dose-label {
  text-align: left !important;
  font-weight: 500;
  background: var(--color-surface);
}

.new-dose-row td {
  background: var(--color-teal-light);
  font-weight: 600;
}

.dose-instructions {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.dose-instructions .label {
  display: block;
  font-size: var(--typography-body-sm-size);
  color: var(--color-slate);
}

.footer-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  padding-top: var(--spacing-md);
  border-top: 1px solid var(--color-hairline);
}

.appointment-info {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  background: var(--color-surface);
  border-radius: var(--rounded-md);
}

.appointment-info .label {
  font-size: var(--typography-body-sm-size);
  color: var(--color-slate);
}

.appointment-info strong {
  font-size: var(--typography-body-md-size);
}

.signature-area {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--spacing-xl);
}

.signature-box {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.signature-box .label {
  font-size: var(--typography-body-sm-size);
  color: var(--color-slate);
}

.sig-line {
  height: 1px;
  background: var(--color-hairline);
  margin: var(--spacing-md) 0 var(--spacing-xs);
}

.doctor-name {
  font-size: var(--typography-body-sm-size);
  color: var(--color-ink);
}

.ttr-good { color: var(--color-success-accent); }
.ttr-warn { color: var(--color-brand-coral); }
.ttr-bad { color: var(--color-brand-red-dark); }

@media print {
  .slip-sheet {
    width: 100%;
    max-width: none;
    padding: 10mm;
    background: white;
    box-shadow: none;
  }
  .inr-section {
    background: white;
    border: 1px solid #e5e5e6;
  }
  .new-dose-row td {
    background: #e6faf9 !important;
    -webkit-print-color-adjust: exact;
    print-color-adjust: exact;
  }
}
</style>
