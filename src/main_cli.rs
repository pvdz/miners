// This whole file is assumed to be #[cfg(not(target_arch = "wasm32"))]

// Necessary for keyboard handling in CLI
use std::sync::mpsc::TryRecvError;

extern crate serde_json;
use std::collections::HashMap;

use std::time::SystemTime;

use super::main_loop::*;
use super::options::*;
use super::helix::*;
use super::biome::*;
use super::app_state::*;
use super::initialize::*;
use super::bridge::*;

pub fn main() {
  log("Running sync main_cli.rs.... :)");

  let mut options = parse_cli_args();
  let (mut state, mut next_root_helix, mut hmap) = initialize(&mut options);
  ga_loop_sync(&mut options, &mut state, &mut next_root_helix, &mut hmap);
}

pub fn ga_loop_sync(options: &mut Options, state: &mut AppState, next_root_helix: &mut Helix, hmap: &mut HashMap<u64, (u64, usize, SerializedHelix)>) {
  loop {
    state.startup = false;
    *next_root_helix = ga_step_sync(options, state, next_root_helix, hmap);
  }
}

pub fn ga_step_sync(options: &mut Options, state: &mut AppState, curr_root_helix: &mut Helix, hmap: &mut HashMap<u64, (u64, usize, SerializedHelix)>) -> Helix {
  let mut biomes: Vec<Biome> = pre_ga_loop(options, state, curr_root_helix);

  while state.has_energy && !state.reset {

    // CLI only: Read input
    let mut waiting = true;
    while waiting {
      // Handle keyboard event
      match state.stdin_channel.try_recv() {
        Ok(key) => {
          // If `x` was pressed in the CLI, stop waiting for one frame. Ignored in the web. (?)
          waiting = parse_input(key, options, state, hmap);
        },
        Err(TryRecvError::Empty) => (),
        Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
      }
      if !options.return_to_move { break; };
    }
    if state.load_best_as_miner_zero || state.reset {
      break;
    }

    go_iteration(options, state, &mut biomes, hmap);
  }

  return post_ga_loop(options, state, biomes, curr_root_helix, hmap);
}

pub fn platform_log(s: &str) {
  println!("{}", s);
}

pub fn platform_print_world(table_str: &str) {
  print!("\x1b[56A\n");
  print!("{}", table_str);
}

pub fn platform_date_now() -> u64 {
  let now = SystemTime::now();
  let dur = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
  return dur as u64;
}
