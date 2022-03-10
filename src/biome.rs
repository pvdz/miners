use super::slot_broken_gps::*;
use super::slot_purity_scanner::*;
use super::slottable::*;
use super::miner::*;
use super::world::*;
use super::helix::*;
use super::options::*;
use super::app_state::*;
use super::inventory::*;
use super::{bridge};
use super::drone_win::*;
use super::drone_san::*;
use super::slot_energy_cell::*;
use super::slot_drone_launcher::*;
use super::slot_jacks_compass::*;

use std::collections::HashMap;

pub struct Biome {
  pub index: usize, // Which biome is this in the current set of biomes?
  pub ticks: u32,
  pub world: World,
  pub miner: Miner,

  // The real path this miner has taken in this world
  pub path: Vec<i32>,
}

pub fn generate_biomes(options: &mut Options, state: &mut AppState, curr_root_helix: &mut Helix) -> Vec<Biome> {
  // Generate a bunch of biomes. Create a world for them and put a miner in there.
  // Each biome shares the same world (governed by the seed). But since the world is destructible
  // we have to give each biome their own world state.
  let mut biomes: Vec<Biome> = vec!();
  for i in 0..options.batch_size {
    let cur_miner: Miner =
      if state.load_best_as_miner_zero {
        state.load_best_as_miner_zero = false;
        println!("loading best miner into biome {}... {}", i, curr_root_helix);
        create_miner_from_helix(curr_root_helix)
      } else {
        create_miner_from_helix(&mutate_helix(&mut state.instance_rng, curr_root_helix, &options)) // The helix will clone/copy. Can/should we prevent this?
      };
    let own_world: World = generate_world(&options);
    let biome = Biome {
      index: i as usize,
      ticks: 0,
      world: own_world,
      miner: cur_miner,
      path: vec!(0, 0),
    };
    // println!("====== miner ======");
    // println!("miner slots: {:?}", &biome.miner.slots);
    // println!("===================");
    biomes.push(biome);
  }


  state.total_miner_count += biomes.len() as u32;
  state.current_miner_count += biomes.len() as u32;
  state.miner_count_since_last_best += biomes.len() as u32;

  return biomes;
}

pub fn tick_biome(options: &mut Options, state: &mut AppState, biome: &mut Biome, hmap: &mut HashMap<u64, (u64, usize, SerializedHelix)>) {
  if biome.miner.movable.now_energy > 0.0 {
    let miner_disabled = biome.miner.movable.disabled;
    biome.ticks += 1;
    let ticks = biome.ticks;
    state.stats_total_biome_ticks += 1;

    tick_world(options, state, biome);
    if !miner_disabled {
      // tick_miner(mminermovable, mmeta, mslots, mwindrone, msandrone);
      tick_miner(options, biome);
    }

    for i in 0..biome.miner.slots.len() {
      let slot: &mut Slottable = &mut biome.miner.slots[i];
      match slot.kind {
        SlotKind::BrokenGps => tick_slot_broken_gps(options, biome, i),
        SlotKind::DroneLauncher => tick_slot_drone_launcher(options, biome, i),
        SlotKind::Drill => (), // noop
        SlotKind::Emptiness => (), // noop
        SlotKind::EnergyCell => tick_slot_energy_cell(options, biome, i),
        SlotKind::Hammer => (), // noop
        SlotKind::JacksCompass => tick_slot_jacks_compass(options, biome, i),
        SlotKind::PurityScanner => tick_slot_purity_scanner(options, biome, i),
        SlotKind::Windrone => tick_windrone(options, biome, i),
        SlotKind::Sandrone => tick_sandrone(options, biome, i),
      }
    }

    // Does this miner still have energy left?
    if biome.miner.movable.now_energy <= 0.0 {
      // This miner stopped now

      let cur_points = get_points(&biome.miner.meta.inventory);
      let has_trail: bool = hmap.contains_key(&cur_points);
      if !has_trail {
        hmap.insert(cur_points, (cur_points, 0, helix_serialize(&biome.miner.helix)));
        bridge::log(format!("Miner {} was new! Score: {} points. Map now contains {} trails.", biome.index, cur_points, hmap.len()).as_str());
      }
    }
  }
}
