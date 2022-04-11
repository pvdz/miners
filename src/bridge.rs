// This file bridges the gap between wasm and rs builds

use super::options::*;
use super::miner::*;

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

pub fn focus_weak(options: &mut Options, biome_index: usize, phase: Phase, desc: &str) {
  if !options.visual {
    log(format!("Setting visual to biome {} at {:?} because {}", biome_index, phase, desc).as_str());
    options.visual = true;
    options.return_to_move = true;
    options.visible_index = biome_index;
  }
}

pub fn focus_force(options: &mut Options, biome_index: usize, phase: Phase, desc: &str) {
  if !options.visual || options.visible_index != biome_index {
    log(format!("Force setting visual to biome {} at {:?} because {}", biome_index, phase, desc).as_str());
  }
  options.visual = true;
  options.visible_index = biome_index;
}
