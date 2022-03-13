// This whole file is assumed to be #[cfg(not(target_arch = "wasm32"))]

// Necessary for keyboard handling in CLI
use std::sync::mpsc::TryRecvError;
use std::{thread};

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

  while !state.reset {
    // CLI only: Read input. When in step mode, keep reading while waiting is `!`.
    let mut waiting = '!';
    while waiting == '!' {
      // Handle keyboard event
      match state.stdin_channel.try_recv() {
        Ok(key) => {
          // `x` means x was pressed, ` ` means space or just a return was pressed, `!` means
          // some other input was pressed. Used for stepping logic.
          waiting = parse_input(key, options, state, hmap);
        },
        Err(TryRecvError::Empty) => (),
        Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
      }
      if !options.return_to_move { break; };

      if waiting == '!' { thread::sleep(state.delay); }
    }
    if state.load_best_as_miner_zero || state.reset {
      break;
    }

    go_iteration(options, state, &mut biomes, hmap);

    let mut end = true;
    for biome in &biomes {
      if biome.miner.movable.now_energy > 0.0 {
        // Switch to this biome, since it's still alive.
        options.visible_index = biome.index;
        end = false;
        break;
      }
    }
    if end {
      break;
    }
  }

  return post_ga_loop(options, state, biomes, curr_root_helix, hmap);
}

pub fn platform_log(s: &str) {
  println!("{}", s);
}

pub fn platform_print_world(table_str: &str) {
  print!("{}", table_str);
  print!("\x1b[56A\n");
}

pub fn platform_date_now() -> u64 {
  let now = SystemTime::now();
  let dur = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
  return dur as u64;
}
