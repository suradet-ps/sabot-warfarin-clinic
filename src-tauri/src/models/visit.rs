use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoseSchedule {
  pub mon: f64,
  pub tue: f64,
  pub wed: f64,
  pub thu: f64,
  pub fri: f64,
  pub sat: f64,
  pub sun: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WfVisit {
  pub id: i64,
  pub hn: String,
  pub visit_date: String,
  pub inr_value: Option<f64>,
  pub inr_source: Option<String>,
  pub current_dose_mgday: Option<f64>,
  pub dose_detail: Option<DoseSchedule>,
  pub new_dose_mgday: Option<f64>,
  pub new_dose_detail: Option<DoseSchedule>,
  pub dose_changed: bool,
  pub next_appointment: Option<String>,
  pub next_inr_due: Option<String>,
  pub physician: Option<String>,
  pub notes: Option<String>,
  pub side_effects: Option<Vec<String>>,
  pub adherence: Option<String>,
  pub created_by: Option<String>,
  pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitInput {
  pub hn: String,
  pub visit_date: String,
  pub inr_value: Option<f64>,
  pub inr_source: Option<String>,
  pub current_dose_mgday: Option<f64>,
  pub dose_detail: Option<DoseSchedule>,
  pub new_dose_mgday: Option<f64>,
  pub new_dose_detail: Option<DoseSchedule>,
  pub dose_changed: bool,
  pub next_appointment: Option<String>,
  pub next_inr_due: Option<String>,
  pub physician: Option<String>,
  pub notes: Option<String>,
  pub side_effects: Option<Vec<String>>,
  pub adherence: Option<String>,
  pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoseSuggestion {
  pub suggested_dose_mgweek: f64,
  pub adjustment_percent: f64,
  pub recommendation: String,
  pub urgency: String,
  pub recheck_days: u32,
}
