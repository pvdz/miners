pub mod options;
pub mod slottable;
pub mod world;
pub mod values;
pub mod drone;
pub mod drone_launcher;
pub mod energy_cell;
pub mod movable;
pub mod emptiness;
pub mod miner;
pub mod helix;
pub mod dome;
pub mod hammer;
pub mod drill;
pub mod async_stdin;

use std::{thread, time};
use std::io::{self, Read};
use std::io::BufRead;
use crate::io::stdin;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;

use std::time::{Duration, SystemTime};

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

  let delay = time::Duration::from_millis(values::DELAY_MS);

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

  // Print the initial world at least once
  // let table_str: String = world::serialize_world(&golden_map, &best_miner, &best_miner);
  // println!("{}", table_str);

  let stdin_channel = async_stdin::spawn_stdin_channel();

  let mut miner_count: i32 = 0;
  let start_time = SystemTime::now();

  loop {
    let mut domes = [
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
      dome::Dome {
        world: golden_map.clone(),
        miner: miner::create_miner_from_helix(helix::mutated_helix(&mut instance_rng, best_miner.0), &mut instance_rng), // The helix will clone/copy. Can/should we prevent this?
      },
    ];

    miner_count = miner_count + (domes.len() as i32);

    // Recreate the rng fresh for every new Miner
    // let mut rng = Pcg64::seed_from_u64(options.seed);
    // let mut drone_rng = Pcg64::seed_from_u64(options.seed);

    let mut world: world::World = golden_map.clone();

    // println!("Start {} x: {} y: {} dir: {} energy: {} {: >100}", 0, domes[0].miner.movable.x, domes[0].miner.movable.y, domes[0].miner.movable.dir, domes[0].miner.movable.energy, domes[0].miner.meta.points, ' ');
    if options.visual {
      let table_str: String = world::serialize_world(&world, &domes, best_miner);
      println!("{}", table_str);
    }

    // Move it move it
    let mut iteration = 0;
    let mut has_energy = true;
    while has_energy {


      match stdin_channel.try_recv() {
        Ok(key) => match key.as_str() {
          "v\n" => options.visual = !options.visual,
          "v" => options.visual = !options.visual,
          v => panic!("wat? `{}`", v),
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
        let table_str: String = world::serialize_world(&domes[0].world, &domes, best_miner);
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
      print!("\x1b[{}A\n", 54 + domes.len());

      for m in 1..domes.len() {
        let points = (domes[m].miner.meta.points as f64 * ((100.0 + domes[m].miner.helix.multiplier_points as f64) / 100.0)) as i32;
        println!("- Points: {} :: {}", points, domes[m].miner.helix);
      }
    }

    println!(
      "Out of energy! Time: {} s, iterations: {: >5}, miners: {}. Best points: {: >5}, winner points: {: >5}. Helix: max energy: {: >5}, drone gen: {: >10}, bump cost: {: >10} {: >100}",
      match start_time.elapsed() { Ok(t) => t.as_secs(), _ => 99999 },
      iteration,
      miner_count,

      best_miner.1,
      winner.1,

      ((values::INIT_ENERGY as f32) * ((100.0 + winner.0.multiplier_energy_start) as f32) / 100.0) as i32,
      winner.0.drone_gen_cooldown,
      winner.0.block_bump_cost,
      ' '
    );
    if winner.1 > best_miner.1 {
      best_miner = winner;
    }
  }
}

