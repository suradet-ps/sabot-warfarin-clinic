<script setup lang="ts">
import { computed } from 'vue'
import type { DispensingRecord } from '#/types/dispensing'
import { doseDayLabels, formatThaiDate, normalizeDoseSchedule, sortDispensing } from '#/utils/clinic'

const props = defineProps<{ records: DispensingRecord[] }>()
const orderedRecords = computed(() => sortDispensing(props.records))

function activeDays(record: DispensingRecord): string {
  if (!record.parsedDose) return '-'
  const schedule = normalizeDoseSchedule(record.parsedDose.schedule)
  const labels = Object.entries(schedule)
    .filter(([, value]) => Number(value) > 0)
    .map(([day]) => doseDayLabels[day as keyof typeof doseDayLabels])
  return labels.join(' ') || '-'
}
</script>

<template>
  <section class="card dispensing-section">
    <div>
      <h3 class="h5">&#x0E1B;&#x0E23;&#x0E30;&#x0E27;&#x0E31;&#x0E15;&#x0E34;&#x0E01;&#x0E32;&#x0E23;&#x0E08;&#x0E48;&#x0E32;&#x0E22;&#x0E22;&#x0E32;</h3>
      <p class="body-sm section-meta">&#x0E02;&#x0E49;&#x0E2D;&#x0E21;&#x0E39;&#x0E25;&#x0E08;&#x0E32;&#x0E01; HosXP &#x0E1E;&#x0E23;&#x0E49;&#x0E2D;&#x0E21;&#x0E01;&#x0E32;&#x0E23;&#x0E15;&#x0E35;&#x0E04;&#x0E27;&#x0E32;&#x0E21;&#x0E27;&#x0E34;&#x0E18;&#x0E35;&#x0E43;&#x0E0A;&#x0E49;&#x0E22;&#x0E32;&#x0E40;&#x0E1B;&#x0E47;&#x0E19; mg/week</p>
    </div>
    <table class="table">
      <thead>
        <tr>
          <th>&#x0E27;&#x0E31;&#x0E19;&#x0E17;&#x0E35;&#x0E48;</th>
          <th>VN</th>
          <th>&#x0E22;&#x0E32;</th>
          <th>&#x0E27;&#x0E34;&#x0E18;&#x0E35;&#x0E43;&#x0E0A;&#x0E49;&#x0E22;&#x0E32;</th>
          <th>&#x0E04;&#x0E33;&#x0E19;&#x0E27;&#x0E13;&#x0E44;&#x0E14;&#x0E49;</th>
          <th>&#x0E08;&#x0E33;&#x0E19;&#x0E27;&#x0E19;</th>
        </tr>
      </thead>
      <tbody>
        <tr v-if="orderedRecords.length === 0">
          <td colspan="6" class="empty-cell">&#x0E44;&#x0E21;&#x0E48;&#x0E21;&#x0E35;&#x0E1B;&#x0E23;&#x0E30;&#x0E27;&#x0E31;&#x0E15;&#x0E34;&#x0E01;&#x0E32;&#x0E23;&#x0E08;&#x0E48;&#x0E32;&#x0E22;&#x0E22;&#x0E32;</td>
        </tr>
        <tr v-for="record in orderedRecords" :key="`${record.vstdate}-${record.vn ?? 'na'}-${record.icode}-${record.qty}`" class="comparison-row">
          <td>{{ formatThaiDate(record.vstdate) }}</td>
          <td>{{ record.vn || '-' }}</td>
          <td>
            <div class="drug-stack">
              <span class="body-sm-medium">{{ record.drugName }}</span>
              <span class="caption section-meta">{{ record.strength }}</span>
            </div>
          </td>
          <td>
            <div class="usage-stack">
              <span class="body-sm">{{ record.usageText || '-' }}</span>
              <span v-if="record.drugusageCode" class="caption section-meta">code: {{ record.drugusageCode }}</span>
            </div>
          </td>
          <td>
            <div v-if="record.parsedDose" class="calc-stack">
              <span class="body-sm-medium">{{ record.parsedDose.mgPerWeek.toFixed(1) }} mg/week</span>
              <span class="caption section-meta">{{ record.parsedDose.mgPerDayAverage.toFixed(2) }} mg/day เฉลี่ย</span>
              <span class="caption section-meta">{{ activeDays(record) }}</span>
              <span v-if="record.usageParseNote" class="caption calc-note">{{ record.usageParseNote }}</span>
            </div>
            <div v-else class="usage-stack">
              <span class="body-sm calc-missing">คำนวณอัตโนมัติไม่ได้</span>
              <span v-if="record.usageParseNote" class="caption calc-note">{{ record.usageParseNote }}</span>
            </div>
          </td>
          <td>{{ record.qty.toFixed(0) }}</td>
        </tr>
      </tbody>
    </table>
  </section>
</template>

<style scoped>
.dispensing-section { display: flex; flex-direction: column; gap: var(--spacing-lg); }
.section-meta { color: var(--color-slate); }
.empty-cell { text-align: center; color: var(--color-stone); }
.comparison-row td { border-bottom-color: var(--color-hairline-soft); }
.drug-stack, .usage-stack, .calc-stack { display: flex; flex-direction: column; gap: 2px; }
.calc-note { color: var(--color-brand-coral); }
.calc-missing { color: var(--color-brand-red-dark); }
</style>
