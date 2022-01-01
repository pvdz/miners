pub mod options;
pub mod slottable;
pub mod world;
pub mod values;
pub mod icons;
pub mod drone;
pub mod movable;
pub mod miner;
pub mod helix;
pub mod biome;
pub mod slot_hammer;
pub mod slot_drill;
pub mod async_stdin;
pub mod slot_purity_scanner;
pub mod slot_broken_gps;
pub mod slot_drone_launcher;
pub mod slot_energy_cell;
pub mod slot_emptiness;
// pub mod trie;
pub mod cell_contents;

use std::{thread, time};
use std::sync::mpsc::TryRecvError;

use std::time::SystemTime;

use rand::prelude::*;
use rand_pcg::{Pcg64, Lcg128Xsl64};
use rand::distributions::{Distribution, Uniform};

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
  // let mut trie = trie::Trie::new();
  // println!("da trie: {}", trie);
  //
  // println!("Now adding an entry...");
  //
  // let path = vec!(1, 3, -2000, 4);
  // let trail = trie.path_to_trail(path);
  // trie.write(&trail, 1);
  // println!("Trie now A: {}", trie);
  //
  // let path = vec!(1, 3, -3, 4);
  // let trail = trie.path_to_trail(path);
  // trie.write(&trail, 1);
  // println!("Trie now B: {}", trie);
  //
  // let path = vec!(1, 3, -3, 4);
  // let trail = trie.path_to_trail(path);
  // println!("result: {}", trie.read(&trail));
  //
  // let path = vec!(1, 3, -3, 5);
  // let trail = trie.path_to_trail(path);
  // println!("result: {}", trie.read(&trail));
  // panic!("hard stop");

  let mut options = options::parse_cli_args();

  if options.seed == 0 {
    // Did not receive a seed from the CLI so generate one now. We'll print it so if we find
    // something interesting we can re-play it reliably.
    let mut seed_rng = rand::thread_rng();
    let seed_range = Uniform::from(0..1000000);
    options.seed = seed_range.sample(&mut seed_rng);
  }
  println!("Seed: {}", options.seed);

  let mut delay = time::Duration::from_millis(options.speed);

  // This copy of rng is the one that is "random" for this whole run, not one epoch
  // It's seeded so are able to repro a run
  let mut instance_rng: Lcg128Xsl64  = Pcg64::seed_from_u64(options.seed);

  let mut best_miner: (helix::Helix, i32) = (
     helix::create_initial_helix(&mut instance_rng),
     0,
  );
  let mut best_min_x = 0;
  let mut best_min_y = 0;
  let mut best_max_x = 0;
  let mut best_max_y = 0;

  let stdin_channel = async_stdin::spawn_stdin_channel();

  let mut total_miner_count: i32 = 0;
  let start_time = SystemTime::now();

  loop
  {
    // Generate a bunch of biomes. Create a world for them and put a miner in there.
    // Each biome shares the same world (governed by the seed). But since the world is destructible
    // we have to give each biome their own world state.
    let mut biomes: Vec<biome::Biome> = vec!();
    for _ in 0..10 {
      let cur_miner: miner::Miner = miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)); // The helix will clone/copy. Can/should we prevent this?
      let own_world: world::World = world::generate_world(&options);
      let biome = biome::Biome {
        world: own_world,
        miner: cur_miner,
        path: vec!(0, 0),
      };
      biomes.push(biome);
    }

    total_miner_count = total_miner_count + (biomes.len() as i32);

    if options.visual {
      let table_str: String = world::serialize_world(&biomes[0].world, &biomes, best_miner, &options);
      println!("{}", table_str);
    }

    // Move it move it
    let mut iteration = 0; // How many iterations for the current cycle
    let mut has_energy = true; // As long as any miner in the current cycle has energy left...
    while has_energy {
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
          _ => (),
        }
        Err(TryRecvError::Empty) => (),
        Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
      }

      has_energy = false;
      for m in 0..biomes.len() {
        let biome = &mut biomes[m];
        if biome.miner.movable.energy > 0.0 {
          let mminer: &mut movable::Movable = &mut biome.miner.movable;
          let mmeta: &mut miner::MinerMeta = &mut biome.miner.meta;
          let mworld: &mut world::World = &mut biome.world;
          movable::move_movable(mminer, mmeta, mworld, &options);
          for i in 0..biome.miner.slots.len() {
            let slot: &mut slottable::Slottable = &mut biome.miner.slots[i];
            match slot.kind {
              slottable::SlotKind::Emptiness => (), // noop
              slottable::SlotKind::EnergyCell => slot_energy_cell::tick_slot_energy_cell(slot, mminer, mmeta, mworld, &options),
              slottable::SlotKind::DroneLauncher => (), // noop
              slottable::SlotKind::Hammer => (), // noop
              slottable::SlotKind::Drill => (), // noop
              slottable::SlotKind::PurityScanner => slot_purity_scanner::tick_slot_purity_scanner(slot, mmeta), // noop
              slottable::SlotKind::BrokenGps => slot_broken_gps::tick_slot_broken_gps(slot, mminer), // noop
              _ => {
                panic!("Fix slot range generator in helix")
              },
            }
          }
          // Does this miner still have energy left?
          if biome.miner.movable.energy > 0.0 {
            has_energy = true;
          }
        }
      }

      iteration = iteration + 1;

      if options.visual {
        // let table_str: String = world::serialize_world(&biomes[0].world, &biomes, best_miner, &options);
        // print!("\x1b[53A\n");
        // println!("{}", table_str);
        //
        // thread::sleep(delay);
      }

      for m in 0..biomes.len() {
        let biome: &mut biome::Biome = &mut biomes[m];
        // if biome.miner.meta.drone_gen_cooldown > 0 {
        //   biome.miner.meta.drone_gen_cooldown = biome.miner.meta.drone_gen_cooldown - 1;
        // }
        for slot in biome.miner.slots.iter_mut() {
          if biome.miner.meta.prev_move_bumped {
            let cooldown = slot.cur_cooldown;
            slot.cur_cooldown = cooldown + (cooldown * (biome.miner.helix.block_bump_cost / 50000.0));
          }
        }
      }
    }

    let mut winner = (
      biomes[0].miner.helix,
      (biomes[0].miner.meta.points as f64 * ((100.0 + biomes[0].miner.helix.multiplier_points as f64) / 100.0)) as i32,
      &biomes[0].world,
    );
    for m in 1..biomes.len() {
      let biome: &biome::Biome = &biomes[m];
      let points = (biome.miner.meta.points as f64 * ((100.0 + biome.miner.helix.multiplier_points as f64) / 100.0)) as i32;

      if points > winner.1 {
        winner = (
          biome.miner.helix,
          points,
          &biome.world,
        )
      }
    }

    if options.visual {
      print!("\x1b[{}A\n", 53 + biomes.len());

      for m in 1..biomes.len() {
        let biome: &biome::Biome = &biomes[m];
        let points = (biome.miner.meta.points as f64 * ((100.0 + biome.miner.helix.multiplier_points as f64) / 100.0)) as i32;
        println!("- Points: {} :: {}", points, biome.miner.helix);
      }
    }

    let mut he : String = "".to_string();
    helix::helix_to_string(&mut he, &winner.0);

    println!(
      "Out of energy! Time: {} s, iterations: {: >5}, miners: {}. Winner/Best points: {: >5} / {: >5}. Winner @ [{}x{} , {}x{}] -> {}{: >50}",
      match start_time.elapsed() { Ok(t) => t.as_secs(), _ => 99999 },
      iteration,
      total_miner_count,

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
      best_miner = (winner.0, winner.1);
      best_min_x = winner.2.min_x;
      best_min_y = winner.2.min_y;
      best_max_x = winner.2.max_x;
      best_max_y = winner.2.max_y;
    }
  }
}

