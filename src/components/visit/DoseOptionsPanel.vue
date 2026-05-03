<script setup lang="ts">
import { computed } from 'vue';
import { Check } from 'lucide-vue-next';
import PillVisual from '@/components/shared/PillVisual.vue';
import type { RegimenOption } from '@/types/dose';

const props = defineProps<{
  options: RegimenOption[];
  selectedIndex: number | null;
  loading?: boolean;
}>();

const emit = defineEmits<{
  (e: 'select', index: number): void;
}>();

const DAY_NAMES = ['จ.', 'อ.', 'พ.', 'พฤ.', 'ศ.', 'ส.', 'อา.'];

const DAY_HEADER_COLORS = [
  'bg-yellow-light',
  'bg-rose-light',
  'bg-teal-light',
  'bg-orange-light',
  'bg-blue-light',
  'bg-purple-light',
  'bg-coral-light',
];

function getDayName(dayIndex: number): string {
  return DAY_NAMES[dayIndex] ?? '';
}

function getDayHeaderColor(dayIndex: number): string {
  return DAY_HEADER_COLORS[dayIndex] ?? '';
}

function expandPills(pills: { mg: number; count: number; is_half: boolean }[]) {
  const expanded: { mg: number; isHalf: boolean; key: string }[] = [];
  for (const pill of pills) {
    for (let i = 0; i < pill.count; i++) {
      expanded.push({
        mg: pill.mg,
        isHalf: pill.is_half,
        key: `${pill.mg}-${pill.is_half}-${i}-${expanded.length}`,
      });
    }
  }
  return expanded;
}

function getPillLabel(pill: { mg: number; count: number; is_half: boolean }): string {
  const countText = pill.is_half ? `x${pill.count}(ครึ่ง)` : `x${pill.count}`;
  return `${pill.mg} mg ${countText}`;
}
</script>

<template>
  <div class="dose-options-panel">
    <div v-if="loading" class="loading-state">
      <span>กำลังคำนวณตัวเลือก...</span>
    </div>

    <div v-else-if="options.length === 0" class="empty-state">
      <span class="body-sm">ไม่พบตัวเลือกที่เหมาะสม</span>
    </div>

    <div v-else class="options-list">
      <div
        v-for="(option, index) in options" :key="index"
        class="option-card" :class="{ 'option-selected': selectedIndex === index }"
        @click="emit('select', index)"
      >
        <div class="option-header">
          <div class="option-info">
            <span v-if="selectedIndex === index" class="check-icon">
              <Check :size="16" />
            </span>
            <span class="option-label">ตัวเลือก {{ index + 1 }}</span>
          </div>
          <div class="option-description">{{ option.description }}</div>
          <div class="option-weekly-dose">
            {{ option.weekly_dose_actual.toFixed(1) }} <span class="text-muted">mg/สัปดาห์</span>
          </div>
        </div>

        <div class="schedule-grid">
          <div
            v-for="day in option.weekly_schedule" :key="day.day_index"
            class="schedule-day" :class="{
              'day-stop': day.is_stop_day,
              'day-special': day.is_special_day,
            }"
          >
            <div class="day-header" :class="getDayHeaderColor(day.day_index)">
              {{ getDayName(day.day_index) }}
            </div>
            <div class="day-content">
              <template v-if="day.is_stop_day">
                <span class="stop-text">หยุดยา</span>
              </template>
              <template v-else>
                <div class="pills-row">
                  <PillVisual
                    v-for="pill in expandPills(day.pills)" :key="pill.key"
                    :mg="pill.mg" :is-half="pill.isHalf"
                  />
                </div>
                <div class="dose-text">({{ day.total_dose.toFixed(1) }} mg)</div>
              </template>
            </div>
          </div>
        </div>

        <div class="pills-summary">
          <span class="summary-header">{{ option.total_pills_summary.header }}</span>
          <div v-if="option.total_pills_summary.pill_lines.length > 0" class="summary-lines">
            <div v-for="line in option.total_pills_summary.pill_lines" :key="line.mg" class="summary-line">
              <PillVisual :mg="line.mg" />
              <span>{{ line.mg }}mg: {{ line.dispensed_count }} เม็ด</span>
              <span v-if="line.usage_note" class="usage-note">{{ line.usage_note }}</span>
            </div>
          </div>
          <span v-else class="no-pills">ไม่ต้องจ่ายยา</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dose-options-panel {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.loading-state,
.empty-state {
  padding: var(--spacing-xl);
  text-align: center;
  color: var(--color-stone);
}

.options-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.option-card {
  border: 1px solid var(--color-hairline-soft);
  border-radius: var(--rounded-xl);
  padding: var(--spacing-lg);
  cursor: pointer;
  transition: all 0.2s ease;
  background: var(--color-canvas);
}

.option-card:hover {
  border-color: var(--color-hairline);
  box-shadow: var(--elevation-2);
}

.option-selected {
  border-color: var(--color-success-accent);
  background: var(--color-success-accent);
  background: linear-gradient(to bottom, var(--color-success-accent), color-mix(in srgb, var(--color-success-accent) 90%, white));
}

.option-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-md);
  flex-wrap: wrap;
}

.option-info {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.check-icon {
  color: var(--color-success-accent);
}

.option-label {
  font-weight: 600;
  font-size: var(--typography-caption-bold-size);
}

.option-description {
  flex: 1;
  font-size: var(--typography-body-sm-size);
  color: var(--color-slate);
}

.option-weekly-dose {
  font-weight: 600;
  font-size: var(--typography-body-md-size);
  color: var(--color-ink);
}

.schedule-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-md);
}

.schedule-day {
  border: 1px solid var(--color-hairline-soft);
  border-radius: var(--rounded-md);
  overflow: hidden;
}

.day-stop {
  background: var(--color-surface-soft);
}

.day-special {
  border-color: var(--color-brand-coral);
  border-width: 2px;
}

.day-header {
  padding: var(--spacing-xs);
  text-align: center;
  font-weight: 600;
  font-size: var(--typography-caption-size);
  color: var(--color-slate);
}

.day-content {
  padding: var(--spacing-sm);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
  min-height: 60px;
}

.stop-text {
  font-size: var(--typography-micro-size);
  color: var(--color-stone);
}

.pills-row {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 2px;
}

.dose-text {
  font-size: var(--typography-micro-size);
  color: var(--color-slate);
}

.pills-summary {
  padding: var(--spacing-md);
  background: var(--color-surface-soft);
  border-radius: var(--rounded-md);
  font-size: var(--typography-body-sm-size);
}

.summary-header {
  font-weight: 600;
  color: var(--color-slate);
  display: block;
  margin-bottom: var(--spacing-xs);
}

.summary-lines {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.summary-line {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  color: var(--color-slate);
}

.usage-note {
  color: var(--color-stone);
  font-size: var(--typography-micro-size);
}

.no-pills {
  color: var(--color-stone);
}

.text-muted {
  color: var(--color-stone);
  font-weight: 400;
}
</style>