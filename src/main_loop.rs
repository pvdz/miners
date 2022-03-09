use super::world::*;
use super::biome::*;
use super::helix::*;
use super::options::*;
use super::app_state::*;
use super::inventory::*;
use super::{bridge};

use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use std::{thread};

// Required to be able to cal next_u64
use rand::prelude::*;

pub fn pre_ga_loop(options: &mut Options, state: &mut AppState, curr_root_helix: &mut Helix) -> Vec<Biome> {

  state.stats_total_batches += 1;

  let biomes: Vec<Biome> = generate_biomes(options, state, curr_root_helix);

  // bridge::log("loaded");

  // Move it move it
  state.batch_loops = 0; // How many iterations for the current GA step
  state.has_energy = true; // As long as any miner in the current cycle has energy left...

  return biomes;
}

pub fn post_ga_loop(options: &mut Options, state: &mut AppState, biomes: Vec<Biome>, curr_root_helix: &mut Helix, hmap: &mut HashMap<u64, (u64, usize, SerializedHelix)>) -> Helix {

  // if state.load_best_as_miner_zero {
  //   return *state.best_helix_from_file;
  // }

  let mut next_root_helix = *curr_root_helix;

  if !state.reset {
    let points = get_points(&biomes[options.visible_index].miner.meta.inventory);
    let inv = clone_inventory(&biomes[options.visible_index].miner.meta.inventory);
    let mut winner: (Helix, u64, &World, usize, usize, Inventory) = (
      biomes[options.visible_index].miner.helix,
      points as u64,
      &biomes[options.visible_index].world,
      0,
      0,
      inv,
    );

    // Find best biome
    for m in 1..biomes.len() { // 1 because zero is used as init above
      let biome: &Biome = &biomes[m];
      let points = get_points(&biome.miner.meta.inventory) as u64;
      let inv = clone_inventory(&biome.miner.meta.inventory);
      if points > winner.1 {
        winner = (
          biome.miner.helix,
          points,
          &biome.world,
          0,
          0,
          inv,
        )
      }
    }

    if options.visual {
      for m in 0..biomes.len() {
        let biome: &Biome = &biomes[m];
        let points = get_points(&biome.miner.meta.inventory) as u64;
        println!(
          "- Biome {: <2}: Points: {: <6} [{: >4}x{: <4} , {: >4}x{: <4}] :: {}{: <100}",
          m, points,
          biome.world.min_x, biome.world.min_y, biome.world.max_x, biome.world.max_y,
          biome.miner.helix,
          ' '
        );
      }
    }

    let mut he : String = "".to_string();
    helix_to_string(&mut he, &winner.0);

    println!(
      "Time: {} s, batches: {: <5} bath loops: {: <5} miners: {}, in current seed: {}. Winner/Best points: {: >5} / {: >5}. Winner @ [{}x{} , {}x{}] -> {}{: >50}",
      bridge::date_now() - state.start_time,
      state.stats_total_batches,
      state.batch_loops,
      state.total_miner_count,
      state.current_miner_count,

      winner.1,
      state.best_miner.1,
      state.best_min_x,
      state.best_min_y,
      state.best_max_x,
      state.best_max_y,

      he,
      ' '
    );

    if winner.1 > state.best_miner.1 {
      println!("\x1b[32;1mFound a new best!\x1b[0m: From {} to {}. Inventory: {}", state.best_miner.1, winner.1, ui_inventory(&winner.5, options));
      state.best_miner = (winner.0, winner.1, winner.3, winner.4, winner.5); // helix, points, steps, uniques, inventory
      next_root_helix = winner.0;
      state.best_min_x = winner.2.min_x;
      state.best_min_y = winner.2.min_y;
      state.best_max_x = winner.2.max_x;
      state.best_max_y = winner.2.max_y;
      state.miner_count_since_last_best = 0;
    }
    if !options.mutate_from_best {
      // Mutate from last winner regardless of whether it was a new best
      next_root_helix = winner.0;
    }

    println!(
      "Hash Map has {} nodes with average trail len of {}. Ticks/s: {}",
      hmap.len(),
      state.trail_lens / hmap.len() as u64,
      state.stats_last_ticks_sec
    );

    if if options.reset_after_noop { state.miner_count_since_last_best } else { state.current_miner_count } > options.reset_rate {
      if options.reset_after_noop {
        println!("Auto reset after no new best in {} iterations", state.miner_count_since_last_best);
      } else {
        println!("Auto reset after {} iterations, auto resets after {}", options.reset_rate, state.current_miner_count);
      }
      state.reset = true;
    }
  }

  if state.reset {
    let new_seed = state.instance_rng.next_u64();
    bridge::log(format!("New miner seed: {}", new_seed).as_str());
    next_root_helix = create_initial_helix(&mut state.instance_rng, new_seed);
    state.current_miner_count = 0;

    // Do we reset other counters?

    state.reset = false;
    bridge::log("Resetting helix...");
  }

  // println!("map: {}", serde_json::to_string_pretty(&hmap).unwrap());
  // panic!("halt");

  return next_root_helix;
}

pub fn go_iteration(options: &mut Options, state: &mut AppState, biomes: &mut Vec<Biome>, hmap: &mut HashMap<u64, (u64, usize, SerializedHelix)>) {
  // This is basically the main game loop

  // log("inside loop start");

  state.stats_total_batch_loops += 1;
  state.batch_loops += 1;

  // Tick the biomes
  state.has_energy = false;
  for m in 0..biomes.len() {
    let biome = &mut biomes[m];
    tick_biome(options, state, biome, hmap);
  }

  // Stop drawing the world when the main miner is out of energy. Speed things up visually.
  let dur_sec = bridge::date_now() - state.start_time;
  if options.visual && biomes[options.visible_index].miner.movable.now_energy > 0.0 {
    options.frames_now += 1;
    if options.frames_now > options.frame_skip {
      options.frames_now = 0;

      let table_str: String = serialize_world(
        &biomes[options.visible_index].world,
        &biomes,
        options,
        state,
        format!("Best miner: Points: {}  Steps: {} ({})   Map: {}x{} ~ {}x{}  {}", state.best_miner.1, state.best_miner.2, state.best_miner.3, state.best_min_x, state.best_min_y, state.best_max_x, state.best_max_y, state.best_miner.0),
        format!("Miner Dictionary contains {} entries. Average steps: {}. Total time: {} s, batches: {}, batch loops: {}, biome ticks: {}, ticks/s: {}", hmap.len(), state.trail_lens / hmap.len().max(1) as u64, dur_sec, state.stats_total_batches, state.stats_total_batch_loops, state.stats_total_biome_ticks, state.stats_last_ticks_sec),
      );
      bridge::print_world(&table_str);

      // TODO: if we're trying to match a certain fps then we have to deduct the frame time from this delay. Not that it really matters here.
      // TODO: delay is currently 1:1 bound with tick time. We should detach that ;) Maybe. Yes for sure. The sleep is an artificial delay.
      #[cfg(not(target_arch = "wasm32"))]
      thread::sleep(state.delay);
    }
  }

  if dur_sec > state.stats_last_second {
    state.stats_last_second = dur_sec;
    // stats_last_batches = state.stats_total_batches;
    // stats_last_batch_loops = stats_total_batch_loops;
    state.stats_last_ticks_sec = state.stats_total_biome_ticks - state.stats_last_biome_ticks;
    state.stats_last_biome_ticks = state.stats_total_biome_ticks;
  }

  // As a way to balance the block_bump_cost value; the higher that penalty is, the faster
  // your slots cool down. The markup should not be major but probably if block_bump_cost is
  // close to zero, the slots cooldowns should not get any boosts.
  for m in 0..biomes.len() {
    let biome: &mut Biome = &mut biomes[m];
    if biome.miner.movable.now_energy > 0.0 {
      for slot in biome.miner.slots.iter_mut() {
        if biome.miner.meta.prev_move_bumped {
          slot.cur_cooldown *= 1.0 + (biome.miner.helix.block_bump_cost / 50000.0);
        }
      }
    }
  }
}
