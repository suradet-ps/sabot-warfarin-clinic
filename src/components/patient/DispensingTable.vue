<script setup lang="ts">
import { computed } from 'vue'
import type { DispensingRecord } from '#/types/dispensing'
import { formatThaiDate, sortDispensing } from '#/utils/clinic'

const props = defineProps<{ records: DispensingRecord[] }>()
const orderedRecords = computed(() => sortDispensing(props.records))
</script>

<template>
  <section class="card dispensing-section">
    <div><h3 class="h5">&#x0E1B;&#x0E23;&#x0E30;&#x0E27;&#x0E31;&#x0E15;&#x0E34;&#x0E01;&#x0E32;&#x0E23;&#x0E08;&#x0E48;&#x0E32;&#x0E22;&#x0E22;&#x0E32;</h3><p class="body-sm section-meta">&#x0E02;&#x0E49;&#x0E2D;&#x0E21;&#x0E39;&#x0E25;&#x0E08;&#x0E32;&#x0E01; HosXP</p></div>
    <table class="table">
      <thead><tr><th>&#x0E27;&#x0E31;&#x0E19;&#x0E17;&#x0E35;&#x0E48;</th><th>&#x0E0A;&#x0E37;&#x0E48;&#x0E2D;&#x0E22;&#x0E32;</th><th>&#x0E04;&#x0E27;&#x0E32;&#x0E21;&#x0E41;&#x0E23;&#x0E07;</th><th>&#x0E08;&#x0E33;&#x0E19;&#x0E27;&#x0E19;</th></tr></thead>
      <tbody>
        <tr v-if="orderedRecords.length === 0"><td colspan="4" class="empty-cell">&#x0E44;&#x0E21;&#x0E48;&#x0E21;&#x0E35;&#x0E1B;&#x0E23;&#x0E30;&#x0E27;&#x0E31;&#x0E15;&#x0E34;&#x0E01;&#x0E32;&#x0E23;&#x0E08;&#x0E48;&#x0E32;&#x0E22;&#x0E22;&#x0E32;</td></tr>
        <tr v-for="record in orderedRecords" :key="`${record.vstdate}-${record.icode}-${record.qty}`" class="comparison-row"><td>{{ formatThaiDate(record.vstdate) }}</td><td>{{ record.drugName }}</td><td>{{ record.strength }}</td><td>{{ record.qty.toFixed(0) }}</td></tr>
      </tbody>
    </table>
  </section>
</template>

<style scoped>
.dispensing-section { display: flex; flex-direction: column; gap: var(--spacing-lg); }
.section-meta { color: var(--color-slate); }
.empty-cell { text-align: center; color: var(--color-stone); }
.comparison-row td { border-bottom-color: var(--color-hairline-soft); }
</style>
