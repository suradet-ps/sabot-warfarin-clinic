<script setup lang="ts">
import { computed } from 'vue'
import type { InrRecord } from '#/types/inr'
import { formatThaiDate } from '#/utils/clinic'

const props = defineProps<{
  inrRecords: InrRecord[]
  targetLow: number
  targetHigh: number
}>()

const chartWidth = 960
const chartHeight = 320
const padding = { top: 20, right: 56, bottom: 40, left: 20 }

type ChartPoint = InrRecord & { x: number; y: number }

const displayRecords = computed(() => {
  const ordered = [...props.inrRecords].sort((a, b) => a.date.localeCompare(b.date))
  const latest = ordered[ordered.length - 1]
  if (!latest) return []

  const latestDate = new Date(latest.date)
  if (Number.isNaN(latestDate.getTime())) return ordered

  const windowStart = new Date(latestDate)
  windowStart.setFullYear(windowStart.getFullYear() - 1)
  const startKey = windowStart.toISOString().slice(0, 10)

  return ordered.filter((record) => record.date >= startKey)
})

const valueRange = computed(() => {
  const values = displayRecords.value.map((record) => record.value)
  if (!values.length) return null

  const rawMin = Math.min(...values, props.targetLow, props.targetHigh)
  const rawMax = Math.max(...values, props.targetLow, props.targetHigh)
  const min = Math.max(0, Math.floor((rawMin - 0.4) * 2) / 2)
  const max = Math.ceil((rawMax + 0.4) * 2) / 2

  return { min, max: max === min ? max + 1 : max }
})

const plotWidth = computed(() => chartWidth - padding.left - padding.right)
const plotHeight = computed(() => chartHeight - padding.top - padding.bottom)

function xForDate(date: string): number {
  const first = displayRecords.value[0]
  const last = displayRecords.value[displayRecords.value.length - 1]
  if (!first || !last) return padding.left

  const firstTime = new Date(first.date).getTime()
  const lastTime = new Date(last.date).getTime()
  const currentTime = new Date(date).getTime()
  const span = Math.max(lastTime - firstTime, 1)
  const offset = currentTime - firstTime
  return padding.left + (offset / span) * plotWidth.value
}

function yForValue(value: number): number {
  const range = valueRange.value
  if (!range) return padding.top
  return padding.top + ((range.max - value) / (range.max - range.min)) * plotHeight.value
}

const points = computed<ChartPoint[]>(() =>
  displayRecords.value.map((record) => ({
    ...record,
    x: xForDate(record.date),
    y: yForValue(record.value),
  })),
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
  if (!valueRange.value) return null
  const yTop = yForValue(props.targetHigh)
  const yBottom = yForValue(props.targetLow)
  return {
    y: Math.min(yTop, yBottom),
    height: Math.abs(yBottom - yTop),
    top: yTop,
    bottom: yBottom,
  }
})

const yTicks = computed(() => {
  const range = valueRange.value
  if (!range) return []

  const step = Math.max(0.5, Math.ceil(((range.max - range.min) / 4) * 2) / 2)
  const ticks: number[] = []
  for (let value = range.min; value <= range.max + 0.001; value += step) {
    ticks.push(Number(value.toFixed(1)))
  }
  return ticks
})

const xTicks = computed(() => {
  const records = displayRecords.value
  if (!records.length) return []
  if (records.length <= 6) {
    return records.map((record) => ({
      label: new Date(record.date).toLocaleDateString('th-TH', { month: 'short', day: 'numeric' }),
      x: xForDate(record.date),
    }))
  }

  const lastIndex = records.length - 1
  return Array.from({ length: 6 }, (_, index) => {
    const pointIndex = Math.round((index / 5) * lastIndex)
    const record = records[pointIndex]
    return {
      label: new Date(record.date).toLocaleDateString('th-TH', { month: 'short' }),
      x: xForDate(record.date),
    }
  })
})
</script>

<template>
  <div class="trend-card card">
    <div class="trend-header">
      <div>
        <h3 class="h5">INR ย้อนหลัง</h3>
        <p class="body-sm trend-meta" v-if="displayRecords.length">
          ล่าสุด: {{ formatThaiDate(displayRecords[displayRecords.length - 1]?.date) }} · แสดงย้อนหลัง 1 ปี
        </p>
      </div>
      <span class="badge badge-success">เป้าหมาย {{ targetLow.toFixed(1) }}–{{ targetHigh.toFixed(1) }}</span>
    </div>

    <div v-if="!displayRecords.length" class="trend-empty card">
      <p class="body-sm" style="color: var(--color-stone)">ไม่มีข้อมูล INR</p>
    </div>

    <div v-else class="chart-shell">
      <svg class="chart-svg" :viewBox="`0 0 ${chartWidth} ${chartHeight}`" role="img" aria-label="กราฟแนวโน้ม INR">
        <rect
          v-if="targetBand"
          :x="padding.left"
          :y="targetBand.y"
          :width="plotWidth"
          :height="targetBand.height"
          class="target-band"
        />

        <g class="grid">
          <line
            v-for="tick in yTicks"
            :key="`y-${tick}`"
            :x1="padding.left"
            :y1="yForValue(tick)"
            :x2="chartWidth - padding.right"
            :y2="yForValue(tick)"
          />
        </g>

        <g class="targets">
          <line
            :x1="padding.left"
            :y1="yForValue(targetHigh)"
            :x2="chartWidth - padding.right"
            :y2="yForValue(targetHigh)"
            class="target-line"
          />
          <line
            :x1="padding.left"
            :y1="yForValue(targetLow)"
            :x2="chartWidth - padding.right"
            :y2="yForValue(targetLow)"
            class="target-line"
          />
        </g>

        <path :d="linePath" class="trend-line" />

        <g class="point-layer">
          <circle
            v-for="point in points"
            :key="`${point.date}-${point.value}`"
            :cx="point.x"
            :cy="point.y"
            r="4"
            class="trend-point"
          />
        </g>

        <g class="x-axis">
          <template v-for="tick in xTicks" :key="`x-${tick.x}-${tick.label}`">
            <line :x1="tick.x" :y1="padding.top" :x2="tick.x" :y2="chartHeight - padding.bottom" class="axis-grid" />
            <text :x="tick.x" :y="chartHeight - 12" text-anchor="middle">{{ tick.label }}</text>
          </template>
        </g>

        <g class="y-axis">
          <text
            v-for="tick in yTicks"
            :key="`label-${tick}`"
            :x="chartWidth - 8"
            :y="yForValue(tick) + 4"
            text-anchor="end"
          >
            {{ tick.toFixed(1) }}
          </text>
        </g>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.trend-card { display: flex; flex-direction: column; gap: var(--spacing-lg); }
.trend-header { display: flex; justify-content: space-between; align-items: flex-start; gap: var(--spacing-md); }
.trend-meta { color: var(--color-slate); }
.trend-empty { display: grid; place-items: center; min-height: 16rem; }
.chart-shell {
  border: 1px solid var(--color-hairline-soft);
  border-radius: var(--rounded-xl);
  overflow: hidden;
  background: linear-gradient(180deg, rgba(255,255,255,0.96) 0%, rgba(248,248,248,0.98) 100%);
}
.chart-svg {
  display: block;
  width: 100%;
  height: auto;
}
.grid line,
.axis-grid {
  stroke: var(--color-hairline-soft);
  stroke-width: 1;
}
.target-band {
  fill: color-mix(in srgb, var(--color-success-accent) 10%, transparent);
}
.target-line {
  stroke: var(--color-success-accent);
  stroke-width: 1.5;
  stroke-dasharray: 7 7;
}
.trend-line {
  fill: none;
  stroke: var(--color-primary);
  stroke-width: 3;
  stroke-linecap: round;
  stroke-linejoin: round;
}
.trend-point {
  fill: var(--color-primary);
  stroke: var(--color-canvas);
  stroke-width: 2;
}
.x-axis text,
.y-axis text {
  fill: var(--color-slate);
  font-size: 12px;
}
</style>
