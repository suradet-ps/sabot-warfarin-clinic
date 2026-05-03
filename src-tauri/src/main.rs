#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  if let Err(error) = sabot_warfarin_clinic_lib::run() {
    eprintln!("failed to run tauri application: {error}");
  }
}
