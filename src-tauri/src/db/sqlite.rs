//! SQLite persistence layer for the warfarin clinic application.
//!
//! Uses runtime queries (`sqlx::query()`) throughout so no DATABASE_URL is
//! needed at compile time. All public functions return `anyhow::Result`.

use anyhow::{bail, Context, Result};
use chrono::Utc;
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::path::PathBuf;

use crate::models::{
  appointment::{AppointmentInput, WfAppointment},
  inr::InrRecord,
  outcome::{OutcomeInput, WfOutcome},
  patient::{EnrollmentInput, WfPatient},
  visit::{VisitInput, WfVisit},
};

// Pool initialisation

/// Opens (or creates) the SQLite database and runs embedded migrations.
pub async fn init_pool(db_path: PathBuf) -> Result<SqlitePool> {
  let url = format!("sqlite://{}?mode=rwc", db_path.display());
  let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&url)
    .await
    .with_context(|| format!("failed to open SQLite database at {}", db_path.display()))?;

  sqlx::migrate!("./migrations")
    .run(&pool)
    .await
    .context("failed to run SQLite migrations")?;

  Ok(pool)
}

// wf_patients

/// Inserts a new enrolled patient and returns the new row ID.
pub async fn enroll_patient(pool: &SqlitePool, input: &EnrollmentInput) -> Result<i64> {
  let now = Utc::now().to_rfc3339();
  let id = sqlx::query(
    "INSERT INTO wf_patients \
         (hn, enrolled_at, enrolled_by, status, indication, \
          target_inr_low, target_inr_high, notes, created_at, updated_at) \
         VALUES (?, ?, ?, 'active', ?, ?, ?, ?, ?, ?)",
  )
  .bind(&input.hn)
  .bind(&input.enrolled_at)
  .bind(&input.enrolled_by)
  .bind(&input.indication)
  .bind(input.target_inr_low)
  .bind(input.target_inr_high)
  .bind(&input.notes)
  .bind(&now)
  .bind(&now)
  .execute(pool)
  .await
  .context("failed to enroll patient")?
  .last_insert_rowid();

  Ok(id)
}

/// Returns all active warfarin clinic patients.
pub async fn get_active_patients(pool: &SqlitePool) -> Result<Vec<WfPatient>> {
  let rows = sqlx::query(
    "SELECT id, hn, enrolled_at, enrolled_by, status, indication, \
         target_inr_low, target_inr_high, notes, created_at, updated_at \
         FROM wf_patients WHERE status = 'active' ORDER BY enrolled_at DESC",
  )
  .fetch_all(pool)
  .await
  .context("failed to query active patients")?;

  Ok(
    rows
      .iter()
      .map(|r| WfPatient {
        id: r.get("id"),
        hn: r.get("hn"),
        enrolled_at: r.get("enrolled_at"),
        enrolled_by: r.get("enrolled_by"),
        status: r.get("status"),
        indication: r.get("indication"),
        target_inr_low: r.get("target_inr_low"),
        target_inr_high: r.get("target_inr_high"),
        notes: r.get("notes"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
      })
      .collect(),
  )
}

/// Returns all enrolled warfarin clinic patients regardless of status.
pub async fn get_all_patients(pool: &SqlitePool) -> Result<Vec<WfPatient>> {
  let rows = sqlx::query(
    "SELECT id, hn, enrolled_at, enrolled_by, status, indication, \
         target_inr_low, target_inr_high, notes, created_at, updated_at \
         FROM wf_patients ORDER BY enrolled_at DESC",
  )
  .fetch_all(pool)
  .await
  .context("failed to query all patients")?;

  Ok(
    rows
      .iter()
      .map(|r| WfPatient {
        id: r.get("id"),
        hn: r.get("hn"),
        enrolled_at: r.get("enrolled_at"),
        enrolled_by: r.get("enrolled_by"),
        status: r.get("status"),
        indication: r.get("indication"),
        target_inr_low: r.get("target_inr_low"),
        target_inr_high: r.get("target_inr_high"),
        notes: r.get("notes"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
      })
      .collect(),
  )
}

/// Fetches a single patient by HN.
pub async fn get_patient_by_hn(pool: &SqlitePool, hn: &str) -> Result<Option<WfPatient>> {
  let row = sqlx::query(
    "SELECT id, hn, enrolled_at, enrolled_by, status, indication, \
         target_inr_low, target_inr_high, notes, created_at, updated_at \
         FROM wf_patients WHERE hn = ?",
  )
  .bind(hn)
  .fetch_optional(pool)
  .await
  .context("failed to query patient")?;

  Ok(row.map(|r| WfPatient {
    id: r.get("id"),
    hn: r.get("hn"),
    enrolled_at: r.get("enrolled_at"),
    enrolled_by: r.get("enrolled_by"),
    status: r.get("status"),
    indication: r.get("indication"),
    target_inr_low: r.get("target_inr_low"),
    target_inr_high: r.get("target_inr_high"),
    notes: r.get("notes"),
    created_at: r.get("created_at"),
    updated_at: r.get("updated_at"),
  }))
}

/// Returns all enrolled HNs (any status).
pub async fn get_all_enrolled_hns(pool: &SqlitePool) -> Result<Vec<String>> {
  let rows = sqlx::query("SELECT hn FROM wf_patients")
    .fetch_all(pool)
    .await
    .context("failed to query enrolled HNs")?;
  Ok(rows.iter().map(|r| r.get("hn")).collect())
}

/// Updates a patient's status and records the change metadata.
pub async fn update_patient_status(
  pool: &SqlitePool,
  hn: &str,
  status: &str,
  reason: Option<&str>,
  effective_date: Option<&str>,
) -> Result<()> {
  let now = Utc::now().to_rfc3339();
  let effective_date = effective_date
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(ToOwned::to_owned)
    .unwrap_or_else(|| Utc::now().date_naive().format("%Y-%m-%d").to_string());
  let reason = reason
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(ToOwned::to_owned);

  let mut tx = pool
    .begin()
    .await
    .context("failed to begin patient status update transaction")?;

  let result = sqlx::query("UPDATE wf_patients SET status = ?, updated_at = ? WHERE hn = ?")
    .bind(status)
    .bind(&now)
    .bind(hn)
    .execute(&mut *tx)
    .await
    .context("failed to update patient status")?;

  if result.rows_affected() == 0 {
    bail!("patient not found: {hn}");
  }

  sqlx::query(
    "INSERT INTO wf_patient_status_history (hn, status, reason, effective_date, created_at) \
         VALUES (?, ?, ?, ?, ?)",
  )
  .bind(hn)
  .bind(status)
  .bind(reason)
  .bind(&effective_date)
  .bind(&now)
  .execute(&mut *tx)
  .await
  .context("failed to record patient status history")?;

  tx.commit()
    .await
    .context("failed to commit patient status update")?;

  Ok(())
}

// wf_visits

/// Inserts a visit record and returns the new row ID.
pub async fn save_visit(pool: &SqlitePool, input: &VisitInput) -> Result<i64> {
  let now = Utc::now().to_rfc3339();
  let dose_detail_json = input
    .dose_detail
    .as_ref()
    .map(|d| serde_json::to_string(d).unwrap_or_default());
  let new_dose_detail_json = input
    .new_dose_detail
    .as_ref()
    .map(|d| serde_json::to_string(d).unwrap_or_default());
  let side_effects_json = input
    .side_effects
    .as_ref()
    .map(|s| serde_json::to_string(s).unwrap_or_default());
  let dose_changed = i32::from(input.dose_changed);

  let id = sqlx::query(
    "INSERT INTO wf_visits \
         (hn, visit_date, inr_value, inr_source, \
          current_dose_mgday, dose_detail, new_dose_mgday, new_dose_detail, \
          dose_changed, next_appointment, next_inr_due, \
          physician, notes, side_effects, adherence, created_by, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
  )
  .bind(&input.hn)
  .bind(&input.visit_date)
  .bind(input.inr_value)
  .bind(&input.inr_source)
  .bind(input.current_dose_mgday)
  .bind(&dose_detail_json)
  .bind(input.new_dose_mgday)
  .bind(&new_dose_detail_json)
  .bind(dose_changed)
  .bind(&input.next_appointment)
  .bind(&input.next_inr_due)
  .bind(&input.physician)
  .bind(&input.notes)
  .bind(&side_effects_json)
  .bind(&input.adherence)
  .bind(&input.created_by)
  .bind(&now)
  .execute(pool)
  .await
  .context("failed to save visit")?
  .last_insert_rowid();

  Ok(id)
}

/// Returns all visit records for a patient, newest first.
pub async fn get_visit_history(pool: &SqlitePool, hn: &str) -> Result<Vec<WfVisit>> {
  let rows = sqlx::query(
    "SELECT id, hn, visit_date, inr_value, inr_source, \
         current_dose_mgday, dose_detail, new_dose_mgday, new_dose_detail, \
         dose_changed, next_appointment, next_inr_due, \
         physician, notes, side_effects, adherence, created_by, created_at \
         FROM wf_visits WHERE hn = ? ORDER BY visit_date DESC",
  )
  .bind(hn)
  .fetch_all(pool)
  .await
  .context("failed to query visit history")?;

  rows
    .iter()
    .map(|r| {
      let dose_detail = r
        .try_get::<Option<String>, _>("dose_detail")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok());
      let new_dose_detail = r
        .try_get::<Option<String>, _>("new_dose_detail")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok());
      let side_effects = r
        .try_get::<Option<String>, _>("side_effects")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok());
      let dose_changed: i32 = r.try_get("dose_changed").unwrap_or(0);
      Ok(WfVisit {
        id: r.get("id"),
        hn: r.get("hn"),
        visit_date: r.get("visit_date"),
        inr_value: r.try_get("inr_value").ok(),
        inr_source: r.try_get("inr_source").ok(),
        current_dose_mgday: r.try_get("current_dose_mgday").ok(),
        dose_detail,
        new_dose_mgday: r.try_get("new_dose_mgday").ok(),
        new_dose_detail,
        dose_changed: dose_changed != 0,
        next_appointment: r.try_get("next_appointment").ok(),
        next_inr_due: r.try_get("next_inr_due").ok(),
        physician: r.try_get("physician").ok(),
        notes: r.try_get("notes").ok(),
        side_effects,
        adherence: r.try_get("adherence").ok(),
        created_by: r.try_get("created_by").ok(),
        created_at: r.get("created_at"),
      })
    })
    .collect()
}

pub async fn get_visit_by_id(pool: &SqlitePool, visit_id: i64) -> Result<Option<WfVisit>> {
  let row = sqlx::query(
    "SELECT id, hn, visit_date, inr_value, inr_source, \
         current_dose_mgday, dose_detail, new_dose_mgday, new_dose_detail, \
         dose_changed, next_appointment, next_inr_due, \
         physician, notes, side_effects, adherence, created_by, created_at \
         FROM wf_visits WHERE id = ?",
  )
  .bind(visit_id)
  .fetch_optional(pool)
  .await
  .context("failed to query visit by id")?;

  Ok(row.as_ref().map(|r| {
    let dose_detail = r
      .try_get::<Option<String>, _>("dose_detail")
      .ok()
      .flatten()
      .and_then(|s| serde_json::from_str(&s).ok());
    let new_dose_detail = r
      .try_get::<Option<String>, _>("new_dose_detail")
      .ok()
      .flatten()
      .and_then(|s| serde_json::from_str(&s).ok());
    let side_effects = r
      .try_get::<Option<String>, _>("side_effects")
      .ok()
      .flatten()
      .and_then(|s| serde_json::from_str(&s).ok());
    let dose_changed: i32 = r.try_get("dose_changed").unwrap_or(0);
    WfVisit {
      id: r.get("id"),
      hn: r.get("hn"),
      visit_date: r.get("visit_date"),
      inr_value: r.try_get("inr_value").ok(),
      inr_source: r.try_get("inr_source").ok(),
      current_dose_mgday: r.try_get("current_dose_mgday").ok(),
      dose_detail,
      new_dose_mgday: r.try_get("new_dose_mgday").ok(),
      new_dose_detail,
      dose_changed: dose_changed != 0,
      next_appointment: r.try_get("next_appointment").ok(),
      next_inr_due: r.try_get("next_inr_due").ok(),
      physician: r.try_get("physician").ok(),
      notes: r.try_get("notes").ok(),
      side_effects,
      adherence: r.try_get("adherence").ok(),
      created_by: r.try_get("created_by").ok(),
      created_at: r.get("created_at"),
    }
  }))
}

/// Returns INR values recorded via the clinic visit form (fallback).
pub async fn get_inr_from_visits(pool: &SqlitePool, hn: &str) -> Result<Vec<InrRecord>> {
  let rows = sqlx::query(
    "SELECT visit_date, inr_value, inr_source FROM wf_visits \
         WHERE hn = ? AND inr_value IS NOT NULL ORDER BY visit_date ASC",
  )
  .bind(hn)
  .fetch_all(pool)
  .await
  .context("failed to query INR from visits")?;

  Ok(
    rows
      .iter()
      .filter_map(|r| {
        let value: Option<f64> = r.try_get("inr_value").ok();
        value.map(|v| InrRecord {
          date: r.get("visit_date"),
          value: v,
          source: r
            .try_get::<Option<String>, _>("inr_source")
            .ok()
            .flatten()
            .unwrap_or_else(|| "manual".to_string()),
          lab_order_number: None,
          vn: None,
        })
      })
      .collect(),
  )
}

// wf_appointments

/// Inserts a new appointment and returns the new row ID.
pub async fn schedule_appointment(pool: &SqlitePool, input: &AppointmentInput) -> Result<i64> {
  let now = Utc::now().to_rfc3339();
  let id = sqlx::query(
    "INSERT INTO wf_appointments (hn, appt_date, appt_type, status, notes, created_at) \
         VALUES (?, ?, ?, 'scheduled', ?, ?)",
  )
  .bind(&input.hn)
  .bind(&input.appt_date)
  .bind(&input.appt_type)
  .bind(&input.notes)
  .bind(&now)
  .execute(pool)
  .await
  .context("failed to schedule appointment")?
  .last_insert_rowid();

  Ok(id)
}

/// Returns all appointments for a patient, sorted by date.
pub async fn get_appointments(pool: &SqlitePool, hn: &str) -> Result<Vec<WfAppointment>> {
  let rows = sqlx::query(
    "SELECT id, hn, appt_date, appt_type, status, notes, created_at \
         FROM wf_appointments WHERE hn = ? ORDER BY appt_date ASC",
  )
  .bind(hn)
  .fetch_all(pool)
  .await
  .context("failed to query appointments")?;

  Ok(
    rows
      .iter()
      .map(|r| WfAppointment {
        id: r.get("id"),
        hn: r.get("hn"),
        appt_date: r.get("appt_date"),
        appt_type: r.try_get("appt_type").ok(),
        status: r.get("status"),
        notes: r.try_get("notes").ok(),
        created_at: r.get("created_at"),
      })
      .collect(),
  )
}

/// Returns all pending (scheduled) appointments across all patients.
pub async fn get_pending_appointments(pool: &SqlitePool) -> Result<Vec<WfAppointment>> {
  let rows = sqlx::query(
    "SELECT id, hn, appt_date, appt_type, status, notes, created_at \
         FROM wf_appointments WHERE status = 'scheduled' ORDER BY appt_date ASC",
  )
  .fetch_all(pool)
  .await
  .context("failed to query pending appointments")?;

  Ok(
    rows
      .iter()
      .map(|r| WfAppointment {
        id: r.get("id"),
        hn: r.get("hn"),
        appt_date: r.get("appt_date"),
        appt_type: r.try_get("appt_type").ok(),
        status: r.get("status"),
        notes: r.try_get("notes").ok(),
        created_at: r.get("created_at"),
      })
      .collect(),
  )
}

// wf_outcomes

pub async fn record_adverse_event(pool: &SqlitePool, input: &OutcomeInput) -> Result<i64> {
  let now = Utc::now().to_rfc3339();
  let id = sqlx::query(
    "INSERT INTO wf_outcomes \
         (hn, event_date, event_type, description, inr_at_event, action_taken, created_by, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
  )
  .bind(&input.hn)
  .bind(&input.event_date)
  .bind(&input.event_type)
  .bind(&input.description)
  .bind(input.inr_at_event)
  .bind(&input.action_taken)
  .bind(&input.created_by)
  .bind(&now)
  .execute(pool)
  .await
  .context("failed to record adverse event")?
  .last_insert_rowid();

  Ok(id)
}

pub async fn get_outcomes(pool: &SqlitePool, hn: &str) -> Result<Vec<WfOutcome>> {
  let rows = sqlx::query(
    "SELECT id, hn, event_date, event_type, description, inr_at_event, action_taken, created_by, created_at \
         FROM wf_outcomes WHERE hn = ? ORDER BY event_date DESC, id DESC",
  )
  .bind(hn)
  .fetch_all(pool)
  .await
  .context("failed to query outcomes")?;

  Ok(
    rows
      .iter()
      .map(|r| WfOutcome {
        id: r.get("id"),
        hn: r.get("hn"),
        event_date: r.get("event_date"),
        event_type: r.get("event_type"),
        description: r.try_get("description").ok(),
        inr_at_event: r.try_get("inr_at_event").ok(),
        action_taken: r.try_get("action_taken").ok(),
        created_by: r.try_get("created_by").ok(),
        created_at: r.get("created_at"),
      })
      .collect(),
  )
}

// wf_settings

/// Fetches all settings as key-value pairs.
pub async fn get_all_settings(pool: &SqlitePool) -> Result<Vec<(String, String)>> {
  let rows = sqlx::query("SELECT key, value FROM wf_settings ORDER BY key")
    .fetch_all(pool)
    .await
    .context("failed to query settings")?;
  Ok(
    rows
      .iter()
      .map(|r| (r.get("key"), r.get("value")))
      .collect(),
  )
}

/// Upserts a setting value.
pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<()> {
  sqlx::query(
    "INSERT INTO wf_settings (key, value) VALUES (?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
  )
  .bind(key)
  .bind(value)
  .execute(pool)
  .await
  .context("failed to upsert setting")?;
  Ok(())
}

/// Fetches a single setting by key.
pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
  let row = sqlx::query("SELECT value FROM wf_settings WHERE key = ?")
    .bind(key)
    .fetch_optional(pool)
    .await
    .context("failed to query setting")?;
  Ok(row.map(|r| r.get("value")))
}

// AppState

/// Application state managed by Tauri, wrapping the SQLite connection pool.
///
/// Registered with `tauri::Builder::manage()` and injected into every command
/// handler via `tauri::State<'_, AppState>`.
pub struct AppState {
  /// SQLite connection pool.
  pub pool: SqlitePool,
}

impl AppState {
  /// Constructs `AppState` from an already-initialised pool.
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }
}
