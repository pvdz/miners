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

use std::{thread, time};

use rand::prelude::*;
use rand_pcg::{Pcg64, Lcg128Xsl64};
use rand::distributions::{Distribution, Uniform};

// Power up / character ability ideas:
// - after breaking a block do not change direction
// - break blocks two ticks per hit
// - double energy
// - random starting position?
// - wider reach? ability to hit a diagonal block
// - ability to move diagonally, too?
// - touch diamonds/items in the 9x9 around you
// - diamonds give you energy
// - active: generate random diamond / block
// - when you hit a block, also hit all blocks behind it up to the first non-block cell
// - active: randomly hit a block (within radius? next hit hits twice?)
// - if you can move forward two spaces, you do (this could also be an active)
// - if turning left or right puts you towards a diamond, prefer that (something radar ish)
// - split up? could share energy source or at least deplete at a faster rate, collisions destroy you, stuff like that.


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
    ];

    // Recreate the rng fresh for every new Miner
    // let mut rng = Pcg64::seed_from_u64(options.seed);
    // let mut drone_rng = Pcg64::seed_from_u64(options.seed);

    let mut world: world::World = golden_map.clone();

    println!("Start {} x: {} y: {} dir: {} energy: {} points: {} {: >100}", 0, domes[0].miner.movable.x, domes[0].miner.movable.y, domes[0].miner.movable.dir, domes[0].miner.movable.energy, domes[0].miner.meta.points, ' ');
    if options.visual {
      let table_str: String = world::serialize_world(&world, &domes[0].miner, best_miner);
      println!("{}", table_str);
    }

    // Move it move it
    let mut iteration = 0;
    while domes[0].miner.movable.energy > 0 {

      for m in 0..domes.len() {
        movable::move_movable(&mut domes[m].miner.movable, &mut domes[m].miner.meta, &mut domes[m].world);
        for i in 0..domes[m].miner.slots.len() {
          domes[m].miner.slots[i].before_paint(&mut domes[m].miner.movable, &mut domes[m].miner.meta, &mut domes[m].world);
        }
      }

      iteration = iteration + 1;

      if options.visual {
        let table_str: String = world::serialize_world(&domes[0].world, &domes[0].miner, best_miner);
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
      print!("\x1b[{}A\n", 55 + domes.len());

      for m in 1..domes.len() {
        let points = (domes[m].miner.meta.points as f64 * ((100.0 + domes[m].miner.helix.multiplier_points as f64) / 100.0)) as i32;
        println!("- Points: {} :: {}", points, domes[m].miner.helix);
      }
    }

    println!(
      "Out of energy! Iterations: {}, max energy: {}, final points: {} best points was: {} {: >100}",
      iteration,
      ((values::INIT_ENERGY as f32) * ((100.0 + winner.0.multiplier_energy_start) as f32) / 100.0) as i32,
      winner.1,
      best_miner.1,
      ' '
    );
    if winner.1 > best_miner.1 {
      println!("Found a better miner {} to {} points {: >100}", best_miner.1, winner.1, ' ');
      best_miner = winner;
    } else {
      println!("Did not find a better miner. Current best: {} points {: >100}", best_miner.1, ' ');
    }
  }
}

