import type { WfAppointment } from '#/types/appointment'
import type { DispensingRecord } from '#/types/dispensing'
import type { InrRecord } from '#/types/inr'
import type { WfOutcome } from '#/types/outcome'
import type { HosxpPatient } from '#/types/patient'
import type { DoseSchedule, WfVisit } from '#/types/visit'

export const doseDayKeys = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'] as const
export type DoseDayKey = (typeof doseDayKeys)[number]

export const doseDayLabels: Record<DoseDayKey, string> = {
  mon: 'จ',
  tue: 'อ',
  wed: 'พ',
  thu: 'พฤ',
  fri: 'ศ',
  sat: 'ส',
  sun: 'อา',
}

export function emptyDoseSchedule(): DoseSchedule {
  return { mon: 0, tue: 0, wed: 0, thu: 0, fri: 0, sat: 0, sun: 0 }
}

export function normalizeDoseSchedule(input?: Partial<DoseSchedule> | string | null): DoseSchedule {
  if (!input) return emptyDoseSchedule()
  if (typeof input === 'string') {
    try {
      return normalizeDoseSchedule(JSON.parse(input) as Partial<DoseSchedule>)
    } catch {
      return emptyDoseSchedule()
    }
  }
  return {
    mon: Number(input.mon ?? 0),
    tue: Number(input.tue ?? 0),
    wed: Number(input.wed ?? 0),
    thu: Number(input.thu ?? 0),
    fri: Number(input.fri ?? 0),
    sat: Number(input.sat ?? 0),
    sun: Number(input.sun ?? 0),
  }
}

export function scheduleWeeklyTotal(schedule?: Partial<DoseSchedule> | string | null): number {
  const normalized = normalizeDoseSchedule(schedule)
  return doseDayKeys.reduce((sum, key) => sum + Number(normalized[key] ?? 0), 0)
}

export function scheduleAverageDose(schedule?: Partial<DoseSchedule> | string | null): number {
  return scheduleWeeklyTotal(schedule) / doseDayKeys.length
}

export function formatThaiDate(value?: string | null): string {
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  const day = `${date.getDate()}`.padStart(2, '0')
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const year = date.getFullYear() + 543
  return `${day}/${month}/${year}`
}

function formatDateInput(date: Date): string {
  const year = `${date.getFullYear()}`
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')
  return `${year}-${month}-${day}`
}

export function dateInputToday(): string {
  return formatDateInput(new Date())
}

export function dateInputYearsAgo(years: number): string {
  const date = new Date()
  date.setFullYear(date.getFullYear() - years)
  return formatDateInput(date)
}

export function calculateAge(birthday?: string | null): number | null {
  if (!birthday) return null
  const birth = new Date(birthday)
  if (Number.isNaN(birth.getTime())) return null
  const today = new Date()
  let age = today.getFullYear() - birth.getFullYear()
  const monthDiff = today.getMonth() - birth.getMonth()
  if (monthDiff < 0 || (monthDiff === 0 && today.getDate() < birth.getDate())) age -= 1
  return age
}

export function sexLabel(sex?: string | null): string {
  if (sex === 'M') return 'ชาย'
  if (sex === 'F') return 'หญิง'
  return '-'
}

export function patientFullName(info?: HosxpPatient | null): string {
  if (!info) return '-'
  return [info.pname, info.fname, info.lname].filter(Boolean).join(' ').trim() || info.hn
}

export function daysUntil(dateValue?: string | null): number | null {
  if (!dateValue) return null
  const date = new Date(dateValue)
  if (Number.isNaN(date.getTime())) return null
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  date.setHours(0, 0, 0, 0)
  return Math.round((date.getTime() - today.getTime()) / (1000 * 60 * 60 * 24))
}

export function getCssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim()
}

export function sortAppointments(appointments: WfAppointment[]): WfAppointment[] {
  return [...appointments].sort((a, b) => `${a.apptDate}`.localeCompare(`${b.apptDate}`))
}

export function sortOutcomes(outcomes: WfOutcome[]): WfOutcome[] {
  return [...outcomes].sort((a, b) => `${b.eventDate}`.localeCompare(`${a.eventDate}`))
}

export function sortDispensing(records: DispensingRecord[]): DispensingRecord[] {
  return [...records].sort((a, b) => `${b.vstdate}`.localeCompare(`${a.vstdate}`))
}

export function latestVisit(visits: WfVisit[]): WfVisit | undefined {
  return [...visits].sort((a, b) => `${b.visitDate}`.localeCompare(`${a.visitDate}`))[0]
}

export function latestInrRecord(records: InrRecord[]): InrRecord | undefined {
  return [...records].sort((a, b) => `${b.date}`.localeCompare(`${a.date}`))[0]
}
