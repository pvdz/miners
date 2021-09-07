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
struct Miner {
  x: usize,
  y: usize,
  dir: i32,
  energy: i32,
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

fn move_miner(miner: &mut Miner, world: &mut World, nextx: usize, nexty: usize, nextdir: i32) {
  match world[nextx][nexty] {
    '█' => {
      world[nextx][nexty] = '▓';
      miner.dir = nextdir;
      miner.energy = miner.energy - miner.block_bump_cost;
    },
    '▓' => {
      world[nextx][nexty] = '▒';
      miner.dir = nextdir;
      miner.energy = miner.energy - miner.block_bump_cost;
    },
    '▒' => {
      world[nextx][nexty] = '░';
      miner.dir = nextdir;
      miner.energy = miner.energy - miner.block_bump_cost;
    },
    '░' => {
      world[nextx][nexty] = ICON_DIAMOND; // Or a different powerup?
      miner.dir = nextdir; // Or maybe not? Could be a miner property or powerup
      miner.energy = miner.energy - miner.block_bump_cost;
    },
    ICON_ENERGY => {
      miner.energy = miner.energy + (E_VALUE as f64 * ((100.0 + miner.multiplier_energy_pickup as f64) / 100.0)) as i32;
      world[nextx][nexty] = ' ';
      miner.x = nextx;
      miner.y = nexty;
    },
    ICON_DIAMOND => {
      miner.points = miner.points + 1; // Different gems with different points. Miners could have properties or powerups to affect this.
      world[nextx][nexty] = ' ';
      miner.x = nextx;
      miner.y = nexty;
    },
    _ => {
      miner.x = nextx;
      miner.y = nexty;
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

  #[allow(dead_code)]

  // ░ ▒ ▓ █

  let mut init_rng = Pcg64::seed_from_u64(options.seed);
  let multiplier_range = Uniform::from(0..100);
  let multiplier_energy_start = multiplier_range.sample(&mut init_rng);
  let multiplier_points = 100 - multiplier_energy_start;
  let multiplier_energy_pickup = multiplier_range.sample(&mut init_rng);

  let mut miner: Miner = Miner {
    x: WIDTH >> 1,
    y: HEIGHT >> 1,
    dir: DIR_UP,
    energy: (INIT_ENERGY as f64 * (multiplier_energy_start as f64 / 100.0)) as i32,
    points: 0,

    multiplier_energy_start,
    multiplier_points,
    multiplier_energy_pickup,
    block_bump_cost: 5,
  };
  let mut best_miner = miner;

  let mut golden_map: World = generate_world(&options);

  // Print the initial world at least once
  let table_str: String = serialize_world(&golden_map, miner.x, miner.y, miner.dir);
  println!("{}", table_str);

  loop {
    // Recreate the rng fresh for every new Miner
    let mut rng = Pcg64::seed_from_u64(options.seed);
    let mut world: World = golden_map.clone();

    println!("Start {} x: {} y: {} dir: {} energy: {} points: {} multiplier_points: {} multiplier_energy_start: {} multiplier_energy_pickup: {}                 ", 0, miner.x, miner.y, miner.dir, miner.energy, miner.points, miner.multiplier_points, miner.multiplier_energy_start, miner.multiplier_energy_pickup);
    println!("data here");
    let table_str: String = serialize_world(&world, miner.x, miner.y, miner.dir);
    println!("{}", table_str);

    // Move it move it
    let mut iteration = 0;
    while miner.energy > 0 {
      match miner.dir {
        DIR_UP => {
          let nextx: usize = miner.x.clone();
          let nexty = if miner.y == 0 { HEIGHT - 1 } else { miner.y - 1 };
          move_miner(&mut miner, &mut world, nextx, nexty, DIR_LEFT);
        },
        DIR_LEFT => {
          let nextx = if miner.x == 0 { WIDTH - 1 } else { miner.x - 1 };
          let nexty: usize = miner.y.clone();
          move_miner(&mut miner, &mut world, nextx, nexty, DIR_DOWN);
        },
        DIR_DOWN => {
          let nextx: usize = miner.x.clone();
          let nexty = if miner.y == HEIGHT - 1 { 0 } else { miner.y + 1 };
          move_miner(&mut miner, &mut world, nextx, nexty, DIR_RIGHT);
        },
        DIR_RIGHT => {
          let nextx = if miner.x == WIDTH - 1 { 0 } else { miner.x + 1 };
          let nexty: usize = miner.y.clone();
          move_miner(&mut miner, &mut world, nextx, nexty, DIR_UP);
        },

        _ => {
          println!("unexpected dir is: {}", miner.dir);
          panic!("dir is enum");
        },
      }

      miner.energy = miner.energy - 1;
      iteration = iteration + 1;

      if options.visual {
        let table_str: String = serialize_world(&world, miner.x, miner.y, miner.dir);
        print!("\x1b[54A\n");
        println!("update {} x: {} y: {} dir: {} energy: {} points: {}                 ", iteration + 1, miner.x, miner.y, miner.dir, miner.energy, miner.points);
        println!("{}", table_str);

        thread::sleep(delay);
      }
    }

    // TODO: use dedicated unseeded rng here, once we do.

    let prev_m_p = miner.multiplier_points;
    let mut delta_p = (prev_m_p as f64 * 0.05) as i32;
    if delta_p == 0 {
      delta_p = 1;
    }
    let next_m_p = prev_m_p + delta_p;

    let prev_m_es = miner.multiplier_energy_start;
    let mut delta_es = (prev_m_es as f64 * 0.05) as i32;
    if delta_es == 0 {
      delta_es = 1;
    }
    let next_m_es = prev_m_es + delta_es;

    let prev_m_ep = miner.multiplier_energy_pickup;
    let mut delta_ep = (prev_m_ep as f64 * 0.05) as i32;
    if delta_ep == 0 {
      delta_ep = 1;
    }
    let next_m_ep = prev_m_ep + delta_ep;

    let post_points = miner.points as i32 * ((miner.points as f64 * ((100.0 + miner.multiplier_points as f64) / 100.0)) as i32);
    let best_points = best_miner.points as i32 * ((best_miner.points as f64 * ((100.0 + best_miner.multiplier_points as f64) / 100.0)) as i32);
    print!("\x1b[55A\n");
    println!("Out of energy! Iterations: {}, absolute points: {} final points: {}       ", iteration, miner.points, post_points);
    if post_points > best_points {
      println!("Found a better miner {} to {} points                 ", best_points, post_points);
      best_miner = miner;
    }

    miner = Miner {
      x: WIDTH >> 1,
      y: HEIGHT >> 1,
      dir: DIR_UP,
      energy: (INIT_ENERGY as f64 * (next_m_es as f64 / 100.0)) as i32,
      points: 0,

      multiplier_points: next_m_p,
      multiplier_energy_start: next_m_es,
      multiplier_energy_pickup: next_m_ep,
      block_bump_cost: 5,
    };
  }
}

