use std::{thread, time};
use std::env;

use rand::prelude::*;
use rand_pcg::Pcg64;
use rand::distributions::{Distribution, Uniform};

const WIDTH: usize = 50;
const HEIGHT: usize = 50;
const INIT_BLOCKS_PER_ROW: i32 = WIDTH as i32 >> 1; // Half?

const E_COUNT: i32 = 50; // How many modules do we spawn
const E_VALUE: i32 = 125; // Energy module bonus. 5%?
const INIT_ENERGY: i32 = 1000;

// TODO: this must be typeable :)
const DIR_UP   : i32 = 1;
const DIR_RIGHT: i32 = 2;
const DIR_DOWN : i32 = 3;
const DIR_LEFT : i32 = 4;

const DELAY_MS: u64 = 35;

// Power up / character ability ideas:
// - after breaking a block do not change direction
// - break blocks two ticks per hit
// - double energy
// - random starting position?
// - wider reach? ability to hit a diagonal block
// - ability to move diagionally, too?
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

struct Options {
  seed: u64,
  visual: bool,
}

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
  multiplier_energy: i32,
  multiplier_points: i32,
//  multiplier_cooldown: i32,
}

//fn print_table(array: &[[char; HEIGHT]; WIDTH], miner: &Miner) {
fn print_table(array: &[[char; HEIGHT]; WIDTH], miner_x: usize, miner_y: usize, miner_dir: i32) {
  print!("/");
  for _ in 0..WIDTH*2 {
    print!("-");
  }
  println!("\\");

  for y in 0..HEIGHT {
    print!("|");
    for x in 0..WIDTH {
//      if x == miner.x && y == miner.y {
      if x == miner_x && y == miner_y {
//        match miner.dir.as_str() {
        match miner_dir {
          DIR_UP => print!("^ "),
          DIR_DOWN => print!("v "),
          DIR_LEFT => print!("< "),
          DIR_RIGHT => print!("> "),
          _ => {
            println!("unexpected dir: {:?}", miner_dir);
            panic!("dir is enum");
          },
        };
      } else {
        match array[x][y] {
          ICON_ENERGY => print!("{}", ICON_ENERGY),
          ICON_DIAMOND => print!("{}", ICON_DIAMOND),
          v => print!("{0}{0}", v),
        }
      }
    }
    println!("|");
  }

  print!("\\");
  for _ in 0..WIDTH*2 {
    print!("-");
  }
  println!("/");
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

  let mut map_rng = Pcg64::seed_from_u64(options.seed);

  let delay = time::Duration::from_millis(DELAY_MS);

  // ░ ▒ ▓ █

  // TODO: Use dedicated rng. Doesn't really matter here yet but maybe later.
  let multiplier_range = Uniform::from(0..100);
  let multiplier_energy = multiplier_range.sample(&mut map_rng);
  let multiplier_points = 100 - multiplier_energy;

  println!("e {} m {} p {} so {}", INIT_ENERGY, multiplier_energy, (multiplier_energy as f64 / 100.0), (INIT_ENERGY as f64 * (multiplier_energy as f64 / 100.0)) as i32);

  let mut miner: Miner = Miner {
    x: WIDTH >> 1,
    y: HEIGHT >> 1,
    dir: DIR_UP,
    energy: (INIT_ENERGY as f64 * (multiplier_energy as f64 / 100.0)) as i32,
    points: 0,

    multiplier_energy,
    multiplier_points,
  };
  let mut best_miner = miner;

  let diex = Uniform::from(0..WIDTH);
  let diey = Uniform::from(0..HEIGHT);

  // Generate the map for this run. We'll clone it for each cycle.
  let mut golden_map: [[char; HEIGHT]; WIDTH] = [[' '; WIDTH]; HEIGHT];

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

  // Print the initial world at least once
  print_table(&golden_map, miner.x, miner.y, miner.dir);

  loop {
    // Recreate the rng fresh for every new Miner
    let mut rng = Pcg64::seed_from_u64(options.seed);
    let mut array = golden_map.clone();

    println!("Start {} x: {} y: {} dir: {} energy: {} points: {} multiplier_points: {} multiplier_energy: {}", 0, miner.x, miner.y, miner.dir, miner.energy, miner.points, miner.multiplier_points, miner.multiplier_energy);

    // Move it move it
    let mut iteration = 0;
    while miner.energy > 0 {
      match miner.dir {
        DIR_UP => {
          let nexty = if miner.y == 0 { HEIGHT - 1 } else { miner.y - 1 };
          match array[miner.x][nexty] {
            '█' => {
              array[miner.x][nexty] = '▓';
              miner.dir = DIR_LEFT;
            },
            '▓' => {
              array[miner.x][nexty] = '▒';
              miner.dir = DIR_LEFT;
            },
            '▒' => {
              array[miner.x][nexty] = '░';
              miner.dir = DIR_LEFT;
            },
            '░' => {
              array[miner.x][nexty] = ICON_DIAMOND; // Or a different powerup?
              miner.dir = DIR_LEFT; // Or maybe not? Could be a miner property or powerup
            },
           ICON_ENERGY => {
              miner.energy = miner.energy + E_VALUE;
              array[miner.x][nexty] = ' ';
              miner.y = nexty;
            },
           ICON_DIAMOND => {
              miner.points = miner.points + 1; // Different gems with different points. Miners could have properties or powerups to affect this.
              array[miner.x][nexty] = ' ';
              miner.y = nexty;
            },
            _ => miner.y = nexty,
          }          
        },
        DIR_LEFT => {
          let nextx = if miner.x == 0 { WIDTH - 1 } else { miner.x - 1 };
          match array[nextx][miner.y] {
            '█' => {
              array[nextx][miner.y] = '▓';
              miner.dir = DIR_DOWN;
            },
            '▓' => {
              array[nextx][miner.y] = '▒';
              miner.dir = DIR_DOWN;
            },
            '▒' => {
              array[nextx][miner.y] = '░';
              miner.dir = DIR_DOWN;
            },
            '░' => {
              array[nextx][miner.y] = ICON_DIAMOND;
              miner.dir = DIR_DOWN;
            },
           ICON_ENERGY => {
              miner.energy = miner.energy + E_VALUE;
              array[nextx][miner.y] = ' ';
              miner.x = nextx;
            },
           ICON_DIAMOND => {
              miner.points = miner.points + 1;
              array[nextx][miner.y] = ' ';
              miner.x = nextx;
            },
            _ => miner.x = nextx,
          }
        },
        DIR_DOWN => {
          let nexty = if miner.y == HEIGHT - 1 { 0 } else { miner.y + 1 };
          match array[miner.x][nexty] {
            '█' => {
              array[miner.x][nexty] = '▓';
              miner.dir = DIR_RIGHT;
            },
            '▓' => {
              array[miner.x][nexty] = '▒';
              miner.dir = DIR_RIGHT;
            },
            '▒' => {
              array[miner.x][nexty] = '░';
              miner.dir = DIR_RIGHT;
            },
            '░' => {
              array[miner.x][nexty] = ICON_DIAMOND;
              miner.dir = DIR_RIGHT;
            },
           ICON_ENERGY => {
              miner.energy = miner.energy + E_VALUE;
              array[miner.x][nexty] = ' ';
              miner.y = nexty;
            },
           ICON_DIAMOND => {
              miner.points = miner.points + 1;
              array[miner.x][nexty] = ' ';
              miner.y = nexty;
            },
            _ => miner.y = nexty,
          }
        },
       DIR_RIGHT => {
          let nextx = if miner.x == WIDTH - 1 { 0 } else { miner.x + 1 };
          match array[nextx][miner.y] {
            '█' => {
              array[nextx][miner.y] = '▓';
              miner.dir = DIR_UP;
            },
            '▓' => {
              array[nextx][miner.y] = '▒';
              miner.dir = DIR_UP;
            },
            '▒' => {
              array[nextx][miner.y] = '░';
              miner.dir = DIR_UP;
            },
            '░' => {
              array[nextx][miner.y] = ICON_DIAMOND;
              miner.dir = DIR_UP;
            },
           ICON_ENERGY => {
              miner.energy = miner.energy + E_VALUE;
              array[nextx][miner.y] = ' ';
              miner.x = nextx;
            },
           ICON_DIAMOND => {
              miner.points = miner.points + 1;
              array[nextx][miner.y] = ' ';
              miner.x = nextx;
            },
            _ => miner.x = nextx,
          }
        },

        _ => {
          println!("unexpected dir is: {}", miner.dir);
          panic!("dir is enum");
        },
      }

      miner.energy = miner.energy - 1;
      iteration = iteration + 1;

      if options.visual {
        println!("update {} x: {} y: {} dir: {} energy: {} points: {}", iteration + 1, miner.x, miner.y, miner.dir, miner.energy, miner.points);
        print_table(&array, miner.x, miner.y, miner.dir);
        thread::sleep(delay);
      }
    }

    // TODO: use dedicated unseeded rng here, once we do.
    let prev_m_p = miner.multiplier_points;
    let prev_m_e = miner.multiplier_energy;
    let next_m_e = prev_m_e + 1;

    let post_points = miner.points as i32 * ((INIT_ENERGY as f64 * (next_m_e as f64 / 100.0)) as i32);
    println!("Final points: {} after {} iterations", miner.points, iteration);
    if miner.points > best_miner.points {
      println!("Found a better miner {} to {} points", best_miner.points, miner.points);
      best_miner = miner;
    }

    miner = Miner {
      x: WIDTH >> 1,
      y: HEIGHT >> 1,
      dir: DIR_UP,
      energy: (INIT_ENERGY as f64 * (next_m_e as f64 / 100.0)) as i32,
      points: 0,

      multiplier_points: prev_m_p - 1,
      multiplier_energy: next_m_e,
    };
  }
}

