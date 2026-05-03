<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Download } from 'lucide-vue-next'

interface CensusReport { active: number; inactive: number; transferred: number; discharged: number; deceased: number; total: number }
interface TtrReport { meanTtr: number }
interface AdverseReport { totalEvents: number }

const census = ref<CensusReport | null>(null)
const ttr = ref<TtrReport | null>(null)
const adverse = ref<AdverseReport | null>(null)
const loading = ref(false)

const reportCards = computed(() => [
  {
    key: 'census',
    title: 'สถิติผู้ป่วย',
    value: census.value ? `${census.value.total}` : '-',
    description: census.value ? `Active ${census.value.active} · Inactive ${census.value.inactive}` : 'กำลังโหลดข้อมูล',
    tone: 'card-feature-yellow',
    rows: census.value ? [['กำลังติดตาม', `${census.value.active}`], ['หยุดติดตาม', `${census.value.inactive}`], ['ส่งต่อ', `${census.value.transferred}`], ['จำหน่าย', `${census.value.discharged}`], ['เสียชีวิต', `${census.value.deceased}`]] : [],
  },
  {
    key: 'ttr',
    title: 'TTR เฉลี่ย',
    value: ttr.value ? `${ttr.value.meanTtr.toFixed(0)}%` : '-',
    description: 'Rosendaal method · 6 เดือนล่าสุด',
    tone: 'card-feature-teal',
    rows: ttr.value ? [['ค่าเฉลี่ยทั้งคลินิก', `${ttr.value.meanTtr.toFixed(2)}%`]] : [],
  },
  {
    key: 'adverse',
    title: 'เหตุการณ์ไม่พึงประสงค์',
    value: adverse.value ? `${adverse.value.totalEvents}` : '-',
    description: 'จำนวนเหตุการณ์ที่บันทึกทั้งหมด',
    tone: 'card-feature-coral',
    rows: adverse.value ? [['รวมเหตุการณ์', `${adverse.value.totalEvents}`]] : [],
  },
])

async function loadReports() {
  loading.value = true
  try {
    const [censusData, ttrData, adverseData] = await Promise.all([
      invoke<CensusReport>('get_report_data', { reportType: 'census' }),
      invoke<TtrReport>('get_report_data', { reportType: 'ttr' }),
      invoke<AdverseReport>('get_report_data', { reportType: 'adverse' }),
    ])
    census.value = censusData
    ttr.value = ttrData
    adverse.value = adverseData
  } finally {
    loading.value = false
  }
}

function exportCsv(title: string, rows: string[][]) {
  const csv = [['หัวข้อ', 'ค่า'], ...rows].map((row) => row.map((cell) => `"${cell.replaceAll('"', '""')}"`).join(',')).join('\n')
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `${title}.csv`
  link.click()
  URL.revokeObjectURL(url)
}

onMounted(() => {
  void loadReports()
})
</script>

<template>
  <div class="reports-view">
    <div v-if="loading" class="card">กำลังโหลดรายงาน...</div>
    <div class="reports-grid">
      <section v-for="report in reportCards" :key="report.key" :class="['report-card', 'card', report.tone]">
        <div class="card-head">
          <div>
            <h3 class="h5">{{ report.title }}</h3>
            <p class="body-sm report-description">{{ report.description }}</p>
          </div>
          <button type="button" class="btn btn-secondary" @click="exportCsv(report.key, report.rows)"><Download :size="16" />CSV</button>
        </div>
        <p class="report-value">{{ report.value }}</p>
        <div class="report-table">
          <div v-for="row in report.rows" :key="row[0]" class="report-row">
            <span class="body-sm">{{ row[0] }}</span>
            <strong class="body-sm-medium">{{ row[1] }}</strong>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.reports-view { display: flex; flex-direction: column; gap: var(--spacing-xl); }
.reports-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr)); gap: var(--spacing-xl); }
.report-card { display: flex; flex-direction: column; gap: var(--spacing-lg); }
.card-head,.report-row { display: flex; justify-content: space-between; gap: var(--spacing-md); }
.report-description { color: var(--color-slate); }
.report-value { font-size: var(--typography-stat-display-size); font-weight: var(--typography-stat-display-weight); line-height: var(--typography-stat-display-line-height); letter-spacing: var(--typography-stat-display-letter-spacing); }
.report-table { display: flex; flex-direction: column; gap: var(--spacing-sm); }
.report-row { padding-top: var(--spacing-sm); border-top: 1px solid var(--color-hairline-soft); }
</style>
