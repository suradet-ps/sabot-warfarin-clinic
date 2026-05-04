pub mod commands;
pub mod db;
pub mod dose;
pub mod encrypt;
pub mod models;

use anyhow::{Context, Result};
use commands::{
  alerts::get_patient_alerts,
  appointments::{get_appointments, schedule_appointment},
  inr::{get_inr_history, get_latest_inr},
  interaction::{
    add_drug_interaction, delete_drug_interaction, get_all_drug_interactions,
    get_patient_drug_interactions, search_hosxp_drugs,
  },
  outcomes::{get_outcomes, record_adverse_event},
  patients::{
    enroll_patient, get_active_patient_summaries, get_active_patients, get_patient_detail,
    update_patient_status,
  },
  reports::{calculate_clinic_ttr, calculate_ttr, get_report_data},
  screening::search_warfarin_patients,
  settings::{get_mysql_config_for_ui, get_setting_value, get_settings, save_setting, test_mysql_connection},
  visits::{
    delete_visit, get_visit_by_id, get_visit_history, save_visit, suggest_dose, update_visit,
  },
};
use db::sqlite::{AppState, init_pool};
use tauri::{App, Manager};

fn initialise_app_state(app: &mut App) -> Result<()> {
  let app_dir = app
    .path()
    .app_data_dir()
    .context("failed to resolve app data directory")?;

  std::fs::create_dir_all(&app_dir).with_context(|| {
    format!(
      "failed to create app data directory at {}",
      app_dir.display()
    )
  })?;

  let db_path = app_dir.join("warfarin.db");
  let pool = tauri::async_runtime::block_on(init_pool(db_path.clone())).with_context(|| {
    format!(
      "failed to initialise SQLite database at {}",
      db_path.display()
    )
  })?;

  app.manage(AppState::new(pool));
  Ok(())
}

pub fn run() -> tauri::Result<()> {
  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_store::Builder::new().build())
    .setup(|app| initialise_app_state(app).map_err(Into::into))
    .invoke_handler(tauri::generate_handler![
      search_warfarin_patients,
      get_active_patients,
      get_active_patient_summaries,
      get_patient_detail,
      enroll_patient,
      update_patient_status,
      get_appointments,
      get_visit_history,
      get_visit_by_id,
      save_visit,
      update_visit,
      delete_visit,
      suggest_dose,
      get_patient_alerts,
      get_outcomes,
      record_adverse_event,
      get_settings,
      save_setting,
      get_setting_value,
      test_mysql_connection,
      get_mysql_config_for_ui,
      get_inr_history,
      get_latest_inr,
      schedule_appointment,
      calculate_ttr,
      calculate_clinic_ttr,
      get_report_data,
      get_all_drug_interactions,
      add_drug_interaction,
      delete_drug_interaction,
      search_hosxp_drugs,
      get_patient_drug_interactions,
    ])
    .run(tauri::generate_context!())
}
