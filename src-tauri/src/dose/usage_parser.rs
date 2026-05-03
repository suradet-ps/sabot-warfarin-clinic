use regex::Regex;
use std::collections::BTreeSet;

use crate::models::{
  dispensing::ParsedDoseInfo,
  visit::DoseSchedule,
};

#[derive(Debug, Clone)]
pub struct ParsedUsageResult {
  pub dose: Option<ParsedDoseInfo>,
  pub note: Option<String>,
}

const DAY_KEYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

pub fn parse_dispensing_usage(strength: &str, usage_text: &str) -> ParsedUsageResult {
  let strength_mg = match extract_strength_mg(strength) {
    Some(value) if value > 0.0 => value,
    _ => {
      return ParsedUsageResult {
        dose: None,
        note: Some("ไม่พบความแรงยาที่ใช้คำนวณ".to_string()),
      };
    }
  };

  let normalized = normalize_usage_text(usage_text);
  let tablets_per_dose = match extract_tablet_amount(&normalized) {
    Some(value) if value > 0.0 => value,
    _ => {
      return ParsedUsageResult {
        dose: None,
        note: Some("ไม่สามารถตีความจำนวนเม็ดยาจากวิธีใช้ยาได้".to_string()),
      };
    }
  };

  let day_result = extract_day_indexes(&normalized);
  let mg_per_dose = round_to_two_decimals(tablets_per_dose * strength_mg);
  let mut schedule = empty_schedule();

  for day_index in &day_result.day_indexes {
    set_schedule_value(&mut schedule, *day_index, mg_per_dose);
  }

  let mg_per_week = round_to_two_decimals(day_result.day_indexes.len() as f64 * mg_per_dose);
  let mg_per_day_average = round_to_two_decimals(mg_per_week / 7.0);
  let matched_days = day_result
    .day_indexes
    .iter()
    .map(|day_index| day_key_to_string(*day_index))
    .collect();

  ParsedUsageResult {
    dose: Some(ParsedDoseInfo {
      tablets_per_dose,
      mg_per_dose,
      mg_per_week,
      mg_per_day_average,
      schedule,
      matched_days,
    }),
    note: day_result.note,
  }
}

fn extract_strength_mg(strength: &str) -> Option<f64> {
  let re = Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*mg").expect("valid regex");
  if let Some(captures) = re.captures(strength) {
    return captures.get(1).and_then(|value| value.as_str().parse::<f64>().ok());
  }

  let any_number = Regex::new(r"(\d+(?:\.\d+)?)").expect("valid regex");
  any_number
    .captures(strength)
    .and_then(|captures| captures.get(1))
    .and_then(|value| value.as_str().parse::<f64>().ok())
}

fn extract_tablet_amount(normalized: &str) -> Option<f64> {
  let fraction_re = Regex::new(r"(\d+)\s*/\s*(\d+)").expect("valid regex");
  if let Some(captures) = fraction_re.captures(normalized) {
    let numerator = captures.get(1)?.as_str().parse::<f64>().ok()?;
    let denominator = captures.get(2)?.as_str().parse::<f64>().ok()?;
    if denominator > 0.0 {
      return Some(round_to_two_decimals(numerator / denominator));
    }
  }

  let tablet_re = Regex::new(
    r"(?i)(\d+(?:\.\d+)?)\s*(?:tab(?:let)?s?|เม็ด)",
  )
  .expect("valid regex");
  if let Some(captures) = tablet_re.captures(normalized) {
    return captures.get(1).and_then(|value| value.as_str().parse::<f64>().ok());
  }

  let number_re = Regex::new(r"(?i)(\d+(?:\.\d+)?)").expect("valid regex");
  number_re
    .captures(normalized)
    .and_then(|captures| captures.get(1))
    .and_then(|value| value.as_str().parse::<f64>().ok())
}

struct DayExtractionResult {
  day_indexes: Vec<usize>,
  note: Option<String>,
}

fn extract_day_indexes(normalized: &str) -> DayExtractionResult {
  let tokens = tokenize_usage(normalized);
  if tokens.iter().any(|token| token == "every_day") {
    return DayExtractionResult {
      day_indexes: (0..DAY_KEYS.len()).collect(),
      note: None,
    };
  }

  let mut matched_days = BTreeSet::new();
  let mut index = 0;
  while index < tokens.len() {
    let current_day = match parse_day_token(&tokens[index]) {
      Some(day) => day,
      None => {
        index += 1;
        continue;
      }
    };

    if index + 2 < tokens.len() && is_range_connector(&tokens[index + 1]) {
      if let Some(end_day) = parse_day_token(&tokens[index + 2]) {
        for day in expand_day_range(current_day, end_day) {
          matched_days.insert(day);
        }
        index += 3;
        continue;
      }
    }

    matched_days.insert(current_day);
    index += 1;
  }

  if matched_days.is_empty() {
    DayExtractionResult {
      day_indexes: (0..DAY_KEYS.len()).collect(),
      note: Some("ไม่พบวันในวิธีใช้ยา จึงตีความว่าให้รับประทานทุกวัน".to_string()),
    }
  } else {
    DayExtractionResult {
      day_indexes: matched_days.into_iter().collect(),
      note: None,
    }
  }
}

fn normalize_usage_text(input: &str) -> String {
  let mut text = input.to_lowercase();

  for (thai_digit, arabic_digit) in [
    ('๐', '0'),
    ('๑', '1'),
    ('๒', '2'),
    ('๓', '3'),
    ('๔', '4'),
    ('๕', '5'),
    ('๖', '6'),
    ('๗', '7'),
    ('๘', '8'),
    ('๙', '9'),
  ] {
    text = text.replace(thai_digit, &arabic_digit.to_string());
  }

  for (from, to) in [
    ("ครึ่ง", "0.5"),
    ("half", "0.5"),
    ("หนึ่ง", "1"),
    ("สอง", "2"),
    ("สาม", "3"),
    ("สี่", "4"),
    ("ห้า", "5"),
    ("ทุกวัน", " every_day "),
    ("ทุกวันหลังอาหาร", " every_day "),
    ("ทุกคืน", " every_day "),
    ("ไม่ระบุวัน", " every_day "),
    ("daily", " every_day "),
    ("every day", " every_day "),
    ("qd", " every_day "),
    ("od", " every_day "),
    ("จันทร์", " mon "),
    ("อังคาร", " tue "),
    ("พุธ", " wed "),
    ("พฤหัสบดี", " thu "),
    ("พฤหัส", " thu "),
    ("ศุกร์", " fri "),
    ("เสาร์", " sat "),
    ("อาทิตย์", " sun "),
    ("monday", " mon "),
    ("tuesday", " tue "),
    ("wednesday", " wed "),
    ("thursday", " thu "),
    ("friday", " fri "),
    ("saturday", " sat "),
    ("sunday", " sun "),
    ("mon", " mon "),
    ("tue", " tue "),
    ("wed", " wed "),
    ("thu", " thu "),
    ("fri", " fri "),
    ("sat", " sat "),
    ("sun", " sun "),
  ] {
    text = text.replace(from, to);
  }

  text = text
    .replace(',', " ")
    .replace(';', " ")
    .replace('(', " ")
    .replace(')', " ")
    .replace('\n', " ")
    .replace('\t', " ")
    .replace('-', " - ")
    .replace('–', " - ")
    .replace('—', " - ")
    .replace("ถึง", " ถึง ")
    .replace("to", " to ")
    .replace("thru", " thru ")
    .replace("through", " through ");

  text
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

fn tokenize_usage(normalized: &str) -> Vec<String> {
  normalized
    .split_whitespace()
    .map(|token| match token {
      "จ" => "mon".to_string(),
      "อ" => "tue".to_string(),
      "พ" => "wed".to_string(),
      "พฤ" => "thu".to_string(),
      "ศ" => "fri".to_string(),
      "ส" => "sat".to_string(),
      "อา" => "sun".to_string(),
      other => other.to_string(),
    })
    .collect()
}

fn parse_day_token(token: &str) -> Option<usize> {
  match token {
    "mon" => Some(0),
    "tue" => Some(1),
    "wed" => Some(2),
    "thu" => Some(3),
    "fri" => Some(4),
    "sat" => Some(5),
    "sun" => Some(6),
    _ => None,
  }
}

fn is_range_connector(token: &str) -> bool {
  matches!(token, "-" | "ถึง" | "to" | "thru" | "through")
}

fn expand_day_range(start: usize, end: usize) -> Vec<usize> {
  if start <= end {
    (start..=end).collect()
  } else {
    let mut days: Vec<usize> = (start..DAY_KEYS.len()).collect();
    days.extend(0..=end);
    days
  }
}

fn empty_schedule() -> DoseSchedule {
  DoseSchedule {
    mon: 0.0,
    tue: 0.0,
    wed: 0.0,
    thu: 0.0,
    fri: 0.0,
    sat: 0.0,
    sun: 0.0,
  }
}

fn set_schedule_value(schedule: &mut DoseSchedule, day_index: usize, value: f64) {
  match day_index {
    0 => schedule.mon = value,
    1 => schedule.tue = value,
    2 => schedule.wed = value,
    3 => schedule.thu = value,
    4 => schedule.fri = value,
    5 => schedule.sat = value,
    6 => schedule.sun = value,
    _ => {}
  }
}

fn day_key_to_string(day_index: usize) -> String {
  DAY_KEYS.get(day_index).unwrap_or(&"mon").to_string()
}

fn round_to_two_decimals(value: f64) -> f64 {
  (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
  use super::parse_dispensing_usage;

  #[test]
  fn parses_weekday_schedule_in_thai() {
    let result = parse_dispensing_usage("5 mg", "กิน 1 เม็ด ก่อนนอน วันจันทร์ถึงศุกร์");
    let dose = result.dose.expect("expected parsed dose");
    assert_eq!(dose.mg_per_week, 25.0);
    assert_eq!(dose.schedule.mon, 5.0);
    assert_eq!(dose.schedule.fri, 5.0);
    assert_eq!(dose.schedule.sat, 0.0);
  }

  #[test]
  fn parses_english_range_with_half_tablet() {
    let result = parse_dispensing_usage("3 mg", "กิน 0.5 tab mon-sat");
    let dose = result.dose.expect("expected parsed dose");
    assert_eq!(dose.mg_per_week, 9.0);
    assert_eq!(dose.schedule.mon, 1.5);
    assert_eq!(dose.schedule.sat, 1.5);
    assert_eq!(dose.schedule.sun, 0.0);
  }

  #[test]
  fn defaults_to_every_day_when_days_missing() {
    let result = parse_dispensing_usage("2 mg", "กิน ครึ่ง เม็ด ก่อนนอน");
    let dose = result.dose.expect("expected parsed dose");
    assert_eq!(dose.mg_per_week, 7.0);
    assert_eq!(dose.schedule.sun, 1.0);
    assert!(result.note.is_some());
  }

  #[test]
  fn parses_explicit_every_day_text() {
    let result = parse_dispensing_usage("5 mg", "กิน 1 เม็ด ก่อนนอน ไม่ระบุวัน");
    let dose = result.dose.expect("expected parsed dose");
    assert_eq!(dose.mg_per_week, 35.0);
    assert!(result.note.is_none());
  }

  #[test]
  fn returns_note_when_amount_is_missing() {
    let result = parse_dispensing_usage("5 mg", "ก่อนนอน วันจันทร์ถึงศุกร์");
    assert!(result.dose.is_none());
    assert!(result.note.is_some());
  }
}