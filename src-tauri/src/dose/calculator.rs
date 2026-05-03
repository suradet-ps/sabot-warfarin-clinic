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
/// # Algorithm (per AHA/ACC guidelines adapted for Thai practice)
///
/// | INR range                    | Adjustment  | Urgency  | Recheck |
/// |------------------------------|-------------|----------|---------|
/// | > 5.0                        | Hold, −25%  | hold     | 1–3 d   |
/// | 4.0 – 5.0                    | Hold, −20%  | hold     | 3–7 d   |
/// | target_high + 0.5 – 4.0      | −15–20%     | urgent   | 7 d     |
/// | target_high – target_high+0.5| −10%        | caution  | 14 d    |
/// | In therapeutic range         | 0%          | normal   | 28–42 d |
/// | target_low−0.5 – target_low  | +10%        | caution  | 14 d    |
/// | < target_low − 0.5           | +15–20%     | urgent   | 7–14 d  |
pub fn suggest_dose(
  current_dose: f64,
  inr: f64,
  target_low: f64,
  target_high: f64,
) -> DoseSuggestion {
  let above_high = inr - target_high;
  let below_low = target_low - inr;

  let (adjustment_percent, recommendation, urgency, recheck_days): (f64, &str, &str, u32) =
    if inr > 5.0 {
      (
        -25.0,
        "หยุดยาทันทีและประเมินเร่งด่วน — INR > 5.0 เสี่ยงเลือดออกรุนแรง",
        "hold",
        2,
      )
    } else if inr > 4.0 {
      (
        -20.0,
        "งดยา 1 มื้อ แล้วลดขนาดยา 20% นัดตรวจ INR ใหม่ใน 3–7 วัน",
        "hold",
        5,
      )
    } else if above_high > 0.5 {
      (-15.0, "ลดขนาดยา 15% นัดตรวจ INR ใหม่ใน 7 วัน", "urgent", 7)
    } else if above_high > 0.0 {
      (
        -10.0,
        "ลดขนาดยาเล็กน้อย 10% นัดตรวจ INR ใหม่ใน 14 วัน",
        "caution",
        14,
      )
    } else if below_low > 0.5 {
      (15.0, "เพิ่มขนาดยา 15% นัดตรวจ INR ใหม่ใน 7–14 วัน", "urgent", 10)
    } else if below_low > 0.0 {
      (
        10.0,
        "เพิ่มขนาดยาเล็กน้อย 10% นัดตรวจ INR ใหม่ใน 14 วัน",
        "caution",
        14,
      )
    } else {
      (
        0.0,
        "INR อยู่ในเป้าหมาย — คงขนาดยาเดิม นัดตรวจ INR ใน 4–6 สัปดาห์",
        "normal",
        35,
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
  fn suggest_dose_slightly_above_high_decreases_10_percent() {
    let result = suggest_dose(5.0, 3.3, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, -10.0);
    assert_eq!(result.urgency, "caution");
    // 5.0 * 0.90 = 4.5
    assert_eq!(result.suggested_dose_mgday, 4.5);
  }

  #[test]
  fn suggest_dose_significantly_above_high_decreases_15_percent() {
    let result = suggest_dose(5.0, 3.7, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, -15.0);
    assert_eq!(result.urgency, "urgent");
    // 5.0 * 0.85 = 4.25 → rounded to 4.5
    assert_eq!(result.suggested_dose_mgday, 4.5);
  }

  #[test]
  fn suggest_dose_critical_high_4_to_5_hold_and_reduce_20() {
    let result = suggest_dose(5.0, 4.5, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, -20.0);
    assert_eq!(result.urgency, "hold");
    // 5.0 * 0.80 = 4.0
    assert_eq!(result.suggested_dose_mgday, 4.0);
  }

  #[test]
  fn suggest_dose_over_5_hold_and_reduce_25() {
    let result = suggest_dose(5.0, 5.5, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, -25.0);
    assert_eq!(result.urgency, "hold");
    // 5.0 * 0.75 = 3.75 → rounded to 4.0
    assert_eq!(result.suggested_dose_mgday, 4.0);
    assert_eq!(result.recheck_days, 2);
  }

  #[test]
  fn suggest_dose_slightly_below_low_increases_10_percent() {
    let result = suggest_dose(5.0, 1.8, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, 10.0);
    assert_eq!(result.urgency, "caution");
    // 5.0 * 1.10 = 5.5
    assert_eq!(result.suggested_dose_mgday, 5.5);
  }

  #[test]
  fn suggest_dose_significantly_below_low_increases_15_percent() {
    let result = suggest_dose(5.0, 1.3, 2.0, 3.0);
    assert_eq!(result.adjustment_percent, 15.0);
    assert_eq!(result.urgency, "urgent");
    // 5.0 * 1.15 = 5.75 → rounded to 6.0
    assert_eq!(result.suggested_dose_mgday, 6.0);
  }

  #[test]
  fn suggest_dose_rounds_to_half_mg() {
    // 3.0 * 1.10 = 3.3 → rounds to 3.5
    let result = suggest_dose(3.0, 1.8, 2.0, 3.0);
    assert_eq!(result.suggested_dose_mgday, 3.5);
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
