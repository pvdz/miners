use std::env;

use std::time::Duration;
use std::fs;
use std::collections::HashMap;

use super::{bridge};
use super::app_state::*;
use super::helix::*;

#[cfg(target_arch = "wasm32")]
use serde_derive::{Serialize, Deserialize};

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Options {
  pub batch_size: u8,
  pub initial_miner_code: String,
  pub mutation_rate_genes: f32,
  pub mutation_rate_slots: f32,
  // Mutate a new batch from the overall best or the last winner?
  pub mutate_from_best: bool,
  // Reset every this many generated miners
  pub reset_rate: u32,
  // Only reset after that many miners did not yield a new best?
  pub reset_after_noop: bool,
  // Press enter to forward a tick? Useful for debugging.
  pub return_to_move: bool,
  pub seed: u64,
  pub speed: u64,
  pub frame_skip: u32,
  pub frames_now: u32,
  pub visual: bool,
  // Sandrone will pick up miner after putting down this many push tiles
  pub sandrone_pickup_count: u32,
  // print the world in html rather than terminal ansi?
  pub sandcastle_area_limit: u32, // Sandrone will permanently stop building the wall after the castle area is at least this big
  // Show the miner in all other biomes in the map as well? Confusing but fun? :)
  pub html_mode: bool,
  pub show_biomes: bool,

  pub visible_index: usize, // Which biome are we painting?

  // Debugging
  pub paint_ten_lines: bool,
  // Draw grids at every 10th line/col
  pub paint_zero_zero: bool,
  // Draw hash for the 0,0 coord
  pub paint_miner_ids: bool,
  // Draw biome index for other biome miners rather than emoji
  pub paint_empty_world: bool,
  // Always draw empty tiles instead of the world
  pub hide_world_oob: bool,
  // Do not draw the world that doesn't explicitly exist in memory
  pub hide_world_ib: bool,
  // Do not draw the world that explicitly exists in memory (only oob)
  pub paint_visited: bool,
  // Paint the number of times the miner visited a tile, in the world view?
  pub paint_visited_bool: bool, // If the miner visited a tile, paint that tile so you can see? Not a count, just a yes/no.
  // Film noir?
  pub paint_colors: bool,
  // Can disable background colors while keeping foreground colors
  pub paint_bg_colors: bool,
  // Can disable foreground colors while keeping background colors
  pub paint_fg_colors: bool,
}

pub fn parse_cli_args() -> Options {
  // Defaults:
  let mut options = Options {
    batch_size: 10, // Can be controlled through --batch-size
    initial_miner_code: "".to_string(),
    mutation_rate_genes: 5.0,
    mutation_rate_slots: 5.0,
    mutate_from_best: false,
    seed: 210114, // 0 is random. Can be set through --seed
    speed: 1,
    frame_skip: 0,
    frames_now: 0,
    reset_rate: 500,
    reset_after_noop: true,
    return_to_move: false,
    visual: false, // Can be set through --visual and --no-visual

    sandrone_pickup_count: 200,
    sandcastle_area_limit: 500,

    html_mode: false,

    show_biomes: true,
    visible_index: 0,

    // Debug
    paint_ten_lines: false,
    paint_zero_zero: false,
    paint_empty_world: false,
    paint_miner_ids: false,
    hide_world_oob: false,
    hide_world_ib: false,
    paint_visited: false,
    paint_visited_bool: false,
    paint_colors: true,
    paint_bg_colors: true,
    paint_fg_colors: true,
  };

  let args: Vec<String> = env::args().collect();
  println!("{:?}", args);

  let mut index = 1; // The first one is the binary path so let's skip that :)
  while index < args.len() {
    match args[index].as_str() {
      "--seed" => {
        index += 1;
        options.seed = args[index].trim().parse::<u64>().unwrap_or(0);
        if options.seed == 0 {
          panic!("Seed must be a non-zero positive integer");
        }
      }
      "--visual" => {
        options.visual = true;
      }
      "--no-visual" => {
        options.visual = false;
      }
      "--batch-size" => {
        index += 1;
        options.batch_size = args[index].trim().parse::<u8>().unwrap_or(0);
        if options.batch_size == 0 {
          panic!("Seed must be a non-zero positive integer");
        }
      }
      "--miner" => {
        index += 1;
        options.initial_miner_code = args[index].trim().parse::<String>().unwrap_or("".to_string());
      }
      _ => {
        println!("Unknown parameter: {}", args[index]);
        panic!();
      }
    }

    index = index + 1;
  }

  options
}

pub fn parse_input(key: String, options: &mut Options, state: &mut AppState, hmap: &mut HashMap<u64, (u64, usize, SerializedHelix)>) -> bool {
  let mut waiting = true; // This is for x mode in the CLI

  match key.as_str() {
    "\n" => waiting = false, // Tick forward
    "x\n" => {
      options.return_to_move = !options.return_to_move;
      waiting = false;
    },
    "v\n" => options.visual = !options.visual,
    "+\n" => {
      options.speed = (options.speed as f64 + (options.speed as f64 * 0.1).max(1.0)).max(1.0) as u64;
      state.delay = Duration::from_millis(options.speed);
    },
    "++\n" => {
      options.speed = (options.speed as f64 + (options.speed as f64 * 0.5).max(1.0)).max(1.0) as u64;
      state.delay = Duration::from_millis(options.speed);
    },
    "-\n" => {
      options.speed = (options.speed as f64 - (options.speed as f64 * 0.1).max(1.0)).max(1.0) as u64;
      state.delay = Duration::from_millis(options.speed);
    },
    "--\n" => {
      options.speed = (options.speed as f64 - (options.speed as f64 * 0.5).max(1.0)).max(1.0) as u64;
      state.delay = Duration::from_millis(options.speed);
    },
    "o\n" => options.mutation_rate_genes = (options.mutation_rate_genes - 1.0).max(0.0),
    "oo\n" => options.mutation_rate_genes = (options.mutation_rate_genes - 5.0).max(0.0),
    "p\n" => options.mutation_rate_genes = (options.mutation_rate_genes + 1.0).max(0.0),
    "pp\n" => options.mutation_rate_genes = (options.mutation_rate_genes + 5.0).max(0.0),
    "k\n" => options.mutation_rate_slots = (options.mutation_rate_slots - 1.0).max(0.0),
    "kk\n" => options.mutation_rate_slots = (options.mutation_rate_slots - 5.0).max(0.0),
    "l\n" => options.mutation_rate_slots = (options.mutation_rate_slots + 1.0).max(0.0),
    "ll\n" => options.mutation_rate_slots = (options.mutation_rate_slots + 5.0).max(0.0),
    "n\n" => options.batch_size = (options.batch_size - 1).max(1),
    "m\n" => options.batch_size = options.batch_size + 1,
    "r\n" => {
      state.reset = true;
      bridge::log("Manual reset requested...");
    },
    "b\n" => {
      state.load_best_as_miner_zero = true;
      bridge::log("Aborting current run and loading best as miner now...");
    }
    "g\n" => {
      options.mutate_from_best = !options.mutate_from_best;
      bridge::log(format!("Swapping options.mutate_from_best; now: {}", options.mutate_from_best).as_str());
    }
    "t\n" => {
      options.reset_after_noop = !options.reset_after_noop;
      bridge::log(format!("Swapping options.reset_after_noop; now: {}", options.reset_after_noop).as_str());
    }
    "q\n" => {
      // Save and quit.
      println!("Serializing hash map with {} entries...", hmap.len());
      let s = serde_json::to_string_pretty(&hmap).unwrap();
      let f = format!("./seed_{}.rson", options.seed);
      println!("Storing {} bytes to `{}`", s.len(), f);
      fs::write(f, s).expect("Unable to write file");
      println!("Finished writing. Exiting now...");
      panic!("Quit after request");
    },
    "\x1b\x5b\x41\n" => {
      // [27, 91, 67, 10]
      // Up
      state.viewport_offset_y -= 1;
    },
    "\x1b\x5b\x43\n" => {
      // [27, 91, 67, 10]
      // Right
      state.viewport_offset_x += 1;
    },
    "\x1b\x5b\x42\n" => {
      // [27, 91, 67, 10]
      // Down
      state.viewport_offset_y += 1;
    },
    "\x1b\x5b\x44\n" => {
      // [27, 91, 67, 10]
      // Left
      state.viewport_offset_x -= 1;
    },
    "h\n" => {
      bridge::log("Centering viewport to 0x0");
      state.viewport_offset_x = -(state.viewport_size_w as i32)/2;
      state.viewport_offset_y = -(state.viewport_size_h as i32)/2;
    },
    "c\n" => {
      bridge::log("Centering viewport to miner position");
      state.center_on_miner_next = true;
    },
    "f\n" => {
      state.auto_follow_miner = !state.auto_follow_miner;
      bridge::log(format!("Toggled auto-follow mode: {}", if state.auto_follow_miner { "on" } else { "off" }).as_str());
    },
    e => if e != "" { bridge::log(format!("Input {:?} had no effect", e.as_bytes()).as_str()) },
  }

  return waiting;
}
