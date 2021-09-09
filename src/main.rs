use std::{thread, time};
use std::env;
use std::fmt::Write;

use rand::prelude::*;
use rand_pcg::Pcg64;
use rand::distributions::{Distribution, Uniform};

const WIDTH: usize = 50;
const HEIGHT: usize = 50;
const INIT_BLOCKS_PER_ROW: i32 = WIDTH as i32 >> 1; // Half?

const E_COUNT: i32 = 50; // How many modules do we spawn
const E_VALUE: i32 = 125; // Energy module bonus. 5%?
const INIT_ENERGY: i32 = 3000;

// TODO: this must be typeable :)
const DIR_UP   : i32 = 1;
const DIR_RIGHT: i32 = 2;
const DIR_DOWN : i32 = 3;
const DIR_LEFT : i32 = 4;

const DELAY_MS: u64 = 10;

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

const ICON_DIAMOND: char = '💎';
const ICON_ENERGY: char = '🔋';
const ICON_TURN_RIGHT: char = '🗘';
const ICON_HEAVY_UP: char = '🡅';
const ICON_HEAVY_RIGHT: char = '🡆';
const ICON_HEAVY_DOWN: char = '🡇';
const ICON_HEAVY_LEFT: char = '🡄';
const ICON_INDEX_UP: char = '👆';
const ICON_INDEX_RIGHT: char = '👉';
const ICON_INDEX_DOWN: char = '👇';
const ICON_INDEX_LEFT: char = '👈';

const ICON_MINER_UP: char = ICON_HEAVY_UP;
const ICON_MINER_RIGHT: char = ICON_HEAVY_RIGHT;
const ICON_MINER_DOWN: char = ICON_HEAVY_DOWN;
const ICON_MINER_LEFT: char = ICON_HEAVY_LEFT;

const ICON_DRONE_UP: char = ICON_INDEX_UP;
const ICON_DRONE_RIGHT: char = ICON_INDEX_RIGHT;
const ICON_DRONE_DOWN: char = ICON_INDEX_DOWN;
const ICON_DRONE_LEFT: char = ICON_INDEX_LEFT;

struct Options {
  seed: u64,
  visual: bool,
}
type World = [[char; HEIGHT]; WIDTH];

#[derive(Copy, Clone)]
struct Movable {
  x: usize,
  y: usize,
  dir: i32,
  energy: i32,
}

#[derive(Copy, Clone)]
struct Drone {
  // Each drone has its own x, y, direction, and energy
  movable: Movable,
}

#[derive(Copy, Clone)]
struct MinerMeta {
  points: i32,
  //  item:
  //  cooldown: i32, // Iterations before item can be used again
  drone_gen_cooldown: i32, // Generate a new drone every this many ticks

  // These multipliers are in whole percentages
  multiplier_energy_start: i32,
  multiplier_points: i32,
  block_bump_cost: i32,
  multiplier_energy_pickup: i32,
  //  multiplier_cooldown: i32,
}

#[derive(Copy, Clone)]
struct Miner {
  movable: Movable,
  meta: MinerMeta,
  // Whenever a drone is generated it will take a chunk of energy from the miner
  drones: [Option<Drone>; 32],
}

fn serialize_world(world: &World, miner: &Miner) -> String {
  // Clone the world so we can print the moving entities on it
  // Otherwise for each cell we'd have to scan all the entitie to check if they're on it
  // We could also construct an empty world with just the entities and check for non-zero instead
  let mut new_world: World = world.clone();
  new_world[miner.movable.x][miner.movable.y] = match miner.movable.dir {
    DIR_UP => ICON_MINER_UP,
    DIR_DOWN => ICON_MINER_DOWN,
    DIR_LEFT => ICON_MINER_LEFT,
    DIR_RIGHT => ICON_MINER_RIGHT,
    _ => {
      println!("unexpected dir: {:?}", miner.movable.dir);
      panic!("dir is enum");
    },
  };
  for drone in miner.drones.iter() {
    match drone {
      Some(drone) => {
        if drone.movable.energy > 0 {
          new_world[drone.movable.x][drone.movable.y] = match drone.movable.dir {
            DIR_UP => ICON_DRONE_UP,
            DIR_DOWN => ICON_DRONE_DOWN,
            DIR_LEFT => ICON_DRONE_LEFT,
            DIR_RIGHT => ICON_DRONE_RIGHT,
            _ => {
              println!("unexpected dir: {:?}", drone.movable.dir);
              panic!("dir is enum");
            },
          }
        }
      },
      None => ()
    }
  }

  let mut buf : String = "/".to_string();
  for _ in 0..WIDTH*2 {
    write!(buf, "-").unwrap();
  }
  write!(buf, "\\\n").unwrap();

  for y in 0..HEIGHT {
    write!(buf, "|").unwrap();
    for x in 0..WIDTH {
      let c: char = new_world[x][y];
      match c {
        | ICON_ENERGY
        | ICON_DIAMOND
        | ICON_TURN_RIGHT
        | ICON_INDEX_UP
        | ICON_INDEX_RIGHT
        | ICON_INDEX_LEFT
        | ICON_INDEX_DOWN
        => write!(buf, "{}", c).unwrap(),

        | ICON_HEAVY_UP
        | ICON_HEAVY_RIGHT
        | ICON_HEAVY_DOWN
        | ICON_HEAVY_LEFT
        => write!(buf, "{} ", c).unwrap(),

        v => write!(buf, "{0}{0}", v).unwrap(),
      }
    }
    write!(buf, "|\n").unwrap();
  }

  write!(buf, "\\").unwrap();
  for _ in 0..WIDTH*2 {
    write!(buf, "-").unwrap();
  }
  write!(buf, "/").unwrap();

  buf
}

fn parse_cli_args() -> Options {
  // Defaults:
  let mut options = Options {
    seed: 0,
    visual: true,
  };

  let args: Vec<String> = env::args().collect();
  println!("{:?}", args);


  let mut index = 1; // The first one is the binary path so let's skip that :)
  while index < args.len() {
    match args[index].as_str() {
      "--seed" => {
        index = index + 1;
        options.seed = args[index].trim().parse::<u64>().unwrap_or(0);
        if options.seed == 0 {
          panic!("Seed must be a non-zero positive integer");
        }
      }
      "--visual" => {
        options.visual = true;
      }
      "--no-visual" => {
        options.visual = false;
      }
      _ => {
        println!("Unknown parameter: {}", args[index]);
        panic!();
      }
    }

    index = index + 1;
  }
  
  options
}

fn move_it_xy(movable: &mut Movable, meta: &mut MinerMeta, world: &mut World, nextx: usize, nexty: usize, nextdir: i32) {
  match world[nextx][nexty] {
    '█' => {
      world[nextx][nexty] = '▓';
      movable.dir = nextdir;
      movable.energy = movable.energy - meta.block_bump_cost;
    },
    '▓' => {
      world[nextx][nexty] = '▒';
      movable.dir = nextdir;
      movable.energy = movable.energy - meta.block_bump_cost;
    },
    '▒' => {
      world[nextx][nexty] = '░';
      movable.dir = nextdir;
      movable.energy = movable.energy - meta.block_bump_cost;
    },
    '░' => {
      world[nextx][nexty] = ICON_DIAMOND; // Or a different powerup?
      movable.dir = nextdir; // Or maybe not? Could be a miner property or powerup
      movable.energy = movable.energy - meta.block_bump_cost;
    },
    ICON_ENERGY => {
      movable.energy = movable.energy + (E_VALUE as f64 * ((100.0 + meta.multiplier_energy_pickup as f64) / 100.0)) as i32;
      world[nextx][nexty] = ' ';
      movable.x = nextx;
      movable.y = nexty;
    },
    ICON_DIAMOND => {
      meta.points = meta.points + 1; // Different gems with different points. Miners could have properties or powerups to affect this.
      world[nextx][nexty] = ' ';
      movable.x = nextx;
      movable.y = nexty;
    },
    _ => {
      movable.x = nextx;
      movable.y = nexty;
    },
  }
}

fn move_movable(movable: &mut Movable, meta: &mut MinerMeta, world: &mut World) {
  match movable.dir {
    DIR_UP => {
      let nexty: usize = if movable.y == 0 { HEIGHT - 1 } else { movable.y - 1 };
      move_it_xy(movable, meta, world, movable.x, nexty, DIR_LEFT);
    },
    DIR_LEFT => {
      let nextx = if movable.x == 0 { WIDTH - 1 } else { movable.x - 1 };
      move_it_xy(movable, meta, world, nextx, movable.y, DIR_DOWN);
    },
    DIR_DOWN => {
      let nexty = if movable.y == HEIGHT - 1 { 0 } else { movable.y + 1 };
      move_it_xy(movable, meta, world, movable.x, nexty, DIR_RIGHT);
    },
    DIR_RIGHT => {
      let nextx = if movable.x == WIDTH - 1 { 0 } else { movable.x + 1 };
      move_it_xy(movable, meta, world, nextx, movable.y, DIR_UP);
    },

    _ => {
      println!("unexpected dir is: {}", movable.dir);
      panic!("dir is enum");
    },
  }
}

fn generate_world(options: &Options) -> World {
  let mut map_rng = Pcg64::seed_from_u64(options.seed);

  let diex = Uniform::from(0..WIDTH);
  let diey = Uniform::from(0..HEIGHT);

  // Generate the map for this run. We'll clone it for each cycle.
  let mut golden_map: World = [[' '; WIDTH]; HEIGHT];

  // Add energy modules
  for _ in 0..E_COUNT {
    let x = diex.sample(&mut map_rng);
    let y = diey.sample(&mut map_rng);
    golden_map[x][y] = ICON_ENERGY;
  }

  // Add blocks
  for x in 0..WIDTH {
    for _n in 0..INIT_BLOCKS_PER_ROW {
      let y = diey.sample(&mut map_rng);
      // Do not erase energy modules
      if golden_map[x][y] != ICON_ENERGY {
        golden_map[x][y] = '▓';
      }
    }
  }

  return golden_map;
}

fn main() {
  let mut options = parse_cli_args();

  if options.seed == 0 {
    // Did not receive a seed from the CLI so generate one now. We'll print it so if we find
    // something interesting we can re-play it reliably.
    let mut seed_rng = rand::thread_rng();
    let seed_range = Uniform::from(0..1000000);
    options.seed = seed_range.sample(&mut seed_rng);
  }
  println!("Seed: {}", options.seed);

  let delay = time::Duration::from_millis(DELAY_MS);

  // ░ ▒ ▓ █

  let mut init_rng = Pcg64::seed_from_u64(options.seed);
  let multiplier_range = Uniform::from(0..100);
  let multiplier_energy_start = multiplier_range.sample(&mut init_rng);
  let multiplier_points = 1;
  let multiplier_energy_pickup = multiplier_range.sample(&mut init_rng);

  let drone_range_x = Uniform::from(0..WIDTH);
  let drone_range_y = Uniform::from(0..HEIGHT);

  let mut miner: Miner = Miner {
    movable: Movable {
      x: WIDTH >> 1,
      y: HEIGHT >> 1,
      dir: DIR_UP,
      energy: (INIT_ENERGY as f64 * (multiplier_energy_start as f64 / 100.0)) as i32,
    },
    meta: MinerMeta {
      points: 0,
      drone_gen_cooldown: 50,
      multiplier_energy_start,
      multiplier_points,
      multiplier_energy_pickup,
      block_bump_cost: 5,
    },

    drones: [None; 32],
  };
  let mut best_miner = miner;

  let golden_map: World = generate_world(&options);

  // Print the initial world at least once
  let table_str: String = serialize_world(&golden_map, &miner);
  println!("{}", table_str);

  loop {
    // Recreate the rng fresh for every new Miner
    // let mut rng = Pcg64::seed_from_u64(options.seed);
    let mut drone_rng = Pcg64::seed_from_u64(options.seed);

    let mut world: World = golden_map.clone();

    println!("Start {} x: {} y: {} dir: {} energy: {} points: {} multiplier_points: {} multiplier_energy_start: {} multiplier_energy_pickup: {}                 ", 0, miner.movable.x, miner.movable.y, miner.movable.dir, miner.movable.energy, miner.meta.points, miner.meta.multiplier_points, miner.meta.multiplier_energy_start, miner.meta.multiplier_energy_pickup);
    if options.visual {
      let table_str: String = serialize_world(&world, &miner);
      println!("{}", table_str);
    }

    // Move it move it
    let mut iteration = 0;
    while miner.movable.energy > 0 {

      move_movable(&mut miner.movable, &mut miner.meta, &mut world);
      for drone in miner.drones.iter_mut() {
        match drone {
          Some(drone) => move_movable(&mut drone.movable, &mut miner.meta, &mut world),
          None => (),
        }
      }

      miner.movable.energy = miner.movable.energy - 1;
      iteration = iteration + 1;

      if options.visual {
        let table_str: String = serialize_world(&world, &miner);
        if options.visual {
          print!("\x1b[54A\n");
          println!("update {} x: {} y: {} dir: {} energy: {} points: {} drone_cooldown: {}                         ", iteration + 1, miner.movable.x, miner.movable.y, miner.movable.dir, miner.movable.energy, miner.meta.points, miner.meta.drone_gen_cooldown);
          println!("{}", table_str);
        }

        thread::sleep(delay);
      }

      if miner.meta.drone_gen_cooldown > 0 {
        miner.meta.drone_gen_cooldown = miner.meta.drone_gen_cooldown - 1;
      }
      let mut spawn_drone = miner.meta.drone_gen_cooldown == 0 && miner.movable.energy > 50;

      for i in 0..miner.drones.len() {
        let drone: Option<Drone> = miner.drones[i];
        match drone {
          Some(drone) => {
            if drone.movable.energy <= 0 {
              miner.drones[i] = None;
            }
          },
          None => {
            if spawn_drone {
              miner.drones[i] = Some(Drone { movable: Movable {
                x: miner.movable.x,
                y: miner.movable.y,
                dir: if miner.movable.dir == DIR_UP { DIR_DOWN } else { DIR_UP },
                energy: 50
              }});
              miner.meta.drone_gen_cooldown = 50;
              miner.movable.energy = miner.movable.energy - 50;
              spawn_drone = false;
            }
          },
        }
      }
    }

    // TODO: use dedicated unseeded rng here, once we do.

    let prev_m_p = miner.meta.multiplier_points;
    let mut delta_p = (prev_m_p as f64 * 0.05) as i32;
    if delta_p == 0 {
      delta_p = 1;
    }
    let next_m_p = prev_m_p + delta_p;

    let prev_m_es = miner.meta.multiplier_energy_start;
    let mut delta_es = (prev_m_es as f64 * 0.05) as i32;
    if delta_es == 0 {
      delta_es = 1;
    }
    let next_m_es = prev_m_es + delta_es;

    let prev_m_ep = miner.meta.multiplier_energy_pickup;
    let mut delta_ep = (prev_m_ep as f64 * 0.05) as i32;
    if delta_ep == 0 {
      delta_ep = 1;
    }
    let next_m_ep = prev_m_ep + delta_ep;

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

    miner = Miner {
      movable: Movable {
        x: WIDTH >> 1,
        y: HEIGHT >> 1,
        dir: DIR_UP,
        energy: (INIT_ENERGY as f64 * (next_m_es as f64 / 100.0)) as i32,
      },
      meta: MinerMeta {
        points: 0,
        drone_gen_cooldown: 50,
        multiplier_points: next_m_p,
        multiplier_energy_start: next_m_es,
        multiplier_energy_pickup: next_m_ep,
        block_bump_cost: 5,
      },
      drones: [None; 32],
    };
  }
}

