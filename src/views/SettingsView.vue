<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '#/stores/settings'

const store = useSettingsStore()
const testResult = ref<boolean | null>(null)
const testing = ref(false)

onMounted(() => {
  void store.loadSettings()
})

async function handleTestConnection() {
  testing.value = true
  testResult.value = await store.testConnection()
  testing.value = false
}
</script>

<template>
  <div class="settings-view">
    <div class="settings-section card">
      <h3 class="h4" style="margin-bottom: var(--spacing-xl)">การเชื่อมต่อ HosXP MySQL</h3>
      <div class="form-grid">
        <label class="form-field"><span class="caption" style="color:var(--color-slate)">Host</span><input class="input" v-model="store.mysqlConfig.host" placeholder="localhost" /></label>
        <label class="form-field"><span class="caption" style="color:var(--color-slate)">Port</span><input class="input" type="number" v-model.number="store.mysqlConfig.port" /></label>
        <label class="form-field"><span class="caption" style="color:var(--color-slate)">Database</span><input class="input" v-model="store.mysqlConfig.database" /></label>
        <label class="form-field"><span class="caption" style="color:var(--color-slate)">Username</span><input class="input" v-model="store.mysqlConfig.username" /></label>
        <label class="form-field" style="grid-column: 1 / -1"><span class="caption" style="color:var(--color-slate)">Password</span><input class="input" type="password" v-model="store.mysqlConfig.password" /></label>
      </div>
      <div class="settings-actions">
        <button class="btn btn-secondary" @click="handleTestConnection" :disabled="testing">{{ testing ? 'กำลังทดสอบ...' : 'ทดสอบการเชื่อมต่อ' }}</button>
        <span v-if="testResult === true" class="badge badge-success">✓ เชื่อมต่อสำเร็จ</span>
        <span v-else-if="testResult === false" class="badge badge-danger">✗ เชื่อมต่อไม่ได้</span>
      </div>
    </div>

    <div class="settings-section card">
      <h3 class="h4" style="margin-bottom: var(--spacing-xl)">ข้อมูลโรงพยาบาล</h3>
      <label class="form-field"><span class="caption" style="color:var(--color-slate)">ชื่อโรงพยาบาล</span><input class="input" v-model="store.hospitalName" /></label>
    </div>
  </div>
</template>

<style scoped>
.settings-view { display: flex; flex-direction: column; gap: var(--spacing-xl); max-width: 720px; }
.form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--spacing-md); margin-bottom: var(--spacing-xl); }
.form-field { display: flex; flex-direction: column; gap: var(--spacing-xs); }
.settings-actions { display: flex; align-items: center; gap: var(--spacing-md); }
</style>
