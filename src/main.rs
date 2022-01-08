pub mod options;
pub mod slottable;
pub mod world;
pub mod values;
pub mod icons;
pub mod drone;
pub mod movable;
pub mod cell;
pub mod miner;
pub mod helix;
pub mod inventory;
pub mod biome;
pub mod pickup;
pub mod slot_hammer;
pub mod slot_drill;
pub mod async_stdin;
pub mod slot_purity_scanner;
pub mod slot_broken_gps;
pub mod slot_drone_launcher;
pub mod slot_energy_cell;
pub mod slot_emptiness;
pub mod tile;
pub mod utils;

use std::{thread, time};
use std::borrow::Borrow;
use std::sync::mpsc::TryRecvError;
use std::fs;
use std::path::Path;

extern crate serde_json;
use std::collections::BTreeMap;

use std::time::SystemTime;

use rand::prelude::*;
use rand_pcg::{Pcg64, Lcg128Xsl64};
use rand::distributions::{Distribution, Uniform};
use crate::helix::create_initial_helix;

// Unique character ideas ("powerups with massive impact of which you should only need one")
// - random starting position
//   - Gives you a different path, period
// - double energy
//   - reach paths others cant (?)
// - diagonal movement
//   - reach paths others cant
// - random teleporter / glitching
//   - dunno if this makes sense. maybe hefty energy or slow reload or whatever.
// - bigfoot, one step moves you two spaces if you can
// - basher, don't change direction if you change a block into a diamond
// - radar, prefer to turn towards a diamond if you can

// Power up / character ability ideas:
// - after breaking a block do not change direction
// - break blocks two ticks per hit
// - double energy
// - wider reach? ability to hit a diagonal block
// - touch diamonds/items in the 9x9 around you
// - diamonds give you energy
// - active: generate random diamond / block
// - active: randomly hit a block (within radius? next hit hits twice?)
// - hook that auto-fires after cooldown, teleports you to the nearest forward facing block (provided there is one)
// - something that prevents an endless empty path with hefty cooldown?

// - Item to slowly construct paths (or ability? or auto? resource cost?) which reduce energy spent on that tile


fn main() {
  println!("Starting....");

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

  // https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
  // TODO: not relevant yet but when squeezing perf: does the btree on strings have amortized O(log2(n)+m) time rather than O(log2(n)*m) time? (does it remember string offset of previous step while traversing the tree?).
  let mut btree: BTreeMap<String, (u64, usize, helix::SerializedHelix)> = BTreeMap::new();
  let mut trail_lens: u64 = 0;

  let mut options = options::parse_cli_args();

  let mut best_points_from_file: u64 = 0;
  let mut best_steps_from_file: usize = 0;
  let mut best_helix_from_file: helix::Helix = helix::create_null_helix();
  let seed_btree_file = format!("./seed_{}.rson", options.seed);
  let seed_btree_path = Path::new(&seed_btree_file);
  let mut load_best_as_miner_zero = false;
  if options.seed > 0 && seed_btree_path.is_file() {
    println!("Loading from file... `{}`", seed_btree_file);
    let s = fs::read_to_string(&seed_btree_path).expect("Unable to read file");
    println!("Parsing {} bytes into btree", s.len());
    btree = serde_json::from_str(&s).unwrap();

    let len = btree.len();

    for (_key, (points, unique_steps, serialized_helix)) in btree.iter() {
      trail_lens += unique_steps.to_owned() as u64;
      if points.to_owned() > best_points_from_file {
        best_points_from_file = points.to_owned();
        best_steps_from_file = unique_steps.to_owned() as usize;
        best_helix_from_file = helix::helix_deserialize(serialized_helix);
        load_best_as_miner_zero = true;
      }
    }

    println!("Loaded {} miners from disk. Average unique path len: {}. Most points: {}, with {} unique steps. Best helix: {}", len, trail_lens / len as u64, best_points_from_file, best_steps_from_file, best_helix_from_file);
  }

  // When this gets set (by user interaction) the best miner is cleared and a new miner-seed is randomly picked.
  let mut reset = false;

  if options.seed == 0 {
    // Did not receive a seed from the CLI so generate one now. We'll print it so if we find
    // something interesting we can re-play it reliably.
    let mut seed_rng = rand::thread_rng();
    let seed_range = Uniform::from(0..1000000);
    options.seed = seed_range.sample(&mut seed_rng);
  }
  println!("World seed: {}", options.seed);

  let mut delay = time::Duration::from_millis(options.speed);

  // This copy of rng is the one that is "random" for this whole run, not one epoch
  // It's seeded so are able to repro a run. The initial miner is based on it as well.
  let mut instance_rng: Lcg128Xsl64 = Pcg64::seed_from_u64(options.seed);

  println!("Miner seed: {}", options.seed);
  let mut best_miner: (helix::Helix, u64, usize, usize) =
    if load_best_as_miner_zero {
      (
        best_helix_from_file,
        best_points_from_file,
        0,
        best_steps_from_file,
      )
    } else {
      (
        helix::create_initial_helix(&mut instance_rng, options.seed),
        best_points_from_file,
        0,
        best_steps_from_file,
      )
    };
  let mut best_min_x = 0;
  let mut best_min_y = 0;
  let mut best_max_x = 0;
  let mut best_max_y = 0;

  let mut next_root_helix = best_miner.0;

  let stdin_channel = async_stdin::spawn_stdin_channel();

  let mut total_miner_count: u32 = 0;
  let mut current_miner_count: u32 = 0;
  let start_time = SystemTime::now();

  let mut batches = 0;
  loop {
    batches += 1;

    // Generate a bunch of biomes. Create a world for them and put a miner in there.
    // Each biome shares the same world (governed by the seed). But since the world is destructible
    // we have to give each biome their own world state.
    let mut biomes: Vec<biome::Biome> = vec!();
    for i in 0..options.batch_size {
      let cur_miner: miner::Miner =
        if load_best_as_miner_zero {
          println!("loading best miner into biome {}... {}", i, next_root_helix);
          miner::create_miner_from_helix(next_root_helix)
        } else {
          miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, next_root_helix, &options)) // The helix will clone/copy. Can/should we prevent this?
      };
      load_best_as_miner_zero = false;
      let own_world: world::World = world::generate_world(&options);
      let biome = biome::Biome {
        world: own_world,
        miner: cur_miner,
        path: vec!(0, 0),
      };
      // println!("====== miner ======");
      // println!("miner slots: {:?}", &biome.miner.slots);
      // println!("===================");
      biomes.push(biome);
    }

    total_miner_count = total_miner_count + (biomes.len() as u32);
    current_miner_count = current_miner_count + (biomes.len() as u32);

    // Move it move it
    let mut iteration = 0; // How many iterations for the current cycle
    let mut has_energy = true; // As long as any miner in the current cycle has energy left...
    while has_energy && !reset {
      // This is basically the main game loop

      // Handle keyboard event
      match stdin_channel.try_recv() {
        Ok(key) => match key.as_str() {
          "v\n" => options.visual = !options.visual,
          "+\n" => {
            options.speed = (options.speed as f64 + (options.speed as f64 * 0.1).max(1.0)).max(1.0) as u64;
            delay = time::Duration::from_millis(options.speed);
          },
          "-\n" => {
            options.speed = (options.speed as f64 - (options.speed as f64 * 0.1).max(1.0)).max(1.0) as u64;
            delay = time::Duration::from_millis(options.speed);
          },
          "o\n" => options.mutation_rate_genes = (options.mutation_rate_genes - 1.0).max(0.0),
          "oo\n" => options.mutation_rate_genes = (options.mutation_rate_genes - 5.0).max(0.0),
          "p\n" => options.mutation_rate_genes = (options.mutation_rate_genes + 1.0).max(0.0),
          "pp\n" => options.mutation_rate_genes = (options.mutation_rate_genes + 5.0).max(0.0),
          "k\n" => options.mutation_rate_slots = (options.mutation_rate_slots - 1.0).max(0.0),
          "kk\n" => options.mutation_rate_slots = (options.mutation_rate_slots - 5.0).max(0.0),
          "l\n" => options.mutation_rate_slots = (options.mutation_rate_slots + 1.0).max(0.0),
          "ll\n" => options.mutation_rate_slots = (options.mutation_rate_slots + 5.0).max(0.0),
          "n\n" => options.batch_size = options.batch_size + 1,
          "m\n" => options.batch_size = (options.batch_size - 1).max(1),
          "r\n" => {
            reset = true;
            println!("Manual reset requested...");
          },
          "b\n" => {
            load_best_as_miner_zero = true;
            println!("Aborting current run and loading best as miner now...");
          }
          "q\n" => {
            // Save and quit.
            println!("Serializing btree with {} entries...", btree.len());
            let s = serde_json::to_string_pretty(&btree).unwrap();
            let f = format!("./seed_{}.rson", options.seed);
            println!("Storing {} bytes to `{}`", s.len(), f);
            fs::write(f, s).expect("Unable to write file");
            println!("Finished writing. Exiting now...");
            panic!("Quit after request");
          },
          _ => (),
        }
        Err(TryRecvError::Empty) => (),
        Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
      }

      if load_best_as_miner_zero {
        break;
      }

      if !reset && current_miner_count > options.reset_rate {
        println!("Auto reset after {} iterations, auto resets after {}", options.reset_rate, current_miner_count);
        reset = true;
      }

      if reset {
        let new_seed = instance_rng.next_u64();
        println!("New miner seed: {}", new_seed);
        next_root_helix = helix::create_initial_helix(&mut instance_rng, new_seed);
        current_miner_count = 0;

        // Do we reset other counters?

        break;
      }

      // Tick the biomes
      has_energy = false;
      for m in 0..biomes.len() {
        let first_miner = m == 0;
        let biome = &mut biomes[m];
        if biome.miner.movable.now_energy > 0.0 {
          let mminer: &mut movable::Movable = &mut biome.miner.movable;
          let mslots: &miner::MinerSlots = &mut biome.miner.slots;
          let mmeta: &mut miner::MinerMeta = &mut biome.miner.meta;
          let mdrones: &mut Vec<drone::Drone> = &mut biome.miner.drones;
          let mworld: &mut world::World = &mut biome.world;
          movable::move_movable(mminer, mslots, mmeta, mworld, &options);
          for i in 0..mslots.len() {
            let slot: &mut slottable::Slottable = &mut biome.miner.slots[i];
            match slot.kind {
              slottable::SlotKind::Emptiness => (), // noop
              slottable::SlotKind::EnergyCell => slot_energy_cell::tick_slot_energy_cell(slot, mminer, mmeta, mworld, &options, first_miner),
              slottable::SlotKind::DroneLauncher => slot_drone_launcher::tick_slot_drone_launcher(slot, mminer, mdrones, mmeta, mworld, &options, first_miner),
              slottable::SlotKind::Hammer => (), // noop
              slottable::SlotKind::Drill => (), // noop
              slottable::SlotKind::PurityScanner => slot_purity_scanner::tick_slot_purity_scanner(slot, mmeta, first_miner),
              slottable::SlotKind::BrokenGps => slot_broken_gps::tick_slot_broken_gps(slot, mminer, first_miner),
              _ => {
                panic!("Fix slot range generator in helix")
              },
            }
          }

          // Does this miner still have energy left?
          if biome.miner.movable.now_energy > 0.0 {
            has_energy = true;
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

            // TODO: ugh. get rid of the trail formats in this part... I just made it work for now.
            let has_trail: bool = btree.contains_key(format!("{}", trail).as_str());
            // TODO: why can't it do `.get()?` ? It refuses to do the `?` thing.
            let cur_points = inventory::get_points(&biome.miner.meta.inventory);
            let ok = btree.entry(format!("{}", trail)).or_insert((cur_points, biome.miner.movable.unique.len(), helix::helix_serialize(&biome.miner.helix)));
            if cur_points > ok.0 {
              println!("Helix improved a path, from {} to {}. Helix: {}", ok.0, cur_points, biome.miner.helix);
              ok.0 = cur_points;
              ok.1 = biome.miner.movable.unique.len();
              ok.2 = helix::helix_serialize(&biome.miner.helix);
            }
            if has_trail {
              // println!("This miner was already recorded...");
            } else {
              println!("This miner was new! trail has {} / {} steps and results in {} points. Tree now contains {} trails.", biome.miner.movable.unique.len(), biome.miner.movable.history.len(), cur_points, btree.len());
              trail_lens += biome.miner.movable.unique.len() as u64;
            }
          }
        }
      }

      iteration = iteration + 1;

      // Stop drawing the world when the main miner is out of energy. Speed things up visually.
      if options.visual && biomes[0].miner.movable.now_energy > 0.0 {
        let table_str: String = world::serialize_world(
          &biomes[0].world,
          &biomes,
          &options,
          format!("Best miner: Points: {}  Steps: {} ({})   Map: {}x{} ~ {}x{}  {}", best_miner.1, best_miner.2, best_miner.3, best_min_x, best_min_y, best_max_x, best_max_y, best_miner.0),
          format!("Miner Dictionary contains {} entries. Average steps: {}.", btree.len(), trail_lens / btree.len().max(1) as u64),
        );
        print!("\x1b[58A\n");
        println!("{}", table_str);

        thread::sleep(delay);
      }

      // As a way to balance the block_bump_cost value; the higher that penalty is, the faster
      // your slots cool down. The markup should not be major but probably if block_bump_cost is
      // close to zero, the slots cooldowns should not get any boosts.
      for m in 0..biomes.len() {
        let biome: &mut biome::Biome = &mut biomes[m];
        if biome.miner.movable.now_energy > 0.0 {
          for slot in biome.miner.slots.iter_mut() {
            if biome.miner.meta.prev_move_bumped {
              slot.cur_cooldown *= 1.0 + (biome.miner.helix.block_bump_cost / 50000.0);
            }
          }
        }
      }
    }

    if load_best_as_miner_zero {
      continue;
    }

    if reset {
      reset = false;
      println!("Resetting helix...");
      continue;
    }

    let mut winner = (
      biomes[0].miner.helix,
      inventory::get_points(&biomes[0].miner.meta.inventory) as u64,
      &biomes[0].world,
      biomes[0].miner.movable.history.len(),
      biomes[0].miner.movable.unique.len(),
    );

    for m in 1..biomes.len() { // 1 because zero is used as init above
      let biome: &biome::Biome = &biomes[m];
      let points = inventory::get_points(&biome.miner.meta.inventory) as u64;

      if points > winner.1 {
        winner = (
          biome.miner.helix,
          points,
          &biome.world,
          biome.miner.movable.history.len(),
          biome.miner.movable.unique.len(),
        )
      }
    }

    if options.visual {
      for m in 0..biomes.len() {
        let biome: &biome::Biome = &biomes[m];
        let points = inventory::get_points(&biome.miner.meta.inventory) as u64;
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
    helix::helix_to_string(&mut he, &winner.0);

    println!(
      "Time: {} s, batches: {: <5} iterations: {: <5} miners: {}, in current seed: {}. Winner/Best points: {: >5} / {: >5}. Winner @ [{}x{} , {}x{}] -> {}{: >50}",
      match start_time.elapsed() { Ok(t) => t.as_secs(), _ => 99999 },
      batches,
      iteration,
      total_miner_count,
      current_miner_count,

      winner.1,
      best_miner.1,
      best_min_x,
      best_min_y,
      best_max_x,
      best_max_y,

      he,
      ' '
    );

    if winner.1 > best_miner.1 {
      println!("\x1b[32;1mFound a new best!\x1b[0m: From {} to {}", best_miner.1, winner.1);
      best_miner = (winner.0, winner.1, winner.3, winner.4); // helix, points, steps, uniques
      next_root_helix = winner.0;
      best_min_x = winner.2.min_x;
      best_min_y = winner.2.min_y;
      best_max_x = winner.2.max_x;
      best_max_y = winner.2.max_y;
    }

    println!(
      "Binary tree mode has {} nodes with average trail len of {}.",
      btree.len(),
      trail_lens / btree.len() as u64
    );

    // println!("map: {}", serde_json::to_string_pretty(&btree).unwrap());
    // panic!("halt");
  }
}

