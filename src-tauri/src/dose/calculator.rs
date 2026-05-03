//! Dose suggestion and TTR calculation for warfarin clinic.
//!
//! All functions are pure with no I/O — fully unit-testable.

use crate::models::visit::DoseSuggestion;

/// Rounds a dose value to the nearest 0.5 mg/day practical step.
fn round_to_half_mg(value: f64) -> f64 {
  (value * 2.0).round() / 2.0
}

/// Computes a warfarin dose adjustment suggestion given the current dose,
/// current INR, and the patient's target INR range.
///
/// # Algorithm (per Thai clinical practice guidelines)
///
/// | INR range   | Adjustment | Urgency | Recheck |
/// |-------------|------------|---------|---------|
/// | < 1.5       | +10-20%    | urgent  | 7-14 d  |
/// | 1.5 - 1.9   | +5-10%     | caution | 14 d    |
/// | 2.0 - 3.0   | 0%         | normal  | 28-42 d |
/// | 3.1 - 3.9   | -5-10%     | caution | 14 d    |
/// | 4.0 - 4.9   | Hold 1d, -10% | hold | 7 d    |
/// | 5.0 - 8.9   | Hold 1-2d, Vit K, -10% | hold | 3-7 d |
/// | >= 9.0      | Hold 1-2d, Vit K 1-10mg, -10% | hold | 1-3 d |
pub fn suggest_dose(
  current_dose: f64,
  inr: f64,
  _target_low: f64,
  _target_high: f64,
) -> DoseSuggestion {
  let (adjustment_percent, recommendation, urgency, recheck_days): (f64, &str, &str, u32) =
    if inr >= 9.0 {
      (
        -10.0,
        "หยุดยา 1-2 วัน และให้ Vitamin K 1-10 mg PO และลดขนาดยา 10%",
        "hold",
        2,
      )
    } else if inr >= 5.0 {
      (
        -10.0,
        "หยุดยา 1-2 วัน และให้ Vitamin K 1 mg PO และลดขนาดยา 10%",
        "hold",
        5,
      )
    } else if inr >= 4.0 {
      (
        -10.0,
        "หยุดยา 1 วัน และลดขนาดยา 10%",
        "hold",
        7,
      )
    } else if inr > 3.0 {
      (
        -7.5,
        "ลดขนาดยา 5-10% นัดตรวจ INR ใหม่ใน 14 วัน",
        "caution",
        14,
      )
    } else if inr >= 2.0 {
      (
        0.0,
        "คงขนาดยาเดิม นัดตรวจ INR ใน 4-6 สัปดาห์",
        "normal",
        35,
      )
    } else if inr >= 1.5 {
      (
        7.5,
        "เพิ่มขนาดยา 5-10% นัดตรวจ INR ใหม่ใน 14 วัน",
        "caution",
        14,
      )
    } else {
      (
        15.0,
        "เพิ่มขนาดยา 10-20% นัดตรวจ INR ใหม่ใน 7-14 วัน",
        "urgent",
        10,
      )
    };

  let suggested_dose_mgday =
    round_to_half_mg(current_dose * (1.0 + adjustment_percent / 100.0)).max(0.0);

  DoseSuggestion {
    suggested_dose_mgday,
    adjustment_percent,
    recommendation: recommendation.to_string(),
    urgency: urgency.to_string(),
    recheck_days,
  }
}

/// Calculates Time in Therapeutic Range (TTR) using the Rosendaal linear
/// interpolation method.
///
/// # Arguments
/// * `inr_records` — slice of `(date_str, inr_value)` pairs, any order.
/// * `target_low` — lower bound of therapeutic range.
/// * `target_high` — upper bound of therapeutic range.
/// * `window_days` — only consider INR readings within this many days from today.
///   Pass `u32::MAX` to use all available data.
///
/// # Returns
/// TTR as a percentage (0.0 – 100.0), or `None` if there are fewer than 2
/// readings in the window.
pub fn calculate_ttr(
  inr_records: &[(String, f64)],
  target_low: f64,
  target_high: f64,
  window_days: u32,
) -> Option<f64> {
  // Parse and sort records chronologically.
  let mut records: Vec<(chrono::NaiveDate, f64)> = inr_records
    .iter()
    .filter_map(|(date_str, value)| {
      chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .ok()
        .map(|d| (d, *value))
    })
    .collect();
  records.sort_by_key(|(d, _)| *d);

  // Apply window filter.
  if window_days < u32::MAX {
    let cutoff = chrono::Utc::now().date_naive() - chrono::Duration::days(i64::from(window_days));
    records.retain(|(d, _)| *d >= cutoff);
  }

  if records.len() < 2 {
    return None;
  }

  let mut total_days: i64 = 0;
  let mut in_range_days: i64 = 0;

  for window in records.windows(2) {
    let (d1, inr1) = window[0];
    let (d2, inr2) = window[1];

    let span = (d2 - d1).num_days();
    if span <= 0 {
      continue;
    }

    // For each day in the interval, linearly interpolate the INR.
    for day_offset in 0..span {
      let fraction = day_offset as f64 / span as f64;
      let interpolated_inr = inr1 + fraction * (inr2 - inr1);

      total_days += 1;
      if interpolated_inr >= target_low && interpolated_inr <= target_high {
        in_range_days += 1;
      }
    }
  }

  if total_days == 0 {
    return None;
  }

  Some((in_range_days as f64 / total_days as f64) * 100.0)
}

#[cfg(test)]
mod tests {
  use super::*;

  // ── suggest_dose tests ──────────────────────────────────────────────────

  #[test]
  fn suggest_dose_in_range_returns_no_change() {
    let result = suggest_dose(5.0, 2.5, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, 0.0);
    assert_eq!(result.urgency, "normal");
    assert_eq!(result.suggested_dose_mgday, 5.0);
    assert_eq!(result.recheck_days, 35);
  }

  #[test]
  fn suggest_dose_above_3_0_decreases_7_5_percent() {
    let result = suggest_dose(5.0, 3.3, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, -7.5);
    assert_eq!(result.urgency, "caution");
    // 5.0 * 0.925 = 4.625 → rounded to 4.5
    assert_eq!(result.suggested_dose_mgday, 4.5);
  }

  #[test]
  fn suggest_dose_above_3_0_still_decreases() {
    let result = suggest_dose(5.0, 3.7, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, -7.5);
    assert_eq!(result.urgency, "caution");
    // 5.0 * 0.925 = 4.625 → rounded to 4.5
    assert_eq!(result.suggested_dose_mgday, 4.5);
  }

  #[test]
  fn suggest_dose_4_to_5_hold_and_reduce_10() {
    let result = suggest_dose(5.0, 4.5, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, -10.0);
    assert_eq!(result.urgency, "hold");
    // 5.0 * 0.90 = 4.5
    assert_eq!(result.suggested_dose_mgday, 4.5);
    assert_eq!(result.recheck_days, 7);
  }

  #[test]
  fn suggest_dose_over_5_hold_vit_k() {
    let result = suggest_dose(5.0, 5.5, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, -10.0);
    assert_eq!(result.urgency, "hold");
    // 5.0 * 0.90 = 4.5
    assert_eq!(result.suggested_dose_mgday, 4.5);
    assert_eq!(result.recheck_days, 5);
  }

  #[test]
  fn suggest_dose_1_5_to_1_9_increases_7_5_percent() {
    let result = suggest_dose(5.0, 1.8, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, 7.5);
    assert_eq!(result.urgency, "caution");
    // 5.0 * 1.075 = 5.375 → rounded to 5.5
    assert_eq!(result.suggested_dose_mgday, 5.5);
  }

  #[test]
  fn suggest_dose_below_1_5_increases_15_percent() {
    let result = suggest_dose(5.0, 1.3, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, 15.0);
    assert_eq!(result.urgency, "urgent");
    // 5.0 * 1.15 = 5.75 → rounded to 6.0
    assert_eq!(result.suggested_dose_mgday, 6.0);
  }

  #[test]
  fn suggest_dose_rounds_to_half_mg() {
    // 3.0 * 1.075 = 3.225 → rounds to 3.0
    let result = suggest_dose(3.0, 1.8, 2.0, 3.0);
    assert_eq!(result.suggested_dose_mgday, 3.0);
  }

  #[test]
  fn suggest_dose_zero_dose_stays_zero() {
    let result = suggest_dose(0.0, 1.0, 2.0, 3.0);
    assert_eq!(result.suggested_dose_mgday, 0.0);
  }

  // ── calculate_ttr tests ─────────────────────────────────────────────────

  #[test]
  fn ttr_all_in_range_returns_100() {
    let records = vec![
      ("2024-01-01".to_string(), 2.5),
      ("2024-01-08".to_string(), 2.5),
      ("2024-01-15".to_string(), 2.5),
    ];
    let ttr = calculate_ttr(&records, 2.0, 3.0, u32::MAX).unwrap();
    assert!((ttr - 100.0).abs() < 0.01, "expected 100%, got {ttr:.2}%");
  }

  #[test]
  fn ttr_all_out_of_range_returns_0() {
    let records = vec![
      ("2024-01-01".to_string(), 4.0),
      ("2024-01-08".to_string(), 4.0),
    ];
    let ttr = calculate_ttr(&records, 2.0, 3.0, u32::MAX).unwrap();
    assert!((ttr - 0.0).abs() < 0.01, "expected 0%, got {ttr:.2}%");
  }

  #[test]
  fn ttr_half_in_range_returns_approximately_50() {
    // Days 1–7: INR interpolates 2.0→4.0, passes through 3.0 at midpoint.
    // Days in range: those where interpolated INR <= 3.0.
    let records = vec![
      ("2024-01-01".to_string(), 2.0),
      ("2024-01-15".to_string(), 4.0),
    ];
    let ttr = calculate_ttr(&records, 2.0, 3.0, u32::MAX).unwrap();
    // Should be approximately 50% (first half of the period is in range)
    assert!(ttr > 40.0 && ttr < 60.0, "expected ~50%, got {ttr:.2}%");
  }

  #[test]
  fn ttr_fewer_than_2_records_returns_none() {
    let records = vec![("2024-01-01".to_string(), 2.5)];
    assert!(calculate_ttr(&records, 2.0, 3.0, u32::MAX).is_none());
  }

  #[test]
  fn ttr_empty_returns_none() {
    assert!(calculate_ttr(&[], 2.0, 3.0, u32::MAX).is_none());
  }
}
