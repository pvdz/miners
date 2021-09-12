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

use std::{thread, time};

use rand::prelude::*;
use rand_pcg::Pcg64;
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

  // ░ ▒ ▓ █

  let mut init_rng = Pcg64::seed_from_u64(options.seed);
  let multiplier_range = Uniform::from(0..100);
  let multiplier_energy_start = multiplier_range.sample(&mut init_rng);
  let multiplier_points = 1;
  let multiplier_energy_pickup = multiplier_range.sample(&mut init_rng);

  let mut best_miner = miner::Miner {
    movable: movable::Movable {
      what: values::WHAT_MINER,
      x: values::WIDTH >> 1,
      y: values::HEIGHT >> 1,
      dir: values::DIR_UP,
      energy: 0,
    },
    meta: miner::MinerMeta {
      points: 0,
      max_energy: values::INIT_ENERGY,
      boredom_level: 0,
      boredom_rate: 10,
      boredom_steps: 0,
      drone_gen_cooldown: 50,
      multiplier_energy_start,
      multiplier_points,
      multiplier_energy_pickup,
      block_bump_cost: 5,
    },

    slots: [
      Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }),
      Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }),
      Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }),
      Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }),
      Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }),
      Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }),
      Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }),
      Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }), Box::new(emptiness::Emptiness { }),
    ],
  };

  let golden_map: world::World = world::generate_world(&options);

  // Print the initial world at least once
  let table_str: String = world::serialize_world(&golden_map, &best_miner);
  println!("{}", table_str);

  loop {

    let start_energy = (best_miner.meta.max_energy as f64 * (1.0 + multiplier_energy_start as f64 / 100.0)) as i32;
    let mut miner: miner::Miner = miner::Miner {
      movable: movable::Movable {
        what: values::WHAT_MINER,
        x: values::WIDTH >> 1,
        y: values::HEIGHT >> 1,
        dir: values::DIR_UP,
        energy: start_energy,
      },
      meta: miner::MinerMeta {
        max_energy: start_energy,
        points: 0,
        boredom_level: 0,
        boredom_rate: 0,
        boredom_steps: 0,
        drone_gen_cooldown: 50,
        multiplier_energy_start,
        multiplier_points,
        multiplier_energy_pickup,
        block_bump_cost: 5,
      },

      slots: [
        Box::new(drone_launcher::DroneLauncher { drone: drone::Drone { movable: movable::Movable { what: values::WHAT_DRONE, x: 0, y: 0, dir: values::DIR_DOWN, energy: 0 } } }),
        Box::new(drone_launcher::DroneLauncher { drone: drone::Drone { movable: movable::Movable { what: values::WHAT_DRONE, x: 0, y: 0, dir: values::DIR_DOWN, energy: 0 } } }),
        Box::new(drone_launcher::DroneLauncher { drone: drone::Drone { movable: movable::Movable { what: values::WHAT_DRONE, x: 0, y: 0, dir: values::DIR_DOWN, energy: 0 } } }),
        Box::new(drone_launcher::DroneLauncher { drone: drone::Drone { movable: movable::Movable { what: values::WHAT_DRONE, x: 0, y: 0, dir: values::DIR_DOWN, energy: 0 } } }),
        Box::new(energy_cell::EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
        Box::new(energy_cell::EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
        Box::new(energy_cell::EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
        Box::new(energy_cell::EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
        Box::new(emptiness::Emptiness { }),
      ],
    };

    // Recreate the rng fresh for every new Miner
    // let mut rng = Pcg64::seed_from_u64(options.seed);
    // let mut drone_rng = Pcg64::seed_from_u64(options.seed);

    let mut world: world::World = golden_map.clone();

    println!("Start {} x: {} y: {} dir: {} energy: {} points: {} multiplier_points: {} multiplier_energy_start: {} multiplier_energy_pickup: {}                 ", 0, miner.movable.x, miner.movable.y, miner.movable.dir, miner.movable.energy, miner.meta.points, miner.meta.multiplier_points, miner.meta.multiplier_energy_start, miner.meta.multiplier_energy_pickup);
    if options.visual {
      let table_str: String = world::serialize_world(&world, &miner);
      println!("{}", table_str);
    }

    // Move it move it
    let mut iteration = 0;
    while miner.movable.energy > 0 {

      movable::move_movable(&mut miner.movable, &mut miner.meta, &mut world);
      for i in 0..miner.slots.len() {
      // for slot in miner.slots.iter_mut() {
        miner.slots[i].before_paint(&mut miner.movable, &mut miner.meta, &mut world);
      }

      miner.movable.energy = miner.movable.energy - 1;
      iteration = iteration + 1;

      if options.visual {
        let table_str: String = world::serialize_world(&world, &miner);
        if options.visual {
          print!("\x1b[53A\n");
          // println!("update {} x: {} y: {} dir: {} energy: {} points: {} drone_cooldown: {}                         ", iteration + 1, miner.movable.x, miner.movable.y, miner.movable.dir, miner.movable.energy, miner.meta.points, miner.meta.drone_gen_cooldown);
          println!("{}", table_str);
        }

        thread::sleep(delay);
      }

      if miner.meta.drone_gen_cooldown > 0 {
        miner.meta.drone_gen_cooldown = miner.meta.drone_gen_cooldown - 1;
      }

      for slot in miner.slots.iter_mut() {
        slot.after_paint(&mut miner.movable, &mut miner.meta, &mut world);
      }
    }

    // TODO: use dedicated unseeded rng here, once we do.
    //
    // let prev_m_p = miner.meta.multiplier_points;
    // let mut delta_p = (prev_m_p as f64 * 0.05) as i32;
    // if delta_p == 0 {
    //   delta_p = 1;
    // }
    //
    // let prev_m_es = miner.meta.multiplier_energy_start;
    // let mut delta_es = (prev_m_es as f64 * 0.05) as i32;
    // if delta_es == 0 {
    //   delta_es = 1;
    // }
    //
    // let prev_m_ep = miner.meta.multiplier_energy_pickup;
    // let mut delta_ep = (prev_m_ep as f64 * 0.05) as i32;
    // if delta_ep == 0 {
    //   delta_ep = 1;
    // }

    let post_points = (miner.meta.points as f64 * ((100.0 + miner.meta.multiplier_points as f64) / 100.0)) as i32;
    let best_points = (best_miner.meta.points as f64 * ((100.0 + best_miner.meta.multiplier_points as f64) / 100.0)) as i32;
    if options.visual {
      print!("\x1b[55A\n");
    }
    println!("Out of energy! Iterations: {}, absolute points: {} final points: {}       ", iteration, miner.meta.points, post_points);
    if post_points > best_points {
      println!("Found a better miner {} to {} points                 ", best_points, post_points);
      best_miner = miner;
    }
  }
}

