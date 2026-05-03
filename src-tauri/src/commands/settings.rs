//! Settings and MySQL connection-test commands.

use std::collections::HashMap;

use tauri::State;

use crate::db::{
  mysql::{DbConfig, test_mysql_connection as db_test_connection},
  sqlite::{AppState, get_all_settings, get_setting, set_setting},
};

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
  let pairs = get_all_settings(&state.pool)
    .await
    .map_err(|e| e.to_string())?;
  Ok(pairs.into_iter().collect())
}

#[tauri::command]
pub async fn save_setting(
  key: String,
  value: String,
  state: State<'_, AppState>,
) -> Result<(), String> {
  set_setting(&state.pool, &key, &value)
    .await
    .map_err(|e| e.to_string())
}

/// Persists the MySQL config JSON and verifies the connection.
#[tauri::command]
pub async fn test_mysql_connection(
  config: DbConfig,
  state: State<'_, AppState>,
) -> Result<bool, String> {
  let ok = db_test_connection(&config).await;
  if ok {
    let json = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    set_setting(&state.pool, "mysql_config", &json)
      .await
      .map_err(|e| e.to_string())?;
  }
  Ok(ok)
}

/// Returns a specific setting value.
#[tauri::command]
pub async fn get_setting_value(
  key: String,
  state: State<'_, AppState>,
) -> Result<Option<String>, String> {
  get_setting(&state.pool, &key)
    .await
    .map_err(|e| e.to_string())
}
