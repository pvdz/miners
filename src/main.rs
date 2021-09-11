use std::{thread, time};
use std::env;
use std::fmt::Write;
use std::fmt;

use rand::prelude::*;
use rand_pcg::Pcg64;
use rand::distributions::{Distribution, Uniform};

const WIDTH: usize = 50;
const HEIGHT: usize = 50;
const INIT_BLOCKS_PER_ROW: i32 = WIDTH as i32 >> 1; // Half?

const E_COUNT: i32 = 50; // How many energy pickups do we spawn
const E_VALUE: i32 = 125; // Energy pickup bonus. 5%?
const INIT_ENERGY: i32 = 1000;

// TODO: this must be typeable :)
const DIR_UP   : i32 = 1;
const DIR_RIGHT: i32 = 2;
const DIR_DOWN : i32 = 3;
const DIR_LEFT : i32 = 4;

const WHAT_MINER: i32 = 0;
const WHAT_DRONE: i32 = 1;

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

const TITLE_EMPTINESS: &str = "Empty";
const TITLE_DRONE_LAUNCHER: &str = "Drone Launcher";
const TITLE_ENERGY_CELL: &str = "Energy Cell";

const ICON_BORDER_TL: char = '╔';
const ICON_BORDER_BL: char = '╚';
const ICON_BORDER_TR: char = '╗';
const ICON_BORDER_BR: char = '╝';
const ICON_BORDER_V: char = '║';
const ICON_BORDER_H: char = '═';
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

struct Drone {
  // Each drone has its own x, y, direction, and energy
  movable: Movable,

}

trait Slottable: fmt::Display {
  fn beforePaint(&mut self, minerMovable: &mut Movable, minerMeta: &mut MinerMeta, world: &mut World);
  fn paint(&self, world: &mut World);
  fn afterPaint(&mut self, minerMovable: &mut Movable, minerMeta: &mut MinerMeta, world: &mut World);
  fn title(&self) -> &str;
}

struct DroneLauncher {
  drone: Drone,
}

impl Slottable for DroneLauncher {
  fn beforePaint(&mut self, minerMovable: &mut Movable, minerMeta: &mut MinerMeta, world: &mut World) {
    if self.drone.movable.energy > 0 {
      move_movable(&mut self.drone.movable, minerMeta, world);
    }
  }

  fn paint(&self, world: &mut World) {
    if self.drone.movable.energy > 0 {
      world[self.drone.movable.x][self.drone.movable.y] = match self.drone.movable.dir {
        DIR_UP => ICON_DRONE_UP,
        DIR_DOWN => ICON_DRONE_DOWN,
        DIR_LEFT => ICON_DRONE_LEFT,
        DIR_RIGHT => ICON_DRONE_RIGHT,
        _ => {
          println!("unexpected dir: {:?}", self.drone.movable.dir);
          panic!("dir is enum");
        },
      }
    }
  }

  fn afterPaint(&mut self, minerMovable: &mut Movable, minerMeta: &mut MinerMeta, world: &mut World) {
    if self.drone.movable.energy <= 0 && minerMeta.drone_gen_cooldown == 0 {
      self.drone.movable.energy = 100;
      self.drone.movable.x = minerMovable.x;
      self.drone.movable.y = minerMovable.y;
      self.drone.movable.dir = if minerMovable.dir == DIR_UP { DIR_DOWN } else { DIR_UP };
      minerMeta.drone_gen_cooldown = 50;
      minerMovable.energy = minerMovable.energy - 100;
    }
  }

  fn title(&self) -> &str { return TITLE_DRONE_LAUNCHER; }
}

impl fmt::Display for DroneLauncher {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.drone.movable.energy <= 0 {
      write!(f, "Drone inactive {:>50}", ' ')
    } else {
      write!(f, "x: {}, y: {}, dir: {}, energy: {} {:>50}", self.drone.movable.x, self.drone.movable.y, self.drone.movable.dir, self.drone.movable.energy, ' ')
    }
  }
}

/**
 * An energy cell gives you an energy boost at a certain interval. It takes up n slots.
 */
struct EnergyCell {
  energy_bonus: i32,
  max_cooldown: i32,
  cooldown: i32,
}

impl Slottable for EnergyCell {
  fn beforePaint(&mut self, minerMovable: &mut Movable, minerMeta: &mut MinerMeta, world: &mut World) {
    self.cooldown = self.cooldown + 1;
    if self.cooldown >= self.max_cooldown {
      minerMovable.energy = minerMovable.energy + self.energy_bonus;
      if minerMovable.energy > minerMeta.max_energy {
        minerMovable.energy = minerMeta.max_energy;
      }
      self.cooldown = 0;
    }
  }

  fn paint(&self, world: &mut World) {}

  fn afterPaint(&mut self, minerMovable: &mut Movable, minerMeta: &mut MinerMeta, world: &mut World) {}

  fn title(&self) -> &str { return TITLE_ENERGY_CELL; }
}

impl fmt::Display for EnergyCell {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

    write!(
      f,
      "{}{} {}%",
      std::iter::repeat('|').take(((self.cooldown as f32 / self.max_cooldown as f32) * 10.0) as usize).collect::<String>(),
      std::iter::repeat('-').take(10 - ((self.cooldown as f64 / self.max_cooldown as f64) * 10.0) as usize).collect::<String>(),
      ((self.cooldown as f64 / self.max_cooldown as f64) * 100.0) as i32
    )

    // write!(f, "|||||||||||||||||| {} %", self.energy_bonus)
  }
}

struct Emptiness {
}

impl Slottable for Emptiness {
  fn beforePaint(&mut self, minerMovable: &mut Movable, minerMeta: &mut MinerMeta, world: &mut World) {
    // Do nothing
  }

  fn paint(&self, world: &mut World) {}

  fn afterPaint(&mut self, minerMovable: &mut Movable, minerMeta: &mut MinerMeta, world: &mut World) {}

  fn title(&self) -> &str { return TITLE_EMPTINESS; }
}

impl fmt::Display for Emptiness {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "")
  }
}

struct Movable {
  what: i32,
  x: usize,
  y: usize,
  dir: i32,
  energy: i32,
}

struct MinerMeta {
  max_energy: i32,
  points: i32,
  //  item:
  //  cooldown: i32, // Iterations before item can be used again
  drone_gen_cooldown: i32, // Generate a new drone every this many ticks

  // Increase energy cost per step per boredom level
  // The miner gets bored if it hasn't seen anything in a while. Prevents endless loops
  boredom_level: i32, // Current level of boredom
  boredom_steps: i32, // Current number of boring steps
  boredom_rate: i32, // Number of boring steps after which the boredom level goes up

  // These multipliers are in whole percentages
  multiplier_energy_start: i32,
  multiplier_points: i32,
  block_bump_cost: i32,
  multiplier_energy_pickup: i32,
  //  multiplier_cooldown: i32,
}

struct Miner {
  movable: Movable,
  meta: MinerMeta,
  // Whenever a drone is generated it will take a chunk of energy from the miner
  slots: [Box<Slottable>; 32],
}

fn serialize_world(world: &World, miner: &Miner) -> String {
  // We assume a 150x80 terminal screen space (half my ultra wide)
  // We draw every cell twice because the terminal cells have a 1:2 w:h ratio

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
  for slot in miner.slots.iter() {
    slot.paint(&mut new_world);
  }

  let mut buf : String = ICON_BORDER_TL.to_string();

  write!(buf, "{}", std::iter::repeat(ICON_BORDER_H).take(WIDTH*2).collect::<String>()).unwrap(); // cache this :shrug:
  write!(buf, "{} {: >100}", ICON_BORDER_TR, ' ').unwrap();
  write!(buf, "\n").unwrap();

  for y in 0..HEIGHT {
    write!(buf, "{}", ICON_BORDER_V).unwrap();
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
    write!(buf, "{}", ICON_BORDER_V).unwrap();

    const header: usize = 11;
    match if y < header { y } else { y - header + 100 } {
      // Miner meta information
      0  => write!(buf, "  Miner:  {}  x  {} {: >60}\n", miner.movable.x, miner.movable.y, ' ').unwrap(),
      1  => write!(buf, "  Energy: {}{} ({}%) {} / {} {: >60}\n",
        std::iter::repeat('|').take(((miner.movable.energy as f32 / miner.meta.max_energy as f32) * 20.0) as usize).collect::<String>(),
        std::iter::repeat('-').take(20 - ((miner.movable.energy as f64 / miner.meta.max_energy as f64) * 20.0) as usize).collect::<String>(),
        ((miner.movable.energy as f64 / miner.meta.max_energy as f64) * 100.0) as i32,
        miner.movable.energy,
        miner.meta.max_energy,
        ' '
      ).unwrap(),
      2  => write!(buf, "  Boredom: Level: {} Current step: {} Rate: {} {: >60}\n", miner.meta.boredom_level, miner.meta.boredom_steps, miner.meta.boredom_rate, ' ').unwrap(),
      3  => write!(buf, "  Points: {} {: >60}\n", miner.meta.points, ' ').unwrap(),
      4  => write!(buf, "  Block bump cost: {} {: >60}\n", miner.meta.block_bump_cost, ' ').unwrap(),

      6  => write!(buf, "  GA config: {: >60}\n", ' ').unwrap(),
      7  => write!(buf, "  Multiplier energy:        {} {: >60}\n", miner.meta.multiplier_energy_start, ' ').unwrap(),
      8  => write!(buf, "  Multiplier points:        {} {: >60}\n", miner.meta.multiplier_points, ' ').unwrap(),
      9  => write!(buf, "  Multiplier energy pickup: {} {: >60}\n", miner.meta.multiplier_energy_pickup, ' ').unwrap(),

      // The slots
      100  => write!(buf, "  Slots: {: >60}\n", ' ').unwrap(),
      101  => write!(buf, "    - {: <20} {}\n", miner.slots[0].title(), miner.slots[0]).unwrap(),
      102  => write!(buf, "    - {: <20} {}\n", miner.slots[1].title(), miner.slots[1]).unwrap(),
      103  => write!(buf, "    - {: <20} {}\n", miner.slots[2].title(), miner.slots[2]).unwrap(),
      104  => write!(buf, "    - {: <20} {}\n", miner.slots[3].title(), miner.slots[3]).unwrap(),
      105  => write!(buf, "    - {: <20} {}\n", miner.slots[4].title(), miner.slots[4]).unwrap(),
      106  => write!(buf, "    - {: <20} {}\n", miner.slots[5].title(), miner.slots[5]).unwrap(),
      107  => write!(buf, "    - {: <20} {}\n", miner.slots[6].title(), miner.slots[6]).unwrap(),
      108  => write!(buf, "    - {: <20} {}\n", miner.slots[7].title(), miner.slots[7]).unwrap(),
      109  => write!(buf, "    - {: <20} {}\n", miner.slots[8].title(), miner.slots[8]).unwrap(),
      110  => write!(buf, "    - {: <20} {}\n", miner.slots[9].title(), miner.slots[9]).unwrap(),
      111  => write!(buf, "    - {: <20} {}\n", miner.slots[10].title(), miner.slots[10]).unwrap(),
      112  => write!(buf, "    - {: <20} {}\n", miner.slots[11].title(), miner.slots[11]).unwrap(),
      113  => write!(buf, "    - {: <20} {}\n", miner.slots[12].title(), miner.slots[12]).unwrap(),
      114  => write!(buf, "    - {: <20} {}\n", miner.slots[13].title(), miner.slots[13]).unwrap(),
      115  => write!(buf, "    - {: <20} {}\n", miner.slots[14].title(), miner.slots[14]).unwrap(),
      116  => write!(buf, "    - {: <20} {}\n", miner.slots[15].title(), miner.slots[15]).unwrap(),
      117  => write!(buf, "    - {: <20} {}\n", miner.slots[16].title(), miner.slots[16]).unwrap(),
      118  => write!(buf, "    - {: <20} {}\n", miner.slots[17].title(), miner.slots[17]).unwrap(),
      119  => write!(buf, "    - {: <20} {}\n", miner.slots[18].title(), miner.slots[18]).unwrap(),
      120  => write!(buf, "    - {: <20} {}\n", miner.slots[19].title(), miner.slots[19]).unwrap(),
      121  => write!(buf, "    - {: <20} {}\n", miner.slots[20].title(), miner.slots[20]).unwrap(),
      122  => write!(buf, "    - {: <20} {}\n", miner.slots[21].title(), miner.slots[21]).unwrap(),
      123  => write!(buf, "    - {: <20} {}\n", miner.slots[22].title(), miner.slots[22]).unwrap(),
      124  => write!(buf, "    - {: <20} {}\n", miner.slots[23].title(), miner.slots[23]).unwrap(),
      125  => write!(buf, "    - {: <20} {}\n", miner.slots[24].title(), miner.slots[24]).unwrap(),
      126  => write!(buf, "    - {: <20} {}\n", miner.slots[25].title(), miner.slots[25]).unwrap(),
      127  => write!(buf, "    - {: <20} {}\n", miner.slots[26].title(), miner.slots[26]).unwrap(),
      128  => write!(buf, "    - {: <20} {}\n", miner.slots[27].title(), miner.slots[27]).unwrap(),
      129  => write!(buf, "    - {: <20} {}\n", miner.slots[28].title(), miner.slots[28]).unwrap(),
      130  => write!(buf, "    - {: <20} {}\n", miner.slots[29].title(), miner.slots[29]).unwrap(),
      131  => write!(buf, "    - {: <20} {}\n", miner.slots[30].title(), miner.slots[30]).unwrap(),
      132  => write!(buf, "    - {: <20} {}\n", miner.slots[31].title(), miner.slots[31]).unwrap(),

      _ => {
        if y <= header {
          write!(buf, " {: >60}", ' ').unwrap();
        }
        write!(buf, "\n").unwrap()
      }
    }
  }

  // std::iter::repeat("X").take(10).collect::<String>()

  write!(buf, "{}", ICON_BORDER_BL).unwrap();
  write!(buf, "{}", std::iter::repeat(ICON_BORDER_H).take(WIDTH*2).collect::<String>()).unwrap();
  write!(buf, "{}", ICON_BORDER_BR).unwrap();

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
  let mut was_boring = false; // Did we just move forward? No blocks, no pickups?
  match world[nextx][nexty] {
    '█' => {
      world[nextx][nexty] = '▓';
      movable.dir = nextdir;
      movable.energy = movable.energy - meta.block_bump_cost;
      meta.boredom_steps = meta.boredom_steps + 1;
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
      if movable.energy > meta.max_energy {
        movable.energy = meta.max_energy;
      }
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
      was_boring = true;
      // Prevent endless loops by making it increasingly more difficult to make consecutive moves that where nothing happens
      if movable.what == WHAT_MINER {
        movable.energy = movable.energy - meta.boredom_level;
      }
    },
  }

  if movable.what == WHAT_MINER {
    if was_boring {
      meta.boredom_steps = meta.boredom_steps + 1;
      if meta.boredom_steps >= meta.boredom_rate {
        meta.boredom_steps = 0;
        meta.boredom_level = meta.boredom_level + 1;
        meta.boredom_level = meta.boredom_level + 1;
      }
    } else {
      meta.boredom_steps = 0;
      meta.boredom_level = 0;
    }
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

  let mut best_miner = Miner {
    movable: Movable {
      what: WHAT_MINER,
      x: WIDTH >> 1,
      y: HEIGHT >> 1,
      dir: DIR_UP,
      energy: 0,
    },
    meta: MinerMeta {
      points: 0,
      max_energy: INIT_ENERGY,
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
      Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }),
      Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }),
      Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }),
      Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }),
      Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }),
      Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }),
      Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }),
      Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }), Box::new(Emptiness { }),
    ],
  };

  let golden_map: World = generate_world(&options);

  // Print the initial world at least once
  let table_str: String = serialize_world(&golden_map, &best_miner);
  println!("{}", table_str);

  loop {

    let start_energy = (best_miner.meta.max_energy as f64 * (1.0 + multiplier_energy_start as f64 / 100.0)) as i32;
    let mut miner: Miner = Miner {
      movable: Movable {
        what: WHAT_MINER,
        x: WIDTH >> 1,
        y: HEIGHT >> 1,
        dir: DIR_UP,
        energy: start_energy,
      },
      meta: MinerMeta {
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
        Box::new(DroneLauncher { drone: Drone { movable: Movable { what: WHAT_DRONE, x: 0, y: 0, dir: DIR_DOWN, energy: 0 } } }),
        Box::new(DroneLauncher { drone: Drone { movable: Movable { what: WHAT_DRONE, x: 0, y: 0, dir: DIR_DOWN, energy: 0 } } }),
        Box::new(DroneLauncher { drone: Drone { movable: Movable { what: WHAT_DRONE, x: 0, y: 0, dir: DIR_DOWN, energy: 0 } } }),
        Box::new(DroneLauncher { drone: Drone { movable: Movable { what: WHAT_DRONE, x: 0, y: 0, dir: DIR_DOWN, energy: 0 } } }),
        Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
        Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
        Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
        Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
      ],
    };

    // Recreate the rng fresh for every new Miner
    // let mut rng = Pcg64::seed_from_u64(options.seed);
    // let mut drone_rng = Pcg64::seed_from_u64(options.seed);

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
      for i in 0..miner.slots.len() {
      // for slot in miner.slots.iter_mut() {
        miner.slots[i].beforePaint(&mut miner.movable, &mut miner.meta, &mut world);
      }

      miner.movable.energy = miner.movable.energy - 1;
      iteration = iteration + 1;

      if options.visual {
        let table_str: String = serialize_world(&world, &miner);
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
        slot.afterPaint(&mut miner.movable, &mut miner.meta, &mut world);
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
  }
}

