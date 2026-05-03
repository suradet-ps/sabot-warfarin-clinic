//! Visit management commands — save, query, and dose suggestion.

use tauri::State;

use crate::{
  db::sqlite::{
    delete_visit as db_delete_visit, get_visit_by_id as db_get_visit_by_id,
    get_visit_history as db_history, save_visit as db_save, AppState,
  },
  dose::calculator::suggest_dose as suggest_dose_impl,
  models::visit::{DoseSuggestion, VisitInput, WfVisit},
};

#[tauri::command]
pub async fn get_visit_history(
  hn: String,
  state: State<'_, AppState>,
) -> Result<Vec<WfVisit>, String> {
  db_history(&state.pool, &hn)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_visit_by_id(visit_id: i64, state: State<'_, AppState>) -> Result<WfVisit, String> {
  db_get_visit_by_id(&state.pool, visit_id)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("visit not found: {visit_id}"))
}

#[tauri::command]
pub async fn save_visit(visit: VisitInput, state: State<'_, AppState>) -> Result<i64, String> {
  db_save(&state.pool, &visit)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn suggest_dose(
  current_dose: f64,
  current_inr: f64,
  target_low: f64,
  target_high: f64,
) -> Result<DoseSuggestion, String> {
  Ok(suggest_dose_impl(
    current_dose,
    current_inr,
    target_low,
    target_high,
  ))
}

#[tauri::command]
pub async fn delete_visit(visit_id: i64, state: State<'_, AppState>) -> Result<(), String> {
  db_delete_visit(&state.pool, visit_id)
    .await
    .map_err(|e| e.to_string())
}
