import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useSettingsStore = defineStore('settings', () => {
  const mysqlConfig = ref({
    host: 'localhost',
    port: 3306,
    database: 'hosxp',
    username: '',
    password: '',
  })
  const hospitalName = ref('โรงพยาบาลสระโบสถ์')
  const staffList = ref<string[]>([])
  const isConnected = ref(false)

  async function testConnection() {
    try {
      isConnected.value = await invoke<boolean>('test_mysql_connection', {
        config: mysqlConfig.value,
      })
      return isConnected.value
    } catch {
      isConnected.value = false
      return false
    }
  }

  async function loadSettings() {
    try {
      const settings = await invoke<Record<string, string>>('get_settings')
      if (settings.hospital_name) hospitalName.value = settings.hospital_name
      if (settings.staff_list) staffList.value = JSON.parse(settings.staff_list)
    } catch (e) {
      console.error('Failed to load settings:', e)
    }
  }

  return { mysqlConfig, hospitalName, staffList, isConnected, testConnection, loadSettings }
})
