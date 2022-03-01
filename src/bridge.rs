// This file bridges the gap between wasm and rs builds

#[cfg(target_arch = "wasm32")]
use super::main_web::{platform_log, platform_date_now, platform_print_world};
#[cfg(not(target_arch = "wasm32"))]
use super::main_cli::{platform_log, platform_date_now, platform_print_world};

pub fn log(s: &str) {
  platform_log(format!("{}", s).as_str());
}

pub fn date_now() -> u64 {
  return platform_date_now();
}

pub fn print_world(s: &str) {
  return platform_print_world(s);
}
