//! Appointment scheduling command.

use tauri::State;

use crate::{
  db::sqlite::{
    get_appointments as db_get_appointments, schedule_appointment as db_schedule, AppState,
  },
  models::appointment::AppointmentInput,
};

use crate::models::appointment::WfAppointment;

#[tauri::command]
pub async fn get_appointments(
  hn: String,
  state: State<'_, AppState>,
) -> Result<Vec<WfAppointment>, String> {
  db_get_appointments(&state.pool, &hn)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn schedule_appointment(
  appt: AppointmentInput,
  state: State<'_, AppState>,
) -> Result<i64, String> {
  db_schedule(&state.pool, &appt)
    .await
    .map_err(|e| e.to_string())
}
