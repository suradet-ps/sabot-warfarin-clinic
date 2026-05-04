import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface DrugInteraction {
  id: number
  icode: string
  drugName: string
  strength: string | null
  interactionType: string
  createdAt: string
  updatedAt: string
}

export interface HosxpDrugItem {
  icode: string
  name: string
  strength: string
  units: string
}

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
  const drugInteractions = ref<DrugInteraction[]>([])

  async function testConnection() {
    try {
      isConnected.value = await invoke<boolean>('test_mysql_connection', {
        config: mysqlConfig.value,
      })
      return isConnected.value
    } catch (e) {
      console.error('Connection test failed:', e)
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

  async function loadDrugInteractions() {
    try {
      drugInteractions.value = await invoke<DrugInteraction[]>('get_all_drug_interactions')
    } catch (e) {
      console.error('Failed to load drug interactions:', e)
    }
  }

  async function addDrugInteraction(input: {
    icode: string
    drugName: string
    strength: string | null
    interactionType: string
  }) {
    const id = await invoke<number>('add_drug_interaction', { input })
    await loadDrugInteractions()
    return id
  }

  async function deleteDrugInteraction(id: number) {
    await invoke('delete_drug_interaction', { id })
    await loadDrugInteractions()
  }

  async function searchHosxpDrugs(keyword: string): Promise<HosxpDrugItem[]> {
    return invoke<HosxpDrugItem[]>('search_hosxp_drugs', {
      mysqlConfig: mysqlConfig.value,
      keyword,
    })
  }

  return {
    mysqlConfig,
    hospitalName,
    staffList,
    isConnected,
    drugInteractions,
    testConnection,
    loadSettings,
    loadDrugInteractions,
    addDrugInteraction,
    deleteDrugInteraction,
    searchHosxpDrugs,
  }
})