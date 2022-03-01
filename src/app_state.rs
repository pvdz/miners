use super::{bridge};
use super::helix::*;
use super::inventory::*;
use super::options::*;

use rand_pcg::{Lcg128Xsl64};

#[cfg(not(target_arch = "wasm32"))]
use super::async_stdin;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::Receiver;
use std::time::Duration;

pub struct AppState {
  pub startup: bool,

  pub best_miner: (Helix, u64, usize, usize, Inventory),
  pub trail_lens: u64,
  pub instance_rng: Lcg128Xsl64,

  pub best_min_x: i32,
  pub best_min_y: i32,
  pub best_max_x: i32,
  pub best_max_y: i32,

  #[cfg(not(target_arch = "wasm32"))]
  pub stdin_channel: Receiver<String>,

  // Delay is thread::sleep driven which won't work in main-web-thread so it's cli only
  pub delay: Duration,

  pub total_miner_count: u32,
  pub current_miner_count: u32,
  pub miner_count_since_last_best: u32,

  pub start_time: u64,

  pub stats_last_second: u64,

  pub stats_last_biome_ticks: i32,
  pub stats_last_ticks_sec: i32,

  pub stats_total_batches: i32,
  pub stats_total_batch_loops: i32,
  pub stats_total_biome_ticks: i32,

  pub batch_loops: i32,
  pub has_energy: bool,

  // user input controls

  // When this gets set (by user interaction) the best miner is cleared and a new miner-seed is randomly picked.
  pub reset: bool,
  pub load_best_as_miner_zero: bool,
}

pub fn create_app_state(options: &Options, best_miner: (Helix, u64, usize, usize, Inventory), trail_lens: u64, instance_rng: Lcg128Xsl64) -> AppState {
  return AppState {
    startup: true,

    best_miner,
    trail_lens,
    instance_rng,

    best_min_x: 0,
    best_min_y: 0,
    best_max_x: 0,
    best_max_y: 0,

    #[cfg(not(target_arch = "wasm32"))]
    stdin_channel: async_stdin::spawn_stdin_channel(),

    delay: Duration::from_millis(options.speed),

    total_miner_count: 0,
    current_miner_count: 0,
    miner_count_since_last_best: 0,
    start_time: bridge::date_now(),

    stats_last_second: 0,
    stats_last_biome_ticks: 0,
    stats_last_ticks_sec: 0,

    stats_total_batches: 0,
    stats_total_batch_loops: 0,
    stats_total_biome_ticks: 0,

    batch_loops: 0,
    has_energy: true,

    // user input controls
    reset: false,
    load_best_as_miner_zero: false,
  };
}
