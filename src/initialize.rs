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

  // I expected a Trie to outperform a simple list but it seems that may not be the case.
  // Trie mode: Binary     It has    3179577 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:      20372) out of       2260 total. Each node stores   2+2 x i32 so naive encoding totals (  2+2)*4*3179577   =   24842 kb
  // Trie mode: B3         It has    2371508 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:      15321) out of       2260 total. Each node stores   3+2 x i32 so naive encoding totals (  3+2)*4*2371508   =   18530 kb
  // Trie mode: B4         It has    2087966 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:      13537) out of       2260 total. Each node stores   4+2 x i32 so naive encoding totals (  4+2)*4*2087966   =   16316 kb
  // Trie mode: B5         It has    1937885 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:      12620) out of       2260 total. Each node stores   5+2 x i32 so naive encoding totals (  5+2)*4*1937885   =   15144 kb
  // Trie mode: B6         It has    1853207 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:      12094) out of       2260 total. Each node stores   6+2 x i32 so naive encoding totals (  6+2)*4*1853207   =   14484 kb
  // Trie mode: B7         It has    1777314 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:      11608) out of       2260 total. Each node stores   7+2 x i32 so naive encoding totals (  7+2)*4*1777314   =   13892 kb
  // Trie mode: Octal      It has    3831644 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:      26593) out of       2260 total. Each node stores   8+2 x i32 so naive encoding totals (  8+2)*4*3831644   =   29942 kb
  // Trie mode: Decimal    It has    1639810 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:      10765) out of       2260 total. Each node stores  10+2 x i32 so naive encoding totals ( 10+2)*4*1639810   =   12821 kb
  // Trie mode: B15        It has    1577534 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:       8304) out of       2260 total. Each node stores  15+2 x i32 so naive encoding totals ( 15+2)*4*1577534   =   12339 kb
  // Trie mode: Hex        It has    2787106 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:      19544) out of       2260 total. Each node stores  16+2 x i32 so naive encoding totals ( 16+2)*4*2787106   =   21790 kb
  // Trie mode: Alpha      It has    1488812 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:       9785) out of       2260 total. Each node stores  26+2 x i32 so naive encoding totals ( 26+2)*4*1488812   =   11657 kb
  // Trie mode: Alnum      It has    1427520 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:       9416) out of       2260 total. Each node stores  36+2 x i32 so naive encoding totals ( 36+2)*4*1427520   =   11188 kb
  // Trie mode: AlUp       It has    1311942 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:       8709) out of       2260 total. Each node stores  62+2 x i32 so naive encoding totals ( 62+2)*4*1311942   =   10311 kb
  // Trie mode: B125       It has    1240957 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:       8304) out of       2260 total. Each node stores 125+2 x i32 so naive encoding totals (125+2)*4*1240957   =    9819 kb
  // Trie mode: BYTE       It has    1943041 nodes. It contains        413 unique paths (avg miner steps:       1721, avg trie path len:       1593) out of       2260 total. Each node stores 256+1 x i32 so naive encoding totals (256+1)*4*1943041   = 1950631 kb
  // Binary tree mode      It has        413 nodes. It contains        413 unique paths (avg miner steps:       1721, avg search len   :       1593) out of       2260 total. Each node stores (1+2*4*len) x i32 so naive totals      i32*4*2*413*1593  =    5139 kb
  // Both in terms of serialization as well as search time, a balanced binary tree should outperform a Trie. One caveat: the Trie should outperform in terms of serialization as the number of paths grows. TBD.

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

    for (_key, (points, unique_steps, serialized_helix)) in hmap.iter() {
      trail_lens += unique_steps.to_owned() as u64;
      if points.to_owned() > best_points_from_file {
        best_points_from_file = points.to_owned();
        best_steps_from_file = unique_steps.to_owned() as usize;
        best_helix_from_file = helix_deserialize(serialized_helix);
        load_best_as_miner_zero = true;
      }
    }

    println!("Loaded {} miners from disk. Average unique path len: {}. Most points: {}, with {} unique steps. Best helix: {}", len, trail_lens / len as u64, best_points_from_file, best_steps_from_file, best_helix_from_file);
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
