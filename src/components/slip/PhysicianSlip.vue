<script setup lang="ts">
import { computed, onMounted, ref, watch, nextTick } from 'vue'
import { useSettingsStore } from '#/stores/settings'
import type { PatientDetail } from '#/types/patient'
import type { WfVisit } from '#/types/visit'
import { calculateAge, doseDayKeys, doseDayLabels, formatThaiDate, normalizeDoseSchedule, scheduleWeeklyTotal, scheduleAverageDose } from '#/utils/clinic'
import { createChart, LineSeries } from 'lightweight-charts'

const props = defineProps<{ visit: WfVisit; patient: PatientDetail; ttr: number | null }>()

const settingsStore = useSettingsStore()
const hospitalName = ref('โรงพยาบาลสระโบสถ์')
const chartContainer = ref<HTMLElement | null>(null)
let chart: ReturnType<typeof createChart> | null = null

onMounted(async () => {
  await settingsStore.loadSettings()
  hospitalName.value = settingsStore.hospitalName || hospitalName.value
  await nextTick()
  renderChart()
})

watch(() => props.patient.inrHistory, () => {
  renderChart()
}, { deep: true })

function renderChart() {
  if (!chartContainer.value) return
  if (chart) {
    chart.remove()
    chart = null
  }

  const inrData = [...(props.patient.inrHistory ?? [])]
    .sort((a, b) => a.date.localeCompare(b.date))
    .slice(-12)

  if (inrData.length === 0) return

  chart = createChart(chartContainer.value, {
    width: chartContainer.value.clientWidth,
    height: 180,
    layout: {
      background: { color: 'transparent' },
      textColor: '#5a5a72',
    },
    grid: {
      vertLines: { color: '#f0f0f1' },
      horzLines: { color: '#f0f0f1' },
    },
    timeScale: {
      timeVisible: true,
      borderColor: '#e5e5e6',
    },
    rightPriceScale: {
      borderColor: '#e5e5e6',
    },
    crosshair: {
      mode: 0,
    },
  })

  const lineSeries = chart.addSeries(LineSeries, {
    color: '#1a6fff',
    lineWidth: 2,
    crosshairMarkerVisible: true,
  })

  const data = inrData.map((r, i) => ({
    time: r.date as unknown as string,
    value: r.value,
  }))

  lineSeries.setData(data)

  const targetLow = props.patient.patient.targetInrLow
  const targetHigh = props.patient.patient.targetInrHigh

  if (targetLow > 0) {
    const lowLine = chart.addSeries(LineSeries, {
      color: '#00a878',
      lineWidth: 1,
      lineStyle: 2,
      priceLineVisible: false,
      crosshairMarkerVisible: false,
    })
    lowLine.setData(data.map(d => ({ time: d.time, value: targetLow })))
  }

  if (targetHigh > 0) {
    const highLine = chart.addSeries(LineSeries, {
      color: '#00a878',
      lineWidth: 1,
      lineStyle: 2,
      priceLineVisible: false,
      crosshairMarkerVisible: false,
    })
    highLine.setData(data.map(d => ({ time: d.time, value: targetHigh })))
  }

  chart.timeScale().fitContent()
}

const info = computed(() => props.patient.hosxpInfo)
const p = computed(() => props.patient.patient)
const age = computed(() => (info.value ? calculateAge(info.value.birthday) : null))
const currentDoseSchedule = computed(() => normalizeDoseSchedule(props.visit.doseDetail))
const newDoseSchedule = computed(() => normalizeDoseSchedule(props.visit.newDoseDetail))
const doseDescription = computed(() => props.visit.newDoseDescription || '')

const adherenceLabel: Record<string, string> = {
  good: 'ดี',
  fair: 'พอใช้',
  poor: 'ไม่ดี',
}

const doseInstructions = computed(() => {
  const schedule = newDoseSchedule.value

  const pillsByDay: { day: string; dose: number; pills: string }[] = []

  for (const dayKey of doseDayKeys) {
    const dose = schedule[dayKey] || 0
    const dayLabel = doseDayLabels[dayKey]

    let pillsText = '-'
    if (dose > 0) {
      const pillList: string[] = []
      if (dose >= 5) pillList.push(`${Math.floor(dose / 5)} เม็ด (5 mg)`)
      const rem5 = dose % 5
      if (rem5 >= 3) pillList.push(`${Math.floor(rem5 / 3)} เม็ด (3 mg)`)
      const rem3 = rem5 % 3
      if (rem3 >= 2) pillList.push('1 เม็ด (2 mg)')
      if (dose - Math.floor(dose) >= 0.4) pillList.push('ครึ่งเม็ด (1 mg)')
      pillsText = pillList.join(' + ')
    } else {
      pillsText = 'หยุดยา'
    }

    pillsByDay.push({ day: dayLabel, dose, pills: pillsText })
  }

  const totalMg = scheduleWeeklyTotal(schedule)

  const visitDate = props.visit.visitDate
  const nextAppt = props.visit.nextAppointment
  const dayCount = nextAppt && visitDate 
    ? Math.ceil((new Date(nextAppt).getTime() - new Date(visitDate).getTime()) / (1000 * 60 * 60 * 24)) + 1
    : 7

  return {
    pillsByDay,
    totalDays: dayCount,
    totalMg,
    hasDateRange: !!(visitDate && nextAppt)
  }
})

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
      <div ref="chartContainer" class="inr-chart" />
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

<div v-if="doseInstructions" class="dose-instructions">
        <span class="label">วิธีกินยา{{ doseInstructions.hasDateRange ? ` (ตั้งแต่วันที่ ${formatThaiDate(visit.visitDate)} ถึง ${formatThaiDate(visit.nextAppointment)})` : '' }}</span>
        
        <div v-if="doseDescription" class="dose-pattern">
          <strong>{{ doseDescription }}</strong>
        </div>

        <table class="instruction-table">
          <thead>
            <tr>
              <th v-for="item in doseInstructions.pillsByDay" :key="item.day">{{ item.day }}</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td v-for="item in doseInstructions.pillsByDay" :key="item.day">{{ item.pills }}</td>
            </tr>
          </tbody>
        </table>
        
        <div v-if="doseInstructions.hasDateRange && visit.nextAppointment" class="pill-summary">
          <div class="summary-title">{{ visit.totalPillsSummary?.header || 'รวมยาถึงวันนัด:' }}</div>
          <div v-if="visit.totalPillsSummary?.pillLines?.length" class="pill-lines">
            <div v-for="line in visit.totalPillsSummary.pillLines" :key="line.mg" class="pill-line">
              {{ line.mg }}mg: {{ line.dispensedCount }} เม็ด{{ line.usageNote }}
            </div>
          </div>
          <div v-else class="pill-line">ไม่ต้องจ่ายยา</div>
        </div>
        
        <div class="instruction-summary">รวม {{ doseInstructions.totalDays }} วัน ขนาดยารวม {{ doseInstructions.totalMg.toFixed(1) }} mg</div>
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
  padding: var(--spacing-md);
  background: var(--color-yellow-light);
  border-radius: var(--rounded-md);
  border: 1px solid var(--color-hairline);
}

.dose-instructions .label {
  display: block;
  font-size: var(--typography-body-sm-size);
  color: var(--color-slate);
  margin-bottom: var(--spacing-xs);
}

.instruction-detail {
  font-size: var(--typography-body-sm-size);
  color: var(--color-ink);
  margin-bottom: var(--spacing-sm);
}

.instruction-summary {
  font-size: var(--typography-body-sm-size);
  font-weight: 500;
  color: var(--color-charcoal);
}

.dose-pattern {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-teal-light);
  border-radius: var(--rounded-md);
  margin-bottom: var(--spacing-sm);
  font-size: var(--typography-body-sm-size);
  color: var(--color-moss-dark);
}

.pill-summary {
  margin-top: var(--spacing-md);
  padding: var(--spacing-md);
  background: var(--color-surface);
  border-radius: var(--rounded-md);
  border: 1px solid var(--color-hairline);
}

.summary-title {
  font-weight: 600;
  font-size: var(--typography-body-sm-size);
  color: var(--color-ink);
  margin-bottom: var(--spacing-xs);
}

.pill-lines {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xxs);
}

.pill-line {
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