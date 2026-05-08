use anyhow::Context;
use reqwest::{Client, Url};
use serde_json::json;
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

use crate::db::sqlite::AppState;
use crate::{
  encrypt::{decrypt_value, encrypt_value},
  models::sync::{
    SyncResult, SyncStatus, SyncSummary, WfAppointmentSync, WfDoseHistorySync, WfOutcomeSync,
    WfPatientStatusHistorySync, WfPatientSync, WfVisitSync,
  },
};

const STORE_FILE: &str = "config.json";
const SUPABASE_URL_KEY: &str = "supabase_url";
const SUPABASE_ANON_KEY_KEY: &str = "supabase_anon_key_enc";
const MACHINE_ID_KEY: &str = "machine_id";
const LAST_PULL_AT_KEY: &str = "last_pull_at";
const LAST_SYNC_AT_KEY: &str = "last_sync_at";

pub(crate) fn get_or_create_machine_id(app: &AppHandle) -> Result<String, String> {
  let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
  if let Some(machine_id) = store
    .get(MACHINE_ID_KEY)
    .and_then(|value| value.as_str().map(str::to_owned))
  {
    return Ok(machine_id);
  }

  let machine_id = Uuid::new_v4().to_string();
  store.set(MACHINE_ID_KEY, json!(machine_id));
  store.save().map_err(|e| e.to_string())?;
  Ok(machine_id)
}

fn get_supabase_config(app: &AppHandle) -> Result<(String, String), String> {
  let machine_id = get_or_create_machine_id(app)?;
  let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;

  let url = store
    .get(SUPABASE_URL_KEY)
    .and_then(|value| value.as_str().map(str::to_owned))
    .as_deref()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .ok_or_else(|| "Supabase URL is not configured".to_string())?
    .trim_end_matches('/')
    .to_string();

  let encrypted_key = store
    .get(SUPABASE_ANON_KEY_KEY)
    .and_then(|value| value.as_str().map(str::to_owned))
    .ok_or_else(|| "Supabase anon key is not configured".to_string())?;

  let anon_key = decrypt_value(&encrypted_key, &machine_id)?;
  Ok((url, anon_key))
}

fn supabase_client() -> Client {
  Client::new()
}

fn build_rest_url(base_url: &str, table: &str, query: &[(&str, String)]) -> Result<Url, String> {
  let mut url = Url::parse(&format!(
    "{}/rest/v1/{}",
    base_url.trim_end_matches('/'),
    table
  ))
  .map_err(|e| e.to_string())?;
  if !query.is_empty() {
    url
      .query_pairs_mut()
      .extend_pairs(query.iter().map(|(key, value)| (*key, value.as_str())));
  }
  Ok(url)
}

fn with_auth(builder: reqwest::RequestBuilder, anon_key: &str) -> reqwest::RequestBuilder {
  builder
    .header("apikey", anon_key)
    .header("Authorization", format!("Bearer {anon_key}"))
}

async fn ensure_sync_ids(
  pool: &sqlx::SqlitePool,
  table: &str,
  machine_id: &str,
) -> Result<(), String> {
  let select_sql = format!("SELECT id FROM {table} WHERE sync_id IS NULL");
  let ids = sqlx::query_scalar::<_, i64>(&select_sql)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

  for row_id in ids {
    let update_sql =
      format!("UPDATE {table} SET sync_id = ?, machine_id = COALESCE(machine_id, ?) WHERE id = ?");
    sqlx::query(&update_sql)
      .bind(Uuid::new_v4().to_string())
      .bind(machine_id)
      .bind(row_id)
      .execute(pool)
      .await
      .map_err(|e| e.to_string())?;
  }

  Ok(())
}

async fn mark_synced(pool: &sqlx::SqlitePool, table: &str, synced_at: &str) -> Result<(), String> {
  let update_sql = format!(
    "UPDATE {table} SET synced_at = ? WHERE sync_id IS NOT NULL AND (synced_at IS NULL OR updated_at > synced_at)"
  );
  sqlx::query(&update_sql)
    .bind(synced_at)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
  Ok(())
}

async fn push_rows<T>(
  client: &Client,
  url: &str,
  anon_key: &str,
  table: &str,
  rows: &[T],
) -> Result<(), String>
where
  T: serde::Serialize + Sync,
{
  if rows.is_empty() {
    return Ok(());
  }

  let endpoint = build_rest_url(url, table, &[("on_conflict", "sync_id".to_string())])?;
  let response = with_auth(client.post(endpoint), anon_key)
    .header("Prefer", "resolution=merge-duplicates,return=minimal")
    .json(rows)
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if response.status().is_success() {
    return Ok(());
  }

  Err(
    response
      .text()
      .await
      .unwrap_or_else(|_| "request failed".to_string()),
  )
}

#[tauri::command]
pub async fn save_supabase_config(
  app: AppHandle,
  url: String,
  anon_key: String,
) -> Result<(), String> {
  let normalized_url = url.trim().trim_end_matches('/').to_string();
  if normalized_url.is_empty() {
    return Err("Supabase URL is required".to_string());
  }

  let machine_id = get_or_create_machine_id(&app)?;
  let encrypted_key = encrypt_value(anon_key.trim(), &machine_id)?;
  let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;

  store.set(SUPABASE_URL_KEY, json!(normalized_url));
  store.set(SUPABASE_ANON_KEY_KEY, json!(encrypted_key));
  store.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_supabase_connection(url: String, anon_key: String) -> Result<bool, String> {
  let endpoint = build_rest_url(
    url.trim().trim_end_matches('/'),
    "wf_patients",
    &[("limit", "1".to_string())],
  )?;
  let response = with_auth(supabase_client().get(endpoint), anon_key.trim())
    .send()
    .await
    .map_err(|e| e.to_string())?;
  Ok(response.status().is_success())
}

#[tauri::command]
pub async fn get_sync_summary(app: AppHandle) -> Result<SyncSummary, String> {
  let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
  Ok(SyncSummary {
    has_anon_key: store
      .get(SUPABASE_ANON_KEY_KEY)
      .and_then(|value| value.as_str().map(str::to_owned))
      .is_some(),
    supabase_url: store
      .get(SUPABASE_URL_KEY)
      .and_then(|value| value.as_str().map(str::to_owned)),
  })
}

#[tauri::command]
pub async fn push_to_supabase(
  app: AppHandle,
  state: State<'_, AppState>,
) -> Result<SyncResult, String> {
  let (url, anon_key) = get_supabase_config(&app)?;
  let machine_id = get_or_create_machine_id(&app)?;
  let client = supabase_client();
  let now = chrono::Utc::now().to_rfc3339();
  let mut result = SyncResult::default();

  ensure_sync_ids(&state.pool, "wf_patients", &machine_id).await?;
  ensure_sync_ids(&state.pool, "wf_visits", &machine_id).await?;
  ensure_sync_ids(&state.pool, "wf_dose_history", &machine_id).await?;
  ensure_sync_ids(&state.pool, "wf_appointments", &machine_id).await?;
  ensure_sync_ids(&state.pool, "wf_outcomes", &machine_id).await?;
  ensure_sync_ids(&state.pool, "wf_patient_status_history", &machine_id).await?;

  let patient_rows: Vec<WfPatientSync> = sqlx::query_as(
    "SELECT sync_id, machine_id, hn, enrolled_at, enrolled_by, status, indication, \
            target_inr_low, target_inr_high, notes, created_at, updated_at, deleted_at \
       FROM wf_patients \
      WHERE sync_id IS NOT NULL AND (synced_at IS NULL OR updated_at > synced_at)",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|e| e.to_string())?;
  if let Err(error) = push_rows(&client, &url, &anon_key, "wf_patients", &patient_rows).await {
    result.errors.push(format!("wf_patients: {error}"));
  } else {
    mark_synced(&state.pool, "wf_patients", &now).await?;
    result.pushed += patient_rows.len();
  }

  let visit_rows: Vec<WfVisitSync> = sqlx::query_as(
    "SELECT sync_id, machine_id, hn, visit_date, inr_value, inr_source, current_dose_mgday, \
            dose_detail, new_dose_mgday, new_dose_detail, new_dose_description, \
            selected_dose_option, dose_changed, next_appointment, next_inr_due, physician, \
            notes, side_effects, adherence, created_by, reviewed_at, reviewed_by, \
            created_at, updated_at, deleted_at \
       FROM wf_visits \
      WHERE sync_id IS NOT NULL AND (synced_at IS NULL OR updated_at > synced_at)",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|e| e.to_string())?;
  if let Err(error) = push_rows(&client, &url, &anon_key, "wf_visits", &visit_rows).await {
    result.errors.push(format!("wf_visits: {error}"));
  } else {
    mark_synced(&state.pool, "wf_visits", &now).await?;
    result.pushed += visit_rows.len();
  }

  let dose_history_rows: Vec<WfDoseHistorySync> = sqlx::query_as(
    "SELECT sync_id, machine_id, hn, changed_at, old_dose_mgday, new_dose_mgday, old_detail, \
            new_detail, reason, inr_at_change, changed_by, created_at, updated_at, deleted_at \
       FROM wf_dose_history \
      WHERE sync_id IS NOT NULL AND (synced_at IS NULL OR updated_at > synced_at)",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|e| e.to_string())?;
  if let Err(error) = push_rows(
    &client,
    &url,
    &anon_key,
    "wf_dose_history",
    &dose_history_rows,
  )
  .await
  {
    result.errors.push(format!("wf_dose_history: {error}"));
  } else {
    mark_synced(&state.pool, "wf_dose_history", &now).await?;
    result.pushed += dose_history_rows.len();
  }

  let appointment_rows: Vec<WfAppointmentSync> = sqlx::query_as(
    "SELECT sync_id, machine_id, hn, appt_date, appt_type, status, notes, source_visit_id, \
            generated_from_visit, created_at, updated_at, deleted_at \
       FROM wf_appointments \
      WHERE sync_id IS NOT NULL AND (synced_at IS NULL OR updated_at > synced_at)",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|e| e.to_string())?;
  if let Err(error) = push_rows(
    &client,
    &url,
    &anon_key,
    "wf_appointments",
    &appointment_rows,
  )
  .await
  {
    result.errors.push(format!("wf_appointments: {error}"));
  } else {
    mark_synced(&state.pool, "wf_appointments", &now).await?;
    result.pushed += appointment_rows.len();
  }

  let outcome_rows: Vec<WfOutcomeSync> = sqlx::query_as(
    "SELECT sync_id, machine_id, hn, event_date, event_type, description, inr_at_event, \
            action_taken, created_by, created_at, updated_at, deleted_at \
       FROM wf_outcomes \
      WHERE sync_id IS NOT NULL AND (synced_at IS NULL OR updated_at > synced_at)",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|e| e.to_string())?;
  if let Err(error) = push_rows(&client, &url, &anon_key, "wf_outcomes", &outcome_rows).await {
    result.errors.push(format!("wf_outcomes: {error}"));
  } else {
    mark_synced(&state.pool, "wf_outcomes", &now).await?;
    result.pushed += outcome_rows.len();
  }

  let history_rows: Vec<WfPatientStatusHistorySync> = sqlx::query_as(
    "SELECT sync_id, machine_id, hn, status, reason, effective_date, created_at, updated_at, deleted_at \
       FROM wf_patient_status_history \
      WHERE sync_id IS NOT NULL AND (synced_at IS NULL OR updated_at > synced_at)",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|e| e.to_string())?;
  if let Err(error) = push_rows(
    &client,
    &url,
    &anon_key,
    "wf_patient_status_history",
    &history_rows,
  )
  .await
  {
    result
      .errors
      .push(format!("wf_patient_status_history: {error}"));
  } else {
    mark_synced(&state.pool, "wf_patient_status_history", &now).await?;
    result.pushed += history_rows.len();
  }

  let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
  store.set(LAST_SYNC_AT_KEY, json!(now));
  store.save().map_err(|e| e.to_string())?;

  Ok(result)
}

#[tauri::command]
pub async fn pull_from_supabase(
  app: AppHandle,
  state: State<'_, AppState>,
) -> Result<SyncResult, String> {
  let (url, anon_key) = get_supabase_config(&app)?;
  let client = supabase_client();
  let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
  let last_pull_at = store
    .get(LAST_PULL_AT_KEY)
    .and_then(|value| value.as_str().map(str::to_owned))
    .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

  let mut result = SyncResult::default();

  let patient_url = build_rest_url(
    &url,
    "wf_patients",
    &[("updated_at", format!("gt.{last_pull_at}"))],
  )?;
  let patient_rows: Vec<WfPatientSync> = with_auth(client.get(patient_url), &anon_key)
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())?;
  for row in &patient_rows {
    let query = sqlx::query(
      "INSERT INTO wf_patients \
          (sync_id, machine_id, hn, enrolled_at, enrolled_by, status, indication, target_inr_low, \
           target_inr_high, notes, created_at, updated_at, deleted_at, synced_at) \
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
       ON CONFLICT(sync_id) DO UPDATE SET \
          machine_id = excluded.machine_id, \
          hn = excluded.hn, \
          enrolled_at = excluded.enrolled_at, \
          enrolled_by = excluded.enrolled_by, \
          status = excluded.status, \
          indication = excluded.indication, \
          target_inr_low = excluded.target_inr_low, \
          target_inr_high = excluded.target_inr_high, \
          notes = excluded.notes, \
          updated_at = excluded.updated_at, \
          deleted_at = excluded.deleted_at, \
          synced_at = excluded.synced_at \
       WHERE excluded.updated_at > wf_patients.updated_at",
    )
    .bind(&row.sync_id)
    .bind(&row.machine_id)
    .bind(&row.hn)
    .bind(&row.enrolled_at)
    .bind(&row.enrolled_by)
    .bind(&row.status)
    .bind(&row.indication)
    .bind(row.target_inr_low)
    .bind(row.target_inr_high)
    .bind(&row.notes)
    .bind(&row.created_at)
    .bind(&row.updated_at)
    .bind(&row.deleted_at)
    .bind(&row.updated_at);
    let affected = query
      .execute(&state.pool)
      .await
      .map_err(|e| e.to_string())?
      .rows_affected();
    if affected > 0 {
      result.pulled += 1;
    } else {
      result.conflicts += 1;
    }
  }

  let visit_url = build_rest_url(
    &url,
    "wf_visits",
    &[("updated_at", format!("gt.{last_pull_at}"))],
  )?;
  let visit_rows: Vec<WfVisitSync> = with_auth(client.get(visit_url), &anon_key)
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())?;
  for row in &visit_rows {
    let affected = sqlx::query(
      "INSERT INTO wf_visits \
          (sync_id, machine_id, hn, visit_date, inr_value, inr_source, current_dose_mgday, \
           dose_detail, new_dose_mgday, new_dose_detail, new_dose_description, selected_dose_option, \
           dose_changed, next_appointment, next_inr_due, physician, notes, side_effects, adherence, \
           created_by, reviewed_at, reviewed_by, created_at, updated_at, deleted_at, synced_at) \
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
       ON CONFLICT(sync_id) DO UPDATE SET \
          machine_id = excluded.machine_id, \
          hn = excluded.hn, \
          visit_date = excluded.visit_date, \
          inr_value = excluded.inr_value, \
          inr_source = excluded.inr_source, \
          current_dose_mgday = excluded.current_dose_mgday, \
          dose_detail = excluded.dose_detail, \
          new_dose_mgday = excluded.new_dose_mgday, \
          new_dose_detail = excluded.new_dose_detail, \
          new_dose_description = excluded.new_dose_description, \
          selected_dose_option = excluded.selected_dose_option, \
          dose_changed = excluded.dose_changed, \
          next_appointment = excluded.next_appointment, \
          next_inr_due = excluded.next_inr_due, \
          physician = excluded.physician, \
          notes = excluded.notes, \
          side_effects = excluded.side_effects, \
          adherence = excluded.adherence, \
          created_by = excluded.created_by, \
          reviewed_at = excluded.reviewed_at, \
          reviewed_by = excluded.reviewed_by, \
          updated_at = excluded.updated_at, \
          deleted_at = excluded.deleted_at, \
          synced_at = excluded.synced_at \
       WHERE excluded.updated_at > wf_visits.updated_at",
    )
    .bind(&row.sync_id)
    .bind(&row.machine_id)
    .bind(&row.hn)
    .bind(&row.visit_date)
    .bind(row.inr_value)
    .bind(&row.inr_source)
    .bind(row.current_dose_mgday)
    .bind(&row.dose_detail)
    .bind(row.new_dose_mgday)
    .bind(&row.new_dose_detail)
    .bind(&row.new_dose_description)
    .bind(&row.selected_dose_option)
    .bind(row.dose_changed)
    .bind(&row.next_appointment)
    .bind(&row.next_inr_due)
    .bind(&row.physician)
    .bind(&row.notes)
    .bind(&row.side_effects)
    .bind(&row.adherence)
    .bind(&row.created_by)
    .bind(&row.reviewed_at)
    .bind(&row.reviewed_by)
    .bind(&row.created_at)
    .bind(&row.updated_at)
    .bind(&row.deleted_at)
    .bind(&row.updated_at)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?
    .rows_affected();
    if affected > 0 {
      result.pulled += 1;
    } else {
      result.conflicts += 1;
    }
  }

  let dose_url = build_rest_url(
    &url,
    "wf_dose_history",
    &[("updated_at", format!("gt.{last_pull_at}"))],
  )?;
  let dose_rows: Vec<WfDoseHistorySync> = with_auth(client.get(dose_url), &anon_key)
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())?;
  for row in &dose_rows {
    let affected = sqlx::query(
      "INSERT INTO wf_dose_history \
          (sync_id, machine_id, hn, changed_at, old_dose_mgday, new_dose_mgday, old_detail, \
           new_detail, reason, inr_at_change, changed_by, created_at, updated_at, deleted_at, synced_at) \
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
       ON CONFLICT(sync_id) DO UPDATE SET \
          machine_id = excluded.machine_id, \
          hn = excluded.hn, \
          changed_at = excluded.changed_at, \
          old_dose_mgday = excluded.old_dose_mgday, \
          new_dose_mgday = excluded.new_dose_mgday, \
          old_detail = excluded.old_detail, \
          new_detail = excluded.new_detail, \
          reason = excluded.reason, \
          inr_at_change = excluded.inr_at_change, \
          changed_by = excluded.changed_by, \
          updated_at = excluded.updated_at, \
          deleted_at = excluded.deleted_at, \
          synced_at = excluded.synced_at \
       WHERE excluded.updated_at > wf_dose_history.updated_at",
    )
    .bind(&row.sync_id)
    .bind(&row.machine_id)
    .bind(&row.hn)
    .bind(&row.changed_at)
    .bind(row.old_dose_mgday)
    .bind(row.new_dose_mgday)
    .bind(&row.old_detail)
    .bind(&row.new_detail)
    .bind(&row.reason)
    .bind(row.inr_at_change)
    .bind(&row.changed_by)
    .bind(&row.created_at)
    .bind(&row.updated_at)
    .bind(&row.deleted_at)
    .bind(&row.updated_at)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?
    .rows_affected();
    if affected > 0 {
      result.pulled += 1;
    } else {
      result.conflicts += 1;
    }
  }

  let appointment_url = build_rest_url(
    &url,
    "wf_appointments",
    &[("updated_at", format!("gt.{last_pull_at}"))],
  )?;
  let appointment_rows: Vec<WfAppointmentSync> = with_auth(client.get(appointment_url), &anon_key)
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())?;
  for row in &appointment_rows {
    let affected = sqlx::query(
      "INSERT INTO wf_appointments \
          (sync_id, machine_id, hn, appt_date, appt_type, status, notes, source_visit_id, \
           generated_from_visit, created_at, updated_at, deleted_at, synced_at) \
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
       ON CONFLICT(sync_id) DO UPDATE SET \
          machine_id = excluded.machine_id, \
          hn = excluded.hn, \
          appt_date = excluded.appt_date, \
          appt_type = excluded.appt_type, \
          status = excluded.status, \
          notes = excluded.notes, \
          source_visit_id = excluded.source_visit_id, \
          generated_from_visit = excluded.generated_from_visit, \
          updated_at = excluded.updated_at, \
          deleted_at = excluded.deleted_at, \
          synced_at = excluded.synced_at \
       WHERE excluded.updated_at > wf_appointments.updated_at",
    )
    .bind(&row.sync_id)
    .bind(&row.machine_id)
    .bind(&row.hn)
    .bind(&row.appt_date)
    .bind(&row.appt_type)
    .bind(&row.status)
    .bind(&row.notes)
    .bind(row.source_visit_id)
    .bind(row.generated_from_visit)
    .bind(&row.created_at)
    .bind(&row.updated_at)
    .bind(&row.deleted_at)
    .bind(&row.updated_at)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?
    .rows_affected();
    if affected > 0 {
      result.pulled += 1;
    } else {
      result.conflicts += 1;
    }
  }

  let outcome_url = build_rest_url(
    &url,
    "wf_outcomes",
    &[("updated_at", format!("gt.{last_pull_at}"))],
  )?;
  let outcome_rows: Vec<WfOutcomeSync> = with_auth(client.get(outcome_url), &anon_key)
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())?;
  for row in &outcome_rows {
    let affected = sqlx::query(
      "INSERT INTO wf_outcomes \
          (sync_id, machine_id, hn, event_date, event_type, description, inr_at_event, action_taken, \
           created_by, created_at, updated_at, deleted_at, synced_at) \
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
       ON CONFLICT(sync_id) DO UPDATE SET \
          machine_id = excluded.machine_id, \
          hn = excluded.hn, \
          event_date = excluded.event_date, \
          event_type = excluded.event_type, \
          description = excluded.description, \
          inr_at_event = excluded.inr_at_event, \
          action_taken = excluded.action_taken, \
          created_by = excluded.created_by, \
          updated_at = excluded.updated_at, \
          deleted_at = excluded.deleted_at, \
          synced_at = excluded.synced_at \
       WHERE excluded.updated_at > wf_outcomes.updated_at",
    )
    .bind(&row.sync_id)
    .bind(&row.machine_id)
    .bind(&row.hn)
    .bind(&row.event_date)
    .bind(&row.event_type)
    .bind(&row.description)
    .bind(row.inr_at_event)
    .bind(&row.action_taken)
    .bind(&row.created_by)
    .bind(&row.created_at)
    .bind(&row.updated_at)
    .bind(&row.deleted_at)
    .bind(&row.updated_at)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?
    .rows_affected();
    if affected > 0 {
      result.pulled += 1;
    } else {
      result.conflicts += 1;
    }
  }

  let history_url = build_rest_url(
    &url,
    "wf_patient_status_history",
    &[("updated_at", format!("gt.{last_pull_at}"))],
  )?;
  let history_rows: Vec<WfPatientStatusHistorySync> = with_auth(client.get(history_url), &anon_key)
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())?;
  for row in &history_rows {
    let affected = sqlx::query(
      "INSERT INTO wf_patient_status_history \
          (sync_id, machine_id, hn, status, reason, effective_date, created_at, updated_at, deleted_at, synced_at) \
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
       ON CONFLICT(sync_id) DO UPDATE SET \
          machine_id = excluded.machine_id, \
          hn = excluded.hn, \
          status = excluded.status, \
          reason = excluded.reason, \
          effective_date = excluded.effective_date, \
          updated_at = excluded.updated_at, \
          deleted_at = excluded.deleted_at, \
          synced_at = excluded.synced_at \
       WHERE excluded.updated_at > wf_patient_status_history.updated_at",
    )
    .bind(&row.sync_id)
    .bind(&row.machine_id)
    .bind(&row.hn)
    .bind(&row.status)
    .bind(&row.reason)
    .bind(&row.effective_date)
    .bind(&row.created_at)
    .bind(&row.updated_at)
    .bind(&row.deleted_at)
    .bind(&row.updated_at)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?
    .rows_affected();
    if affected > 0 {
      result.pulled += 1;
    } else {
      result.conflicts += 1;
    }
  }

  let pulled_at = chrono::Utc::now().to_rfc3339();
  store.set(LAST_PULL_AT_KEY, json!(pulled_at.clone()));
  store.set(LAST_SYNC_AT_KEY, json!(pulled_at));
  store.save().map_err(|e| e.to_string())?;

  Ok(result)
}

#[tauri::command]
pub async fn get_sync_status(
  app: AppHandle,
  state: State<'_, AppState>,
) -> Result<SyncStatus, String> {
  let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
  let machine_id = get_or_create_machine_id(&app)?;

  let pending_count = sqlx::query_scalar::<_, i64>(
    "SELECT \
        (SELECT COUNT(*) FROM wf_patients WHERE synced_at IS NULL OR updated_at > synced_at) + \
        (SELECT COUNT(*) FROM wf_visits WHERE synced_at IS NULL OR updated_at > synced_at) + \
        (SELECT COUNT(*) FROM wf_dose_history WHERE synced_at IS NULL OR updated_at > synced_at) + \
        (SELECT COUNT(*) FROM wf_appointments WHERE synced_at IS NULL OR updated_at > synced_at) + \
        (SELECT COUNT(*) FROM wf_outcomes WHERE synced_at IS NULL OR updated_at > synced_at) + \
        (SELECT COUNT(*) FROM wf_patient_status_history WHERE synced_at IS NULL OR updated_at > synced_at)",
  )
  .fetch_one(&state.pool)
  .await
  .context("failed to calculate sync status")
  .map_err(|e| e.to_string())?;

  Ok(SyncStatus {
    pending_count,
    last_sync_at: store
      .get(LAST_SYNC_AT_KEY)
      .and_then(|value| value.as_str().map(str::to_owned)),
    configured: get_supabase_config(&app).is_ok(),
    machine_id,
  })
}
