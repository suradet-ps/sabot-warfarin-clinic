//! TTR report command.

use serde_json::{Value, json};
use tauri::State;

use crate::{
  commands::patients::get_inr_records,
  db::sqlite::{AppState, get_active_patients, get_outcomes},
  dose::calculator::calculate_ttr as calc_ttr,
};

/// Calculates TTR (Rosendaal method) for a single patient over a given window.
///
/// `window_days` — 0 means all-time.
#[tauri::command]
pub async fn calculate_ttr(
  hn: String,
  window_days: u32,
  state: State<'_, AppState>,
) -> Result<Option<f64>, String> {
  let patient = crate::db::sqlite::get_patient_by_hn(&state.pool, &hn)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("patient not found: {hn}"))?;

  let inr_records = get_inr_records(&state, &hn).await;
  let pairs: Vec<(String, f64)> = inr_records
    .iter()
    .map(|r| (r.date.clone(), r.value))
    .collect();

  let window = if window_days == 0 {
    u32::MAX
  } else {
    window_days
  };
  Ok(calc_ttr(
    &pairs,
    patient.target_inr_low,
    patient.target_inr_high,
    window,
  ))
}

/// Calculates mean TTR across all active patients (for clinic-level report).
#[tauri::command]
pub async fn calculate_clinic_ttr(
  window_days: u32,
  state: State<'_, AppState>,
) -> Result<f64, String> {
  let patients = get_active_patients(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

  let window = if window_days == 0 {
    u32::MAX
  } else {
    window_days
  };
  let mut total = 0.0f64;
  let mut count = 0usize;

  for patient in &patients {
    let inr_records = get_inr_records(&state, &patient.hn).await;
    let pairs: Vec<(String, f64)> = inr_records
      .iter()
      .map(|r| (r.date.clone(), r.value))
      .collect();
    if let Some(ttr) = calc_ttr(
      &pairs,
      patient.target_inr_low,
      patient.target_inr_high,
      window,
    ) {
      total += ttr;
      count += 1;
    }
  }

  if count == 0 {
    Ok(0.0)
  } else {
    Ok(total / count as f64)
  }
}

#[tauri::command]
pub async fn get_report_data(
  report_type: String,
  state: State<'_, AppState>,
) -> Result<Value, String> {
  match report_type.as_str() {
    "census" => {
      let patients = crate::db::sqlite::get_all_patients(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

      let mut active = 0usize;
      let mut inactive = 0usize;
      let mut transferred = 0usize;
      let mut discharged = 0usize;
      let mut deceased = 0usize;

      for patient in &patients {
        match patient.status.as_str() {
          "active" => active += 1,
          "inactive" => inactive += 1,
          "transferred" => transferred += 1,
          "discharged" => discharged += 1,
          "deceased" => deceased += 1,
          _ => {}
        }
      }

      Ok(json!({
        "active": active,
        "inactive": inactive,
        "transferred": transferred,
        "discharged": discharged,
        "deceased": deceased,
        "total": patients.len(),
      }))
    }
    "ttr" => {
      let mean_ttr = calculate_clinic_ttr(182, state).await?;
      Ok(json!({ "meanTtr": mean_ttr }))
    }
    "adverse" => {
      let patients = get_active_patients(&state.pool)
        .await
        .map_err(|e| e.to_string())?;
      let mut total_events = 0usize;

      for patient in &patients {
        let outcomes = get_outcomes(&state.pool, &patient.hn)
          .await
          .map_err(|e| e.to_string())?;
        total_events += outcomes.len();
      }

      Ok(json!({ "totalEvents": total_events }))
    }
    _ => Err(format!("unsupported report type: {report_type}")),
  }
}
