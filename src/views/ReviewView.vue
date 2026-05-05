<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ClipboardCheck, Pencil, Search } from 'lucide-vue-next'
import VisitFormPanel from '#/components/visit/VisitFormPanel.vue'
import { formatThaiDate } from '#/utils/clinic'
import type { WfVisit } from '#/types/visit'

const visits = ref<WfVisit[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const searchQuery = ref('')
const visitPanelOpen = ref(false)
const selectedHn = ref('')
const editingVisit = ref<WfVisit | null>(null)
const approving = ref<Set<number>>(new Set())

async function loadVisits() {
  loading.value = true
  error.value = null
  try {
    visits.value = await invoke<WfVisit[]>('get_pending_review_visits')
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function approveVisit(visitId: number) {
  approving.value.add(visitId)
  try {
    await invoke('approve_visit', {
      visitId,
      reviewer: 'เภสัชกร',
    })
    visits.value = visits.value.filter((v) => v.id !== visitId)
    await refreshPendingCount()
  } catch (e) {
    console.error('failed to approve visit', e)
  } finally {
    approving.value.delete(visitId)
  }
}

async function refreshPendingCount() {
  try {
    await invoke('get_pending_review_count')
  } catch (e) {
    console.error('failed to refresh count', e)
  }
}

function handleEdit(visit: WfVisit) {
  selectedHn.value = visit.hn
  editingVisit.value = visit
  visitPanelOpen.value = true
}

async function onVisitSaved(visitId: number) {
  visitPanelOpen.value = false
  editingVisit.value = null
  await loadVisits()
}

async function handleVisitUpdated() {
  visitPanelOpen.value = false
  editingVisit.value = null
  await loadVisits()
}

const filteredVisits = () => {
  if (!searchQuery.value.trim()) return visits.value
  const query = searchQuery.value.toLowerCase()
  return visits.value.filter((v) => v.hn.toLowerCase().includes(query))
}

onMounted(() => {
  void loadVisits()
})
</script>

<template>
  <div class="review-view">
    <div class="page-toolbar">
      <div class="stat-row">
        <div class="stat-chip card">
          <ClipboardCheck :size="16" class="stat-icon" />
          <span class="body-sm">รอตรวจสอบ <strong>{{ visits.length }}</strong> รายการ</span>
        </div>
      </div>
      <div class="search-box">
        <Search :size="16" class="search-icon" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="ค้นหา HN"
          class="search-input"
        />
      </div>
    </div>

    <div v-if="loading" class="card loading-state body-sm">กำลังโหลด...</div>
    <div v-else-if="error" class="card card-feature-coral">{{ error }}</div>
    <div v-else-if="visits.length === 0" class="card empty-state">
      <ClipboardCheck :size="32" class="empty-icon" />
      <p>ไม่มีรายการที่รอตรวจสอบ</p>
    </div>
    <div v-else class="table-wrap card">
      <table class="table">
        <thead>
          <tr>
            <th>HN</th>
            <th>วันที่</th>
            <th>INR</th>
            <th>ขนาดยา (mg/วัน)</th>
            <th>นัดครั้งต่อไป</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="filteredVisits().length === 0">
            <td colspan="6" class="empty-cell">ไม่พบรายการ</td>
          </tr>
          <tr v-for="visit in filteredVisits()" :key="visit.id">
            <td>
              <span class="body-sm-medium">{{ visit.hn }}</span>
            </td>
            <td>{{ formatThaiDate(visit.visitDate) }}</td>
            <td>
              <span v-if="visit.inrValue" class="badge badge-tag-coral">INR {{ visit.inrValue.toFixed(1) }}</span>
              <span v-else>-</span>
            </td>
            <td>{{ visit.newDoseMgday?.toFixed(1) ?? '-' }}</td>
            <td>{{ visit.nextAppointment ? formatThaiDate(visit.nextAppointment) : '-' }}</td>
            <td>
              <div class="action-buttons">
                <button class="btn-icon" title="แก้ไข" @click="handleEdit(visit)">
                  <Pencil :size="14" />
                </button>
                <button
                  class="btn-approve"
                  :disabled="approving.has(visit.id)"
                  @click="approveVisit(visit.id)"
                >
                  ✓
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <VisitFormPanel
      v-model="visitPanelOpen"
      :hn="selectedHn"
      :edit-visit="editingVisit"
      @saved="onVisitSaved"
      @updated="handleVisitUpdated"
    />
  </div>
</template>

<style scoped>
.review-view {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xl);
  padding: var(--spacing-xl);
}

.page-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--spacing-md);
}

.stat-row {
  display: flex;
  gap: var(--spacing-sm);
}

.stat-chip {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-md);
}

.stat-icon {
  color: var(--color-brand-teal);
}

.search-box {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-md);
  background: var(--color-canvas);
  border: 1px solid var(--color-hairline);
  border-radius: var(--rounded-full);
  width: 280px;
}

.search-icon {
  color: var(--color-slate);
  flex-shrink: 0;
}

.search-input {
  border: none;
  background: transparent;
  outline: none;
  width: 100%;
  font-size: var(--typography-body-sm-size);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xxl);
  color: var(--color-slate);
}

.empty-icon {
  color: var(--color-success-accent);
}

.table-wrap {
  overflow-x: auto;
}

.table {
  width: 100%;
  border-collapse: collapse;
}

.table th,
.table td {
  padding: var(--spacing-sm) var(--spacing-md);
  text-align: left;
  border-bottom: 1px solid var(--color-hairline-soft);
}

.table th {
  font-weight: 600;
  color: var(--color-slate);
  font-size: var(--typography-caption-size);
  text-transform: uppercase;
}

.table td {
  font-size: var(--typography-body-sm-size);
}

.empty-cell {
  text-align: center;
  color: var(--color-slate);
  padding: var(--spacing-xl);
}

.action-buttons {
  display: flex;
  gap: var(--spacing-xs);
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: 1px solid var(--color-hairline);
  border-radius: var(--rounded-full);
  background: var(--color-canvas);
  color: var(--color-slate);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-icon:hover {
  background: var(--color-surface);
  color: var(--color-ink);
}

.btn-approve {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: var(--rounded-full);
  background: var(--color-brand-teal);
  color: var(--color-on-dark);
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-approve:hover:not(:disabled) {
  background: var(--color-brand-teal-dark);
}

.btn-approve:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>