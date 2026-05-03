<script setup lang="ts">
const props = defineProps<{
  label: string
  color?: 'coral' | 'teal' | 'purple' | 'success' | 'danger' | 'muted' | 'yellow'
}>()
</script>

<template>
  <span :class="['badge', `pill-${props.color ?? 'muted'}`]">{{ props.label }}</span>
</template>

<style scoped>
.pill-coral { background: var(--color-coral-light); color: var(--color-coral-dark); }
.pill-teal { background: var(--color-teal-light); color: var(--color-moss-dark); }
.pill-purple { background: var(--color-surface-pricing-featured); color: var(--color-brand-blue); }
.pill-success { background: var(--color-success-accent); color: var(--color-on-primary); }
.pill-danger { background: var(--color-brand-red); color: var(--color-brand-red-dark); }
.pill-muted { background: var(--color-surface); color: var(--color-stone); }
.pill-yellow { background: var(--color-yellow-light); color: var(--color-yellow-dark); }
</style>
