import { ref } from 'vue';
import init, { generate_suggestions_rust } from '@/warfarin_logic/pkg/warfarin_logic.js';
import type { RegimenOption, DoseOptionsInput, AvailablePills } from '@/types/dose';
import { DEFAULT_AVAILABLE_PILLS } from '@/types/dose';

const wasmReady = ref(false);
let initPromise: Promise<void> | null = null;

async function initWasm() {
  if (wasmReady.value) return;
  if (initPromise) return initPromise;

  initPromise = (async () => {
    try {
      await init();
      wasmReady.value = true;
      console.log('WASM dose calculator initialized');
    } catch (e) {
      console.error('Failed to initialize WASM module', e);
      throw e;
    }
  })();

  return initPromise;
}

async function generateDoseOptions(
  weeklyDose: number,
  availablePills: AvailablePills,
  allowHalf: boolean,
  specialDayPattern: string,
  daysUntilAppointment: number,
  startDayOfWeek: number
): Promise<RegimenOption[]> {
  if (!wasmReady.value) {
    await initWasm();
  }

  const selectedPills = Object.keys(availablePills)
    .filter(key => availablePills[Number(key)])
    .map(Number);

  if (selectedPills.length === 0) {
    return [];
  }

  if (weeklyDose < 0) {
    return [];
  }

  const input: DoseOptionsInput = {
    weekly_dose: weeklyDose,
    allow_half: allowHalf,
    available_pills: selectedPills,
    special_day_pattern: specialDayPattern,
    days_until_appointment: daysUntilAppointment,
    start_day_of_week: startDayOfWeek,
  };

  try {
    const results = await generate_suggestions_rust(input);
    return results as RegimenOption[];
  } catch (e) {
    console.error('Error generating dose options:', e);
    throw e;
  }
}

export function useDoseCalculator() {
  return {
    wasmReady,
    initWasm,
    generateDoseOptions,
    DEFAULT_AVAILABLE_PILLS,
  };
}