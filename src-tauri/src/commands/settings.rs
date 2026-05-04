//! Settings and MySQL connection-test commands.

use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use tauri::State;

use crate::db::{
  mysql::{DbConfig, test_mysql_connection as db_test_connection},
  sqlite::{AppState, get_all_settings, get_setting, set_setting},
};
use crate::encrypt;

const MYSQL_CONFIG_KEY: &str = "mysql_config";
const ENCRYPTION_KEY_KEY: &str = "encryption_key";

async fn get_or_create_encryption_key(
  pool: &sqlx::SqlitePool,
) -> Result<[u8; 32], String> {
  let existing = get_setting(pool, ENCRYPTION_KEY_KEY)
    .await
    .map_err(|e| e.to_string())?;

  if let Some(key_b64) = existing {
    let key_bytes = BASE64
      .decode(&key_b64)
      .map_err(|e| e.to_string())?;
    let mut key = [0u8; 32];
    if key_bytes.len() != 32 {
      return Err("Invalid encryption key length".to_string());
    }
    key.copy_from_slice(&key_bytes);
    Ok(key)
  } else {
    let key = encrypt::generate_key();
    let key_b64 = BASE64.encode(&key);
    set_setting(pool, ENCRYPTION_KEY_KEY, &key_b64)
      .await
      .map_err(|e| e.to_string())?;
    Ok(key)
  }
}

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
/// Password is encrypted before storage.
#[tauri::command]
pub async fn test_mysql_connection(
  config: DbConfig,
  state: State<'_, AppState>,
) -> Result<bool, String> {
  let ok = db_test_connection(&config).await;
  if ok {
    let key = get_or_create_encryption_key(&state.pool).await?;
    let encrypted_json = encrypt::encrypt_json(&config, &key)?;
    set_setting(&state.pool, MYSQL_CONFIG_KEY, &encrypted_json)
      .await
      .map_err(|e| e.to_string())?;
  }
  Ok(ok)
}

/// Returns the MySQL config with decrypted password for UI display/editing.
/// Tries to decrypt as encrypted format first, falls back to plaintext for backward compatibility.
#[tauri::command]
pub async fn get_mysql_config_for_ui(state: State<'_, AppState>) -> Result<Option<DbConfig>, String> {
  let stored = match get_setting(&state.pool, MYSQL_CONFIG_KEY)
    .await
    .map_err(|e| e.to_string())?
  {
    Some(v) => v,
    None => return Ok(None),
  };

  // Try encrypted format first
  if let Ok(key) = get_or_create_encryption_key(&state.pool).await {
    if let Ok(config) = encrypt::decrypt_json::<DbConfig>(&stored, &key) {
      return Ok(Some(config));
    }
  }

  // Fallback: try plaintext (backward compatibility)
  let config: DbConfig = serde_json::from_str(&stored).map_err(|e| e.to_string())?;
  Ok(Some(config))
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

/// Internal helper: returns decrypted MySQL config for use by other commands.
/// This handles both encrypted (new) and plaintext (legacy) formats.
pub async fn get_mysql_config_internal(
  pool: &sqlx::SqlitePool,
) -> Result<Option<DbConfig>, String> {
  let stored = match get_setting(pool, MYSQL_CONFIG_KEY)
    .await
    .map_err(|e| e.to_string())?
  {
    Some(v) => v,
    None => return Ok(None),
  };

  // Try encrypted format first
  if let Ok(key) = get_or_create_encryption_key(pool).await {
    if let Ok(config) = encrypt::decrypt_json::<DbConfig>(&stored, &key) {
      return Ok(Some(config));
    }
  }

  // Fallback: try plaintext (backward compatibility)
  let config: DbConfig = serde_json::from_str(&stored).map_err(|e| e.to_string())?;
  Ok(Some(config))
}