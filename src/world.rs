// use std::fmt::Write;
use std::collections::VecDeque;

use rand::prelude::*;
use rand_pcg::Pcg64;
use rand::distributions::{Distribution, /*Uniform,*/ Standard};
use crate::cell_contents::Cell::Empty;

// use super::miner::*;
use super::values::*;
use super::icons::*;
use super::options::*;
use super::helix::*;
use super::biome::*;
use super::cell_contents::*;

// The world is procedurally generated and has no theoretical bounds.
// The map retained in memory is only has big as has been visited. Any unvisited cell (or well, any
// _unchanged_ cell rather) should use the default value according to the current seed for the
// procedural generation of it.
// The world needs to be extendable on both sides efficiently but also need efficient direct access.
// Transposing the 2d world on a 1d array is therefor infeasible because extending one axis means
// moving potentially many bytes. A simple vec has the same problem in one direction.
// So we use a vec deque which supports exactly this.

pub type Tile = (
  Cell, // Immovable type of this tile
  u32, // Movable object on this tile
);
pub type Grid = VecDeque<VecDeque<Tile>>;


#[derive(Debug)]
pub struct World {
  // Rectangle of the known world
  pub min_x: i32,
  pub min_y: i32,
  pub max_x: i32,
  pub max_y: i32,
  // Inanimate objects like blocks and pickups
  // The first vec is vertical which should make printing a simple loop :shrug:
  // Every vec should have `abs(min)+max+1` tiles. Assuming only `min` can be negative
  // should be safe because the world always starts at 0,0 and does not shrink.
  pub tiles: Grid,
}

pub fn generate_cell(options: &Options, x: i32, y: i32) -> Cell {
  // println!("  generate_cell({}, {})", x, y);
  // Take the world seed and add the x as a <<32 value and y as is to the seed
  // If either x or y are negative they should subtract that value from the world seed
  // If the result is negative, it should wrap around.
  let nx: i64 = if x < 0 { -(-(x as i64) << 32) } else { (x as i64) << 32 };
  let cell_seed: u64 = ((options.seed as i64) + nx + (y as i64)) as u64;
  let mut cell_rng = Pcg64::seed_from_u64(cell_seed);

  // I guess start with the rarest stuff first, move to the common stuff, end with empty

  // 2% of the cells should contain an energy container (arbitrary)
  if (cell_rng.sample::<f32, Standard>(Standard)) < 0.2f32 {
    return Cell::Energy;
  }

  // Roughly half the cells should be filled with walls
  if cell_rng.sample::<f32, Standard>(Standard) < 0.5f32 {
    // Roughly speaking, 10% is 3, 30% is 2, 60% is 1?
    let kind: f32 = cell_rng.sample(Standard);

    if kind < 0.1 {
      return Cell::Wall3;
    }

    if kind < 0.4 {
      return Cell::Wall2;
    }

    return Cell::Wall1;
  }

  return Cell::Empty;
}

pub fn world_width(world: &World) -> i32 {
  // The world starts at 0,0 and does not shrink so only min_x might be negative
  return world.min_x.abs() + world.max_x + 1;
}

pub fn world_height(world: &World) -> i32 {
  // The world starts at 0,0 and does not shrink so only min_y might be negative
  return world.min_y.abs() + world.max_y + 1;
}

pub fn generate_world(options: &Options) -> World {
  // A world is procedurally generated based on an algorithm. Cells are not prerendered but
  // rather get generated on demand, when they are relevant to be queried (for example because
  // they are visited or because they are painted). A cell is actually generated in the world
  // model once it requires any kind of active change, whether it be a changed tile background
  // or an item on it needs representing.
  //
  // The initial state of a tile is formed by applying a series of chances, in serial, based
  // on the x,y coordinates of the tile and a fixed world-bound rng seed. The result should
  // be idempotent for any pair of coordinates and seed, so if you give it the same coordinate
  // and same seed then the outcome should always be the same no matter how often or when it is
  // requested from the algorithm. Potentially there could be another dimension like time here
  // (like tile state changing as time flows) or like neighbor state.
  //
  // Basically it should use the world state as an offset and the coordinate as a unique forward
  // seed and then generate the series of odds from that. This would give us rng (world seed),
  // consistency (coord as seed) and still an unpredictable odds (consistent procedure).

  // Generate the initial state so we can paint it.
  // Square spanning 25 blocks in all four directions from origin.

  let mut ygrid: VecDeque<VecDeque<Tile>> = VecDeque::new();
  let mut xgrid: VecDeque<Tile> = VecDeque::new();
  xgrid.push_back((Cell::Empty, 0u32));
  ygrid.push_back(xgrid);

  let mut world = World {
    min_x: 0,
    min_y: 0,
    max_x: 0,
    max_y: 0,
    tiles: ygrid,
  };

  let min_x = -3;
  let min_y = -3;
  let max_x = 3;
  let max_y = 3;

  // Explicitly spawn the 51x51 grid (25 cells in each direction plus the 0x0 cell)
  ensure_cell_in_world(&mut world, options, min_x, min_y);
  ensure_cell_in_world(&mut world, options, max_x, max_y);

  // println!("World now: {} {}", world.tiles.len(), world.tiles[0].len());

  return world;
}

fn bound(x: i32, y: i32, min_x: i32, max_x: i32, min_y: i32, max_y: i32) -> bool {
  return x >= min_x && x < max_x && y >= min_y && y < max_y;
}

pub fn coord_to_index(x: i32, y: i32, world: &World) -> (i32, i32) {
  return (world.min_x.abs() - x, world.min_y.abs() - y);
}

pub fn serialize_world(world: &World, biomes: &Vec<Biome>, best: (Helix, i32), options: &Options) -> String {
  // We assume a 150x80 terminal screen space (half my ultra wide)
  // We draw every cell twice because the terminal cells have a 1:2 w:h ratio

  // Start by painting the world. Give it a border too (annoying with calculations but worth it)


  return "fixme".to_owned();


  // new stuff:


  // let view_margin = (1, 1, 1, 1);
  // assert(view_margin.0 >= 0);
  // assert(view_margin.1 >= 0);
  // assert(view_margin.2 >= 0);
  // assert(view_margin.3 >= 0);
  //
  // // World coordinates for the viewport:
  // let viewport_offset_x: i32 = -25; // (TODO: this should make sure the character location is included in the viewport)
  // let viewport_offset_y: i32 = -25; // Same ^
  // let viewport_size_w: i32 = 50;
  // let viewport_size_h: i32 = 50;
  //
  // // Create strings that form lines of margins and borders
  // let margin_top: Vec<char> = vec!().chain(std::iter::repeat(' ').take(view_margin.3 + (view_margin.0 * WIDTH * 2) + view_margin.1));
  // let border_top: Vec<char> = vec!().chain(std::iter::repeat(' ').take(view_margin.3)).chain(vec!(ICON_BORDER_TL)).chain(std::iter::repeat(ICON_BORDER_H).take(WIDTH*2)).chain(vec!(ICON_BORDER_TR)).chain(std::iter::repeat(' ').take(view_margin.1));
  // let border_bottom: Vec<char> = vec!().chain(std::iter::repeat(' ').take(view_margin.3)).chain(vec!(ICON_BORDER_BL)).chain(std::iter::repeat(ICON_BORDER_H).take(WIDTH*2)).chain(vec!(ICON_BORDER_BR)).chain(std::iter::repeat(' ').take(view_margin.3));
  // let margin_bottom: Vec<char> = vec!().chain(std::iter::repeat(' ').take(view_margin.3 + (view_margin.2 * WIDTH * 2) + view_margin.1));
  //
  // let mut view: Vec<Vec<char>> = vec!();
  // if view_margin.0 > 0 {
  //     view.push(margin_top);
  // }
  // view.push(border_top);
  //
  // // First generate the map view with the actual tiles
  //
  // // To get to the correct vec index for given viewport offset;
  // //    world.min.abs() + offset
  // // Example:
  // //    initially min is -25 and the camera is at -25, so -25.abs() + -25 = -25
  //
  // for line in world.tiles.slice(world.min_y.abs() + viewport_offset_y, viewport_size_h) {
  //     let mut view_line: Vec<char> = vec!();
  //
  //     if view_margin.3 > 0 {
  //         view_line.push(' ');
  //     }
  //     view_line.push(ICON_BORDER_V);
  //
  //     for tile in line.slice(world.min_x.abs() + viewport_offset_x, viewport_size_w) {
  //         match tile.0 {
  //             | Cell::Empty => {
  //                 view_line.push(' ');
  //                 view_line.push(' ');
  //             },
  //             | Cell::Energy => {
  //                 view_line.push(ICON_ENERGY);
  //             },
  //             | Cell::Wall1
  //             | Cell::Wall2
  //             | Cell::Wall3 => {
  //                 view_line.push(ICON_BLOCK_100);
  //                 view_line.push(ICON_BLOCK_100);
  //             },
  //             | _tile => panic!("Unknown tile type: {:?}", _tile),
  //         }
  //     }
  //
  //     view_line.push(ICON_BORDER_V);
  //     if view_margin.1 > 0 {
  //         view_line.push(' ');
  //     }
  //
  //     view.push(view_line);
  // }
  // view.push(border_bottom);
  // if view_margin.2 > 0 {
  //     view.push(margin_bottom);
  // }
  //
  // // Now we have a predictable view. Print the movable characters on top of it.
  //
  // for biome in biomes.iter() {
  //     // if the viewport offsets at <-25, -25> and the miner is at <0,0> then paint it at <25,25>
  //     // <-25,-25> and <1,1> then <26,26>
  //     // <0,0> and <10,20> then <10,20>
  //     // <1,2> and <9,18> then <10,20>
  //
  //     let rx: i32 = biome.miner.movable.x - viewport_offset_x;
  //     let ry: i32 = biome.miner.movable.y - viewport_offset_y;
  //     if bound(rx, ry, 0, 0, viewport_size_w, viewport_size_h) {
  //         // Miner is within range. Paint it at the relative coords <rx,ry>
  //         view[view_margin.0 + 1 + ry][view_margin.3 + 1 + rx] = ICON_GHOST;
  //     }
  // }
  //
  // // Print the main miner. This way we know it will be visible and not covered by a ghost.
  // {
  //     let rx: i32 = biomes[0].miner.movable.x - viewport_offset_x;
  //     let ry: i32 = biomes[0].miner.movable.y - viewport_offset_y;
  //     if bound(rx, ry, 0, 0, viewport_size_w, viewport_size_h) {
  //         // Miner is within range. Paint it at the relative coords <rx,ry>
  //         view[view_margin.0 + 1 + ry][view_margin.3 + 1 + rx] = match biomes[0].miner.movable.dir {
  //             DIR_UP => ICON_MINER_UP,
  //             DIR_DOWN => ICON_MINER_DOWN,
  //             DIR_LEFT => ICON_MINER_LEFT,
  //             DIR_RIGHT => ICON_MINER_RIGHT,
  //             _dir => panic!("unexpected dir: {:?}", _dir),
  //         };
  //     }
  // }
  //
  // // The map is finished. Append the "UI" to the right of the map by pushing the characters there.
  // // Each vec should already have been padded with a margin at this point.
  // // Definitely a hack but why not :)
  //
  // view[view_margin.0 as usize + 1].chain("UI stuff goes here".chars());
  //
  // // Join the characters of each line into a string then join those lines with a newline
  //
  // let mut out: Iter<char> = vec!().iter();
  // for y in 0..view.len() {
  //     let v = &view[y];
  //     out.chain(v).chain(vec!("\n"));
  // }
  //
  // return out.into_iter().collect();


  // old stuff:


  // for y in 0..HEIGHT {
  //     write!(buf, "{}", ICON_BORDER_V).unwrap_or_else(|err| panic!("{:?}", err));
  //     for x in 0..WIDTH {
  //         let c: char = painting[x][y];
  //         match c {
  //             | ICON_ENERGY
  //             => write!(buf, "\x1b[33;1m{}\x1b[0m", c),
  //             | ICON_DIAMOND
  //             => match world.values[x][y] {
  //                 '1' => write!(buf, "\x1b[;1;1m{0}\x1b[0m", c),
  //                 '2' => write!(buf, "\x1b[;1;1m\x1b[32m{0}\x1b[0m", c),
  //                 '3' => write!(buf, "\x1b[;1;1m\x1b[34m{0}\x1b[0m", c),
  //                 _ => panic!("Unexpected world value: {}", c),
  //             },
  //             | ICON_TURN_RIGHT
  //             | ICON_INDEX_UP
  //             | ICON_INDEX_RIGHT
  //             | ICON_INDEX_LEFT
  //             | ICON_INDEX_DOWN
  //             | ICON_GHOST
  //             => write!(buf, "{}", c),
  //
  //             | ICON_MINER_UP
  //             | ICON_MINER_RIGHT
  //             | ICON_MINER_DOWN
  //             | ICON_MINER_LEFT
  //             => write!(buf, "\x1b[;1;1m\x1b[31m{} \x1b[0m", c),
  //
  //             | ICON_BLOCK_100
  //             | ICON_BLOCK_75
  //             | ICON_BLOCK_50
  //             | ICON_BLOCK_25
  //             => match world.values[x][y] {
  //                 '1' => write!(buf, "{0}{0}\x1b[0m", c),
  //                 '2' => write!(buf, "\x1b[32m{0}{0}\x1b[0m", c),
  //                 '3' => write!(buf, "\x1b[34m{0}{0}\x1b[0m", c),
  //                 _ => write!(buf, "\x1b[34m{0}{0}\x1b[0m", c),
  //             },
  //
  //             v => write!(buf, "{0}{0}", v),
  //         }.unwrap_or_else(|err| panic!("{:?}", err));
  //     }
  //     write!(buf, "{}", ICON_BORDER_V).unwrap_or_else(|err| panic!("{:?}", err));
  //
  //     const HEADER: usize = 13;
  //     match if y < HEADER { y } else { y - HEADER + 100 } {
  //         // Miner meta information
  //          0  => write!(buf, "  Miner:  {: <2}  x  {: <2} {: >60}\n", miner.movable.x, miner.movable.y, ' ').unwrap(),
  //          1  => write!(buf, "  Energy: {}{} ({: >3}%) {} / {} {: >60}\n",
  //                      std::iter::repeat('|').take(((miner.movable.energy as f32 / miner.meta.max_energy as f32) * 20.0) as usize).collect::<String>(),
  //                      std::iter::repeat('-').take(20 - ((miner.movable.energy as f64 / miner.meta.max_energy as f64) * 20.0) as usize).collect::<String>(),
  //                      ((miner.movable.energy as f64 / miner.meta.max_energy as f64) * 100.0) as i32,
  //                      miner.movable.energy,
  //                      miner.meta.max_energy,
  //                      ' '
  //          ).unwrap(),
  //          2  => write!(buf, "  Boredom: Rate: {: <2} per level. Level: {: <3}. Cost per step: {} {: >60}\n", miner.meta.boredom_rate as i32, miner.meta.boredom_level, (miner.meta.boredom_rate * miner.meta.boredom_level as f32) as i32, ' ').unwrap(),
  //          3  => write!(buf, "  Points: {} {: >60}\n", miner.meta.points, ' ').unwrap(),
  //          4  => write!(buf, "  Block bump cost: {} {: >60}\n", miner.meta.block_bump_cost, ' ').unwrap(),
  //
  //          6  => write!(buf, "  Helix:                         Current:                Best:{: >60}\n", ' ').unwrap(),
  //          7  => write!(buf, "  Max energy:               {: >20} {: >20} {: >60}\n", miner.helix.multiplier_energy_start, best.0.multiplier_energy_start, ' ').unwrap(),
  //          8  => write!(buf, "  Multiplier points:        {: >20} {: >20} {: >60}\n", miner.helix.multiplier_points, best.0.multiplier_points, ' ').unwrap(),
  //          9  => write!(buf, "  Multiplier energy pickup: {: >20} {: >20} {: >60}\n", miner.meta.multiplier_energy_pickup, 0.0, ' ').unwrap(),
  //         10  => write!(buf, "  Block bump cost:          {: >20} {: >20} {: >60}\n", miner.helix.block_bump_cost, best.0.block_bump_cost, ' ').unwrap(),
  //         11  => write!(buf, "  Drone gen cooldown:       {: >20} {: >20} {: >60}\n", miner.helix.drone_gen_cooldown, best.0.drone_gen_cooldown, ' ').unwrap(),
  //
  //         // The slots
  //         100  => write!(buf, "  Slots: {: >120}\n", ' ').unwrap(),
  //         101  => write!(buf, "    - {: <20} {}\n", miner.slots[0].title(), miner.slots[0]).unwrap(),
  //         102  => write!(buf, "    - {: <20} {}\n", miner.slots[1].title(), miner.slots[1]).unwrap(),
  //         103  => write!(buf, "    - {: <20} {}\n", miner.slots[2].title(), miner.slots[2]).unwrap(),
  //         104  => write!(buf, "    - {: <20} {}\n", miner.slots[3].title(), miner.slots[3]).unwrap(),
  //         105  => write!(buf, "    - {: <20} {}\n", miner.slots[4].title(), miner.slots[4]).unwrap(),
  //         106  => write!(buf, "    - {: <20} {}\n", miner.slots[5].title(), miner.slots[5]).unwrap(),
  //         107  => write!(buf, "    - {: <20} {}\n", miner.slots[6].title(), miner.slots[6]).unwrap(),
  //         108  => write!(buf, "    - {: <20} {}\n", miner.slots[7].title(), miner.slots[7]).unwrap(),
  //         109  => write!(buf, "    - {: <20} {}\n", miner.slots[8].title(), miner.slots[8]).unwrap(),
  //         110  => write!(buf, "    - {: <20} {}\n", miner.slots[9].title(), miner.slots[9]).unwrap(),
  //         111  => write!(buf, "    - {: <20} {}\n", miner.slots[10].title(), miner.slots[10]).unwrap(),
  //         112  => write!(buf, "    - {: <20} {}\n", miner.slots[11].title(), miner.slots[11]).unwrap(),
  //         113  => write!(buf, "    - {: <20} {}\n", miner.slots[12].title(), miner.slots[12]).unwrap(),
  //         114  => write!(buf, "    - {: <20} {}\n", miner.slots[13].title(), miner.slots[13]).unwrap(),
  //         115  => write!(buf, "    - {: <20} {}\n", miner.slots[14].title(), miner.slots[14]).unwrap(),
  //         116  => write!(buf, "    - {: <20} {}\n", miner.slots[15].title(), miner.slots[15]).unwrap(),
  //         117  => write!(buf, "    - {: <20} {}\n", miner.slots[16].title(), miner.slots[16]).unwrap(),
  //         118  => write!(buf, "    - {: <20} {}\n", miner.slots[17].title(), miner.slots[17]).unwrap(),
  //         119  => write!(buf, "    - {: <20} {}\n", miner.slots[18].title(), miner.slots[18]).unwrap(),
  //         120  => write!(buf, "    - {: <20} {}\n", miner.slots[19].title(), miner.slots[19]).unwrap(),
  //         121  => write!(buf, "    - {: <20} {}\n", miner.slots[20].title(), miner.slots[20]).unwrap(),
  //         122  => write!(buf, "    - {: <20} {}\n", miner.slots[21].title(), miner.slots[21]).unwrap(),
  //         123  => write!(buf, "    - {: <20} {}\n", miner.slots[22].title(), miner.slots[22]).unwrap(),
  //         124  => write!(buf, "    - {: <20} {}\n", miner.slots[23].title(), miner.slots[23]).unwrap(),
  //         125  => write!(buf, "    - {: <20} {}\n", miner.slots[24].title(), miner.slots[24]).unwrap(),
  //         126  => write!(buf, "    - {: <20} {}\n", miner.slots[25].title(), miner.slots[25]).unwrap(),
  //         127  => write!(buf, "    - {: <20} {}\n", miner.slots[26].title(), miner.slots[26]).unwrap(),
  //         128  => write!(buf, "    - {: <20} {}\n", miner.slots[27].title(), miner.slots[27]).unwrap(),
  //         129  => write!(buf, "    - {: <20} {}\n", miner.slots[28].title(), miner.slots[28]).unwrap(),
  //         130  => write!(buf, "    - {: <20} {}\n", miner.slots[29].title(), miner.slots[29]).unwrap(),
  //         131  => write!(buf, "    - {: <20} {}\n", miner.slots[30].title(), miner.slots[30]).unwrap(),
  //         132  => write!(buf, "    - {: <20} {}\n", miner.slots[31].title(), miner.slots[31]).unwrap(),
  //
  //
  //         133  => write!(buf, "{: <100}\n", ' ').unwrap(),
  //         134  => write!(buf, "{: <100}\n", ' ').unwrap(),
  //         135  => {
  //             let mut he : String = "".to_string();
  //             helix_to_string(&mut he, &best.0);
  //             write!(buf, "    Best {}{: <40}\n", he, ' ').unwrap();
  //         },
  //         136  => write!(buf, "    Seed: {} Speed: {} Gene rate: {} Slot rate: {} (+⏎/-⏎ to change speed, v⏎ to toggle visual mode) {: <100}\n", options.seed, options.speed, options.mutation_rate_genes, options.mutation_rate_slots, ' ').unwrap(),
  //
  //
  //         _ => {
  //             if y <= HEADER {
  //                 write!(buf, " {: >120}", ' ').unwrap();
  //             }
  //             write!(buf, "\n").unwrap()
  //         }
  //     }
  // }

  // std::iter::repeat("X").take(10).collect::<String>()
  //
  // write!(buf, "{}", ICON_BORDER_BL).unwrap();
  // write!(buf, "{}", std::iter::repeat(ICON_BORDER_H).take(WIDTH*2).collect::<String>()).unwrap();
  // write!(buf, "{}", ICON_BORDER_BR).unwrap();
  //
  // buf
}

pub fn ensure_cell_in_world(world: &mut World, options: &Options, x: i32, y: i32) {
  // Expansion occurs by pre/appending a cell to all rows/cols of the axis that expands

  // Note: the world bounds are inclusive (so they exist if x>=world.min_x and x<=world.max_x)

  // println!("ensure_cell_in_world: world box: {}X{} ~ {}X{}, target coord: {}X{}", world.min_x, world.min_y, world.max_x, world.max_y, x, y);

  if x < world.min_x {
    // OOB. We have to prepend a cell to each row
    let world_height = (world.min_y.abs() + world.max_y + 1);
    let to_prepend = x.abs() - world.min_x.abs();
    // println!("Expanding -x (left), prepending {} cols to {} rows", to_prepend, world_height);

    for j in 0..world_height {
      let mut row = &mut world.tiles[j as usize];
      for i in 0..to_prepend {
        let gx = (world.min_x - i) - 1;
        let gy = (world.min_y - (world_height - j)) + 1;
        let cell = generate_cell(&options, gx, gy);
        row.push_front((cell, 0));
      }
    }
    world.min_x = world.min_x - to_prepend;
  } else if x > world.max_x {
    // OOB. We have to append a cell to each row
    let world_height = world.min_y.abs() + world.max_y + 1;
    let to_append = x - world.max_x;
    // println!("Expanding +x (right), appending {} cols to {} rows", to_append, world_height);

    for j in 0..world_height {
      let mut row = &mut world.tiles[j as usize];
      for i in 0..to_append {
        let gx = (world.min_x - i) - 1;
        let gy = (world.min_y - (world_height - j)) + 1;
        let cell = generate_cell(&options, gx, gy);
        row.push_front((cell, 0));
      }
    }
    world.max_x = world.max_x + to_append;
  }

  if y < world.min_y {
    // OOB. Add new row at the start. Fill it up with `abs(min_x)+max_x+1` cells. y must be negative
    let world_width = world.min_x.abs() + world.max_x + 1;
    let to_prepend = y.abs() - world.min_y.abs();
    // println!("Expanding -y (up), creating {} rows with {} cells", to_prepend, world_width);

    // Create n rows and fill them up, then prepend them to the world tiles
    for j in 0..to_prepend {
      let mut new_row: VecDeque<Tile> = VecDeque::new();
      for i in 0..world_width {
        let gx = world.min_x + i;
        let gy = (world.min_y - j) - 1;
        let cell = generate_cell(&options, gx, gy);
        new_row.push_back((cell, 0));
      }
      world.tiles.push_front(new_row);
    }

    world.min_y = world.min_y - to_prepend;
  } else if y > world.max_y {
    // OOB. Add new row at the end. Fill it up with `abs(min_x)+max_x+1` cells.
    let world_width = world.min_x.abs() + world.max_x + 1;
    let to_append = y - world.max_y;
    // println!("Expanding +y (down), creating {} rows with {} cells {} {}", to_append, world_width, world.min_x.abs(), world.max_x);

    // Create n rows and fill them up, then append them to the world tiles
    for j in 0..to_append {
      let mut new_row: VecDeque<Tile> = VecDeque::new();
      for i in 0..world_width {
        let gx = world.min_x + i;
        let gy = (world.max_y + j);
        let cell = generate_cell(&options, gx, gy);
        new_row.push_back((cell, 0));
      }
      world.tiles.push_back(new_row);
    }

    world.max_y = world.max_y + to_append;
  }
}

pub fn create_tile(cell_type: Cell) -> Tile {
  return (cell_type, 0);
}

pub fn get_tile_at(options: &Options, world: &World, x: i32, y: i32) -> Cell {
  if x < world.min_x || x > world.max_x || y < world.min_y || y > world.max_y {
    // OOB. Use generated value
    return generate_cell(options, x, y);
  }

  return world.tiles[(world.min_x.abs() - x) as usize][(world.min_y.abs() - y) as usize].0;
}
