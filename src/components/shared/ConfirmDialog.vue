<script setup lang="ts">
defineProps<{
  title: string
  message: string
  confirmLabel?: string
  cancelLabel?: string
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('cancel')">
    <div class="dialog-box card">
      <div class="dialog-body">
        <h3 class="h4">{{ title }}</h3>
        <p class="body-sm dialog-message">{{ message }}</p>
      </div>
      <div class="dialog-actions">
        <button type="button" class="btn btn-secondary" @click="emit('cancel')">{{ cancelLabel ?? 'ยกเลิก' }}</button>
        <button type="button" class="btn btn-primary" @click="emit('confirm')">{{ confirmLabel ?? 'ยืนยัน' }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-xl);
  background: color-mix(in srgb, var(--color-ink-deep) 24%, transparent);
  z-index: 40;
}
.dialog-box { width: min(100%, 32rem); box-shadow: var(--elevation-4); }
.dialog-body { display: flex; flex-direction: column; gap: var(--spacing-md); }
.dialog-message { color: var(--color-slate); }
.dialog-actions { display: flex; justify-content: flex-end; gap: var(--spacing-sm); margin-top: var(--spacing-xl); }
</style>
