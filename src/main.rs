pub mod options;
pub mod slottable;
pub mod world;
pub mod values;
pub mod drone;
pub mod slot_drone_launcher;
pub mod slot_energy_cell;
pub mod movable;
pub mod slot_emptiness;
pub mod miner;
pub mod helix;
pub mod dome;
pub mod slot_hammer;
pub mod slot_drill;
pub mod async_stdin;
pub mod slot_purity_scanner;
pub mod slot_broken_gps;

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


fn main() {
  println!("Starting....");

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
  // It's seeded so are able to repro a run (in case bugs happen) but I think we should not seed it to the map seed by default (TODO)
  let mut instance_rng: Lcg128Xsl64  = Pcg64::seed_from_u64(options.seed);
  // let multiplier_range = Uniform::from(0..100);
  // let multiplier_energy_start = multiplier_range.sample(&mut init_rng);
  // let multiplier_points = 1;
  // let multiplier_energy_pickup = multiplier_range.sample(&mut init_rng);

  let mut best_miner: (helix::Helix, i32) = (
     helix::create_initial_helix(&mut instance_rng),
     0,
  );

  let golden_map: world::World = world::generate_world(&options);

  let stdin_channel = async_stdin::spawn_stdin_channel();

  let mut miner_count: i32 = 0;
  let start_time = SystemTime::now();

  loop {
    let mut domes = [
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutate_helix(&mut instance_rng, best_miner.0, &options)), // The helix will clone/copy. Can/should we prevent this?
      },
    ];

    miner_count = miner_count + (domes.len() as i32);

    // Recreate the rng fresh for every new Miner
    // let mut rng = Pcg64::seed_from_u64(options.seed);
    // let mut drone_rng = Pcg64::seed_from_u64(options.seed);

    let world: world::World = golden_map.clone();

    // println!("Start {} x: {} y: {} dir: {} energy: {} {: >100}", 0, domes[0].miner.movable.x, domes[0].miner.movable.y, domes[0].miner.movable.dir, domes[0].miner.movable.energy, domes[0].miner.meta.points, ' ');
    if options.visual {
      let table_str: String = world::serialize_world(&world, &domes, best_miner, &options);
      println!("{}", table_str);
    }

    // Move it move it
    let mut iteration = 0;
    let mut has_energy = true;
    while has_energy {

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
          v => (),
        }
        Err(TryRecvError::Empty) => (),
        Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
      }

      has_energy = false;
      for m in 0..domes.len() {
        if domes[m].miner.movable.energy > 0 {
          movable::move_movable(&mut domes[m].miner.movable, &mut domes[m].miner.meta, &mut domes[m].world);
          for i in 0..domes[m].miner.slots.len() {
            domes[m].miner.slots[i].before_paint(&mut domes[m].miner.movable, &mut domes[m].miner.meta, &mut domes[m].world);
          }
          // Does this miner still have energy left?
          if domes[m].miner.movable.energy > 0 {
            has_energy = true;
          }
        }
      }

      iteration = iteration + 1;

      if options.visual {
        let table_str: String = world::serialize_world(&domes[0].world, &domes, best_miner, &options);
        if options.visual {
          print!("\x1b[53A\n");
          println!("{}", table_str);
        }

        thread::sleep(delay);
      }

      for m in 0..domes.len() {
        if domes[m].miner.meta.drone_gen_cooldown > 0 {
          domes[m].miner.meta.drone_gen_cooldown = domes[m].miner.meta.drone_gen_cooldown - 1;
        }
        for slot in domes[m].miner.slots.iter_mut() {
          if domes[m].miner.meta.prev_move_bumped {
            let cooldown = slot.get_cooldown();
            let mut new_cooldown = cooldown + (cooldown * (domes[m].miner.helix.block_bump_cost / 50000.0));
            slot.set_cooldown(new_cooldown);
          }
          slot.after_paint(&mut domes[m].miner.movable, &mut domes[m].miner.meta, &mut domes[m].world);
        }
      }
    }

    let mut winner = (
      domes[0].miner.helix,
      (domes[0].miner.meta.points as f64 * ((100.0 + domes[0].miner.helix.multiplier_points as f64) / 100.0)) as i32
    );
    for m in 1..domes.len() {
      let points = (domes[m].miner.meta.points as f64 * ((100.0 + domes[m].miner.helix.multiplier_points as f64) / 100.0)) as i32;

      if points > winner.1 {
        winner = (
          domes[m].miner.helix,
          points
        )
      }
    }

    if options.visual {
      print!("\x1b[{}A\n", 53 + domes.len());

      for m in 1..domes.len() {
        let points = (domes[m].miner.meta.points as f64 * ((100.0 + domes[m].miner.helix.multiplier_points as f64) / 100.0)) as i32;
        println!("- Points: {} :: {}", points, domes[m].miner.helix);
      }
    }

    let mut he : String = "".to_string();
    helix::helix_to_string(&mut he, &winner.0);

    println!(
      "Out of energy! Time: {} s, iterations: {: >5}, miners: {}. Winner/Best/Max points: {: >5} / {: >5} / {}. Winner {} {: >50}",
      match start_time.elapsed() { Ok(t) => t.as_secs(), _ => 99999 },
      iteration,
      miner_count,

      winner.1,
      best_miner.1,
      golden_map.max_points,

      he,
      ' '
    );
    if winner.1 > best_miner.1 {
      best_miner = winner;
    }
  }
}

