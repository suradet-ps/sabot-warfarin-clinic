use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispensingRecord {
  pub hn: String,
  pub vstdate: String,
  pub icode: String,
  pub drug_name: String,
  pub strength: String,
  pub qty: f64,
  pub unitprice: f64,
}
