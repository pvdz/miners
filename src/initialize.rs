use super::helix::*;
use super::options::*;
use super::app_state::*;
use super::inventory::*;

use std::fs;
use std::path::Path;

use rand::prelude::*;
use rand_pcg::{Pcg64, Lcg128Xsl64};
use rand::distributions::{Distribution, Uniform};

use std::collections::HashMap;

extern crate serde_json;

pub fn initialize(options: &mut Options) -> (AppState, Helix, HashMap<u64, (u64, usize, SerializedHelix)>) {
  // https://doc.rust-lang.org/std/collections/struct.HashMap.html
  let mut hmap: HashMap<u64, (u64, usize, SerializedHelix)> = HashMap::new();
  let mut trail_lens: u64 = 0;

  let mut best_points_from_file: u64 = 0;
  let mut best_steps_from_file: usize = 0;
  let mut best_helix_from_file: Helix = create_null_helix();
  let seed_hmap_file = format!("./seed_{}.rson", options.seed);
  let seed_hmap_path = Path::new(&seed_hmap_file);
  let mut load_best_as_miner_zero = false;
  if options.seed > 0 && seed_hmap_path.is_file() {
    println!("Loading from file... `{}`", seed_hmap_file);
    let s = fs::read_to_string(&seed_hmap_path).expect("Unable to read file");
    println!("Parsing {} bytes into hash map", s.len());
    hmap = serde_json::from_str(&s).unwrap();

    let len = hmap.len();

    for (_key, (points, _unique_steps, serialized_helix)) in hmap.iter() {
      if points.to_owned() > best_points_from_file {
        best_points_from_file = points.to_owned();
        best_helix_from_file = helix_deserialize(serialized_helix);
        load_best_as_miner_zero = true;
      }
    }

    println!("Loaded {} miners from disk. Most points: {}. Best helix: {}", len, best_points_from_file, best_helix_from_file);
  }

  if options.seed == 0 {
    // Did not receive a seed from the CLI so generate one now. We'll print it so if we find
    // something interesting we can re-play it reliably.
    let mut seed_rng = rand::thread_rng();
    let seed_range = Uniform::from(0..1000000);
    options.seed = seed_range.sample(&mut seed_rng);
  }
  println!("World seed: {}", options.seed);

  // let mut delay = time::Duration::from_millis(options.speed);

  // This copy of rng is the one that is "random" for this whole run, not one epoch
  // It's seeded so are able to repro a run. The initial miner is based on it as well.
  let mut instance_rng: Lcg128Xsl64 = Pcg64::seed_from_u64(options.seed);

  println!("Miner seed: {}", options.seed);
  let new_inv = create_inventory();
  let best_miner: (Helix, u64, usize, usize, Inventory) =
    if options.initial_miner_code.len() != 0 {
      let x: (
        u64, // seed
        f32, // drone_gen_cooldown
        f32, // multiplier_energy_start
        f32, // multiplier_points
        f32, // block_bump_cost
        f32, // multiplier_energy_pickup
        String // slots: [SlotKind; 32]
      ) = serde_json::from_str(&options.initial_miner_code).unwrap();
      let y = helix_deserialize(&x);
      (
        y,
        0,
        0,
        0,
        new_inv
      )
    } else if load_best_as_miner_zero {
      (
        best_helix_from_file,
        best_points_from_file,
        0,
        best_steps_from_file,
        new_inv,
      )
    } else {
      (
        create_initial_helix(&mut instance_rng, options.seed),
        best_points_from_file,
        0,
        best_steps_from_file,
        new_inv
      )
    };

  let next_root_helix = best_miner.0;
  let mut state = create_app_state(options, best_miner, trail_lens, instance_rng);
  state.load_best_as_miner_zero = load_best_as_miner_zero;

  return (state, next_root_helix, hmap);
}
