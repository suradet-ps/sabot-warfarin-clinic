<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useScreeningStore } from '#/stores/screening'
import PatientTable from '#/components/screening/PatientTable.vue'
import EnrollModal from '#/components/screening/EnrollModal.vue'
import { Calendar, Search } from 'lucide-vue-next'

const store = useScreeningStore()
const showEnrollModal = ref(false)
const selectedHn = ref<string | null>(null)
// Template refs for date inputs
const dateFromRef = ref<HTMLInputElement | null>(null)
const dateToRef = ref<HTMLInputElement | null>(null)
let autoSearchTimer: ReturnType<typeof window.setTimeout> | null = null

function runSearch(resetPage = true) {
  if (resetPage) store.resetPage()
  void store.search()
}

function scheduleAutoSearch() {
  if (autoSearchTimer !== null) window.clearTimeout(autoSearchTimer)
  autoSearchTimer = window.setTimeout(() => {
    runSearch()
  }, 250)
}

function handlePageChange(page: number) {
  store.setPage(page)
  void store.search()
}

function openDateFromPicker() {
  if (dateFromRef.value) (dateFromRef.value as any).showPicker?.()
}
function openDateToPicker() {
  if (dateToRef.value) (dateToRef.value as any).showPicker?.()
}

onMounted(() => {
  void store.search()
})

onBeforeUnmount(() => {
  if (autoSearchTimer !== null) window.clearTimeout(autoSearchTimer)
})

watch(
  [
    () => store.filters.keyword,
    () => store.filters.dateFrom,
    () => store.filters.dateTo,
  ],
  () => {
    scheduleAutoSearch()
  },
)
</script>

<template>
  <div class="screening-view">
    <div class="page-toolbar">
      <div class="toolbar-left">
        <div class="search-pill-wrap">
          <Search :size="16" style="color: var(--color-steel)" />
          <input
            v-model.trim="store.filters.keyword"
            class="search-pill-input"
            placeholder="ค้นหา HN, ชื่อ, นามสกุล, เบอร์โทร"
            @keydown.enter.prevent="runSearch()"
          />
        </div>
        <div class="date-field">
          <label class="date-label">จากวันที่</label>
          <div class="date-input-wrap">
            <input ref="dateFromRef" v-model="store.filters.dateFrom" class="input toolbar-date-input" type="date" />
            <Calendar :size="15" class="date-icon" @click="openDateFromPicker" />
          </div>
        </div>
        <div class="date-field">
          <label class="date-label">ถึงวันที่</label>
          <div class="date-input-wrap">
            <input ref="dateToRef" v-model="store.filters.dateTo" class="input toolbar-date-input" type="date" />
            <Calendar :size="15" class="date-icon" @click="openDateToPicker" />
          </div>
        </div>
        <!-- Enrollment status filter removed as requested -->
      </div>

      <button class="btn btn-primary toolbar-button" @click="runSearch()">
        <Search :size="16" />
        ค้นหา
      </button>
    </div>

    <p class="caption helper-text"></p>

    <div v-if="store.loading" class="loading-state">กำลังโหลด...</div>
    <div v-else-if="store.error" class="error-state card card-feature-coral">เกิดข้อผิดพลาด: {{ store.error }}</div>

    <PatientTable
      v-else
      :records="store.results"
      :total="store.total"
      :page="store.filters.page"
      :page-size="store.filters.pageSize"
      @enroll="(hn) => { selectedHn = hn; showEnrollModal = true }"
      @page-change="handlePageChange"
    />

    <EnrollModal v-if="showEnrollModal && selectedHn" :hn="selectedHn" @close="showEnrollModal = false" @enrolled="showEnrollModal = false; runSearch()" />
  </div>
</template>

<style scoped>
.screening-view { display: flex; flex-direction: column; gap: var(--spacing-xl); }
.page-toolbar { display: flex; align-items: flex-end; justify-content: space-between; gap: var(--spacing-md); flex-wrap: wrap; }
.toolbar-left { display: flex; align-items: flex-end; gap: var(--spacing-md); flex-wrap: wrap; flex: 1; }
.search-pill-wrap { display: flex; align-items: center; gap: var(--spacing-xs); background: var(--color-surface); border: 1px solid var(--color-hairline); border-radius: var(--rounded-md); padding: 0 var(--spacing-md); height: 44px; min-width: 260px; }
.search-pill-input { border: none; background: transparent; outline: none; font-family: var(--font-family-primary); font-size: var(--typography-body-sm-size); color: var(--color-ink); flex: 1; height: 100%; }
.date-field { display: flex; flex-direction: column; gap: 4px; }
.date-label { font-size: 12px; color: var(--color-slate); font-weight: 500; }
.date-input-wrap { position: relative; display: flex; align-items: center; }
.toolbar-date-input { width: 160px; padding-right: 32px; height: 44px; box-sizing: border-box; }
.date-icon { position: absolute; right: 10px; color: var(--color-slate); cursor: pointer; pointer-events: auto; }
.btn, .toolbar-button { height: 44px; display: inline-flex; align-items: center; padding: 0 14px; }
.helper-text { color: var(--color-slate); }
.loading-state, .error-state { padding: var(--spacing-xxl); text-align: center; color: var(--color-slate); }
</style>
