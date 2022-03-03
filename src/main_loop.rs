use super::slot_broken_gps::*;
use super::slot_purity_scanner::*;
use super::slottable::*;
use super::movable::*;
use super::miner::*;
use super::world::*;
use super::biome::*;
use super::helix::*;
use super::options::*;
use super::app_state::*;
use super::inventory::*;
use super::{bridge};
use super::drone_win::*;
use super::drone_san::*;
use super::drone_me::*;
use super::slot_energy_cell::*;
use super::slot_drone_launcher::*;
use super::slot_jacks_compass::*;

use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use std::{thread};

// Required to be able to cal next_u64
use rand::prelude::*;

pub fn pre_ga_loop(options: &mut Options, state: &mut AppState, curr_root_helix: &mut Helix) -> Vec<Biome> {

  state.stats_total_batches += 1;

  let biomes: Vec<Biome> = generate_biomes(options, state, curr_root_helix);

  bridge::log("loaded");

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
    let points = get_points(&biomes[0].miner.meta.inventory);
    let inv = clone_inventory(&biomes[0].miner.meta.inventory);
    let mut winner: (Helix, u64, &World, usize, usize, Inventory) = (
      biomes[0].miner.helix,
      points as u64,
      &biomes[0].world,
      biomes[0].miner.movable.history.len(),
      biomes[0].miner.movable.unique.len(),
      inv,
    );

    for m in 1..biomes.len() { // 1 because zero is used as init above
      let biome: &Biome = &biomes[m];
      let points = get_points(&biome.miner.meta.inventory) as u64;
      let inv = clone_inventory(&biome.miner.meta.inventory);
      if points > winner.1 {
        winner = (
          biome.miner.helix,
          points,
          &biome.world,
          biome.miner.movable.history.len(),
          biome.miner.movable.unique.len(),
          inv,
        )
      }
    }

    if options.visual {
      for m in 0..biomes.len() {
        let biome: &Biome = &biomes[m];
        let points = get_points(&biome.miner.meta.inventory) as u64;
        println!(
          "- Biome {: <2}: Points: {: <6} Steps: {: <5} Unique: {: <5} [{: >4}x{: <4} , {: >4}x{: <4}] :: {}{: <100}",
          m, points, biome.miner.movable.history.len(), biome.miner.movable.unique.len(),
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
      "Binary tree mode has {} nodes with average trail len of {}. Ticks/s: {}",
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
    if biome.miner.movable.now_energy > 0.0 {
      let miner_disabled = biome.miner.movable.disabled;
      biome.ticks += 1;
      let ticks = biome.ticks;
      state.stats_total_biome_ticks += 1;

      let first_miner = m == 0;

      let mminermovable: &mut Movable = &mut biome.miner.movable;
      let mslots: &mut MinerSlots = &mut biome.miner.slots;
      let mmeta: &mut MinerMeta = &mut biome.miner.meta;
      let mwindrone: &mut Windrone = &mut biome.miner.windrone;
      let msandrone: &mut Sandrone = &mut biome.miner.sandrone;
      let mdrones: &mut Vec<MeDrone> = &mut biome.miner.drones;
      let mworld: &mut World = &mut biome.world;

      tick_world(mworld, &options, msandrone);
      if !miner_disabled {
        tick_miner(mminermovable, mmeta, mslots, mwindrone, msandrone);
      }

      let post_castle = msandrone.post_castle > 0;
      if msandrone.air_lifting {
        // Do not "move" the miner. It is being moved by the sandrone.
      } else if msandrone.air_lifted {
        // The magic castle wall is enabled
        let magic_min_x = msandrone.expansion_min_x;
        let magic_min_y = msandrone.expansion_min_y;
        let magic_max_x = msandrone.expansion_max_x;
        let magic_max_y = msandrone.expansion_max_y;

        move_movable(ticks, mminermovable, mslots, mmeta, mworld, options, Some(msandrone), true, post_castle, magic_min_x, magic_min_y, magic_max_x, magic_max_y);
      } else {
        // No magic castle wall
        move_movable(ticks, mminermovable, mslots, mmeta, mworld, options, None, false, post_castle, 0, 0, 0, 0);
      }

      for i in 0..mslots.len() {
        let slot: &mut Slottable = &mut biome.miner.slots[i];
        match slot.kind {
          SlotKind::BrokenGps => tick_slot_broken_gps(slot, mminermovable, first_miner),
          SlotKind::DroneLauncher => tick_slot_drone_launcher(ticks, slot, mminermovable, mdrones, mmeta, mworld, options, first_miner),
          SlotKind::Drill => (), // noop
          SlotKind::Emptiness => (), // noop
          SlotKind::EnergyCell => tick_slot_energy_cell(slot, mminermovable, mmeta, mworld, options, first_miner),
          SlotKind::Hammer => (), // noop
          SlotKind::JacksCompass => tick_slot_jacks_compass(slot, mminermovable, first_miner, mworld, options),
          SlotKind::PurityScanner => tick_slot_purity_scanner(slot, mmeta, first_miner),
          SlotKind::Windrone => tick_windrone(slot, &mut biome.miner.windrone, mminermovable.x, mminermovable.y, mmeta.inventory.wind, mworld, options, m),
          SlotKind::Sandrone => tick_sandrone(&mut biome.miner.sandrone, mminermovable, mmeta, mworld, options, m),
        }
      }

      // Does this miner still have energy left?
      if biome.miner.movable.now_energy > 0.0 {
        state.has_energy = true;
      } else {
        // This miner stopped now

        // Note: this generates super verbose strings (every pair is an array, every array is spread over four lines). Something to optimize later.
        // let trail: String = serde_json::to_string_pretty(&biome.miner.movable.unique).unwrap();
        let mut trail: String = String::new();

        for (i, (x, y)) in biome.miner.movable.unique.iter().enumerate() {
          if i == 0 {
            let s = format!("{} {}", x, y);
            trail.push_str(s.as_str());
          } else {
            let s = format!(" {} {}", x, y);
            trail.push_str(s.as_str());
          }
        }

        let cur_points = get_points(&biome.miner.meta.inventory);
        let has_trail: bool = hmap.contains_key(&cur_points);
        if !has_trail {
          hmap.insert(cur_points, (cur_points, biome.miner.movable.unique.len(), helix_serialize(&biome.miner.helix)));
          bridge::log(format!("This miner was new! Score: {} points in {} steps. Map now contains {} trails.", cur_points, biome.miner.movable.history.len(), hmap.len()).as_str());
          state.trail_lens += biome.miner.movable.unique.len() as u64;
        }
      }
    }
  }

  // Stop drawing the world when the main miner is out of energy. Speed things up visually.
  let dur_sec = bridge::date_now() - state.start_time;
  if options.visual && biomes[0].miner.movable.now_energy > 0.0 {
    options.frames_now += 1;
    if options.frames_now > options.frame_skip {
      options.frames_now = 0;

      let table_str: String = serialize_world(
        &biomes[0].world,
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
