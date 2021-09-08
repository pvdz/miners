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
const INIT_ENERGY: i32 = 500;

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

fn serialize_world(array: &[[char; HEIGHT]; WIDTH], miner_x: usize, miner_y: usize, miner_dir: i32) -> String {
  let mut buf : String = "/".to_string();
  for _ in 0..WIDTH*2 {
    write!(buf, "-").unwrap();
  }
  write!(buf, "\\\n").unwrap();

  for y in 0..HEIGHT {
    write!(buf, "|").unwrap();
    for x in 0..WIDTH {
      if x == miner_x && y == miner_y {
        match miner_dir {
          DIR_UP => write!(buf, "^ ").unwrap(),
          DIR_DOWN => write!(buf, "v ").unwrap(),
          DIR_LEFT => write!(buf, "< ").unwrap(),
          DIR_RIGHT => write!(buf, "> ").unwrap(),
          _ => {
            println!("unexpected dir: {:?}", miner_dir);
            panic!("dir is enum");
          },
        };
      } else {
        match array[x][y] {
          ICON_ENERGY => write!(buf, "{}", ICON_ENERGY).unwrap(),
          ICON_DIAMOND => write!(buf, "{}", ICON_DIAMOND).unwrap(),
          v => write!(buf, "{0}{0}", v).unwrap(),
        }
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
      let nextx: usize = movable.x.clone();
      let nexty: usize = if movable.y == 0 { HEIGHT - 1 } else { movable.y - 1 };
      move_it_xy(movable, meta, world, nextx, nexty, DIR_LEFT);
    },
    DIR_LEFT => {
      let nextx = if movable.x == 0 { WIDTH - 1 } else { movable.x - 1 };
      let nexty: usize = movable.y.clone();
      move_it_xy(movable, meta, world, nextx, nexty, DIR_DOWN);
    },
    DIR_DOWN => {
      let nextx: usize = movable.x.clone();
      let nexty = if movable.y == HEIGHT - 1 { 0 } else { movable.y + 1 };
      move_it_xy(movable, meta, world, nextx, nexty, DIR_RIGHT);
    },
    DIR_RIGHT => {
      let nextx = if movable.x == WIDTH - 1 { 0 } else { movable.x + 1 };
      let nexty: usize = movable.y.clone();
      move_it_xy(movable, meta, world, nextx, nexty, DIR_UP);
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
  let multiplier_points = 100 - multiplier_energy_start;
  let multiplier_energy_pickup = multiplier_range.sample(&mut init_rng);

  let mut miner: Miner = Miner {
    movable: Movable {
      x: WIDTH >> 1,
      y: HEIGHT >> 1,
      dir: DIR_UP,
      energy: (INIT_ENERGY as f64 * (multiplier_energy_start as f64 / 100.0)) as i32,
    },
    meta: MinerMeta {
      points: 0,
      multiplier_energy_start,
      multiplier_points,
      multiplier_energy_pickup,
      block_bump_cost: 5,
    },

    drones: [None; 32],
  };
  let mut best_miner = miner;

  // miner.drones[0] = Some(Drone { movable: Movable { x: 0, y: 0, dir: DIR_DOWN, energy: 50 }});

  let golden_map: World = generate_world(&options);

  // Print the initial world at least once
  let table_str: String = serialize_world(&golden_map, miner.movable.x, miner.movable.y, miner.movable.dir);
  println!("{}", table_str);

  loop {
    // Recreate the rng fresh for every new Miner
    // let mut rng = Pcg64::seed_from_u64(options.seed);
    let mut world: World = golden_map.clone();

    println!("Start {} x: {} y: {} dir: {} energy: {} points: {} multiplier_points: {} multiplier_energy_start: {} multiplier_energy_pickup: {}                 ", 0, miner.movable.x, miner.movable.y, miner.movable.dir, miner.movable.energy, miner.meta.points, miner.meta.multiplier_points, miner.meta.multiplier_energy_start, miner.meta.multiplier_energy_pickup);
    println!("data here");
    let table_str: String = serialize_world(&world, miner.movable.x, miner.movable.y, miner.movable.dir);
    println!("{}", table_str);

    // Move it move it
    let mut iteration = 0;
    while miner.movable.energy > 0 {

      move_movable(&mut miner.movable, &mut miner.meta, &mut world);

      miner.movable.energy = miner.movable.energy - 1;
      iteration = iteration + 1;

      if options.visual {
        let table_str: String = serialize_world(&world, miner.movable.x, miner.movable.y, miner.movable.dir);
        print!("\x1b[54A\n");
        println!("update {} x: {} y: {} dir: {} energy: {} points: {}                 ", iteration + 1, miner.movable.x, miner.movable.y, miner.movable.dir, miner.movable.energy, miner.meta.points);
        println!("{}", table_str);

        thread::sleep(delay);
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

    let post_points = miner.meta.points as i32 * ((miner.meta.points as f64 * ((100.0 + miner.meta.multiplier_points as f64) / 100.0)) as i32);
    let best_points = best_miner.meta.points as i32 * ((best_miner.meta.points as f64 * ((100.0 + best_miner.meta.multiplier_points as f64) / 100.0)) as i32);
    print!("\x1b[55A\n");
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
        multiplier_points: next_m_p,
        multiplier_energy_start: next_m_es,
        multiplier_energy_pickup: next_m_ep,
        block_bump_cost: 5,
      },
      drones: [None; 32],
    };
  }
}

