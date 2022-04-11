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
  // Seeded with input seed
  pub instance_rng_seeded: Lcg128Xsl64,
  // Seeded from random input, so different for each app start
  pub instance_rng_unseeded: Lcg128Xsl64,

  pub best_min_x: i32,
  pub best_min_y: i32,
  pub best_max_x: i32,
  pub best_max_y: i32,

  // viewport positioning
  pub viewport_offset_x: i32,
  pub viewport_offset_y: i32,
  pub viewport_size_w: usize,
  pub viewport_size_h: usize,

  // One time action, done this way cause otherwise we need to juggle the miner position everywhere
  pub center_on_miner_next: bool,

  // Always make sure the miner is in viewport?
  pub auto_follow_miner: bool,
  pub auto_follow_buffer_min: i32, // Once the miner moves closer than this many tiles to the border
  pub auto_follow_buffer_max: i32, // Change the viewport to make it this many tiles instead

  #[cfg(not(target_arch = "wasm32"))]
  pub stdin_channel: Receiver<String>,

  // Delay is thread::sleep driven which won't work in main-web-thread so it's cli only
  pub delay: Duration,

  // see options.cost_increase_rate and options.cost_increase_interval
  pub cost_increase_value: f32,

  pub total_miner_count: u32,
  pub current_miner_count: u32,
  pub miner_count_since_last_best: u32,

  pub start_time: u64,
  pub pause_after_ticks: u64,

  pub stats_last_second: u64,

  pub stats_last_biome_ticks: i32,
  pub stats_last_ticks_sec: i32,

  pub stats_total_batches: i32,
  pub stats_total_batch_loops: i32,
  pub stats_total_biome_ticks: i32,

  pub batch_ticks: i32,
  pub last_match_loops: i32,
  pub has_energy: bool,
  pub non_visual_print: u64,

  // user input controls

  // When this gets set (by user interaction) the best miner is cleared and a new miner-seed is randomly picked.
  pub reset: bool,
  pub load_best_as_miner_zero: bool,
}

pub fn create_app_state(options: &Options, best_miner: (Helix, u64, usize, usize, Inventory), trail_lens: u64, instance_rng_seeded: Lcg128Xsl64, instance_rng_unseeded: Lcg128Xsl64) -> AppState {
  return AppState {
    startup: true,

    best_miner,
    trail_lens,
    instance_rng_seeded,
    instance_rng_unseeded,

    best_min_x: 0,
    best_min_y: 0,
    best_max_x: 0,
    best_max_y: 0,

    viewport_offset_x: -25,
    viewport_offset_y: -25,
    viewport_size_w: 51,
    viewport_size_h: 51,

    center_on_miner_next: false,

    auto_follow_miner: true,
    auto_follow_buffer_min: 2,
    auto_follow_buffer_max: 10,

    #[cfg(not(target_arch = "wasm32"))]
    stdin_channel: async_stdin::spawn_stdin_channel(),

    delay: Duration::from_millis(options.speed),

    cost_increase_value: 0.0,

    total_miner_count: 0,
    current_miner_count: 0,
    miner_count_since_last_best: 0,
    start_time: bridge::date_now(),
    pause_after_ticks: 0,

    stats_last_second: 0,
    stats_last_biome_ticks: 0,
    stats_last_ticks_sec: 0,

    stats_total_batches: 0,
    stats_total_batch_loops: 0,
    stats_total_biome_ticks: 0,

    batch_ticks: 0,
    last_match_loops: 0,
    has_energy: true,
    non_visual_print: 0,

    // user input controls
    reset: false,
    load_best_as_miner_zero: false,
  };
}
