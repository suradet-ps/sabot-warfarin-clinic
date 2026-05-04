//! Screening command — queries HosXP warfarin dispensing records.
//!
//! When MySQL is not configured this falls back to an empty response so the UI
//! remains functional.  The HosXP MySQL integration is in `db::mysql`.

use tauri::State;

use crate::{
  db::{
    mysql::{search_hosxp_warfarin_patients},
    sqlite::{AppState, get_all_enrolled_hns},
  },
  models::patient::{SearchFilters, SearchResponse},
};

#[tauri::command]
pub async fn search_warfarin_patients(
  filters: SearchFilters,
  state: State<'_, AppState>,
) -> Result<SearchResponse, String> {
  // Get all enrolled HNs from SQLite to flag patients already in the clinic.
  let enrolled_hns = get_all_enrolled_hns(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

  let config = crate::commands::settings::get_mysql_config_internal(&state.pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "ยังไม่ได้ตั้งค่าการเชื่อมต่อ HosXP".to_string())?;

  search_hosxp_warfarin_patients(&config, &filters, &enrolled_hns)
    .await
    .map_err(|e| format!("failed to search HosXP warfarin patients: {:#}", e))
}
