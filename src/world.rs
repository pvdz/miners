// use std::fmt::Write;
use std::collections::VecDeque;

use rand::prelude::*;
use rand_pcg::Pcg64;
use rand::distributions::{Standard};

use super::cell::*;
use super::fountain::*;
use super::windrone::*;
use super::hydrone::*;
use super::color::*;
use super::movable::*;
use super::slottable::*;
use super::inventory::*;
use super::icons::*;
use super::options::*;
use super::pickup::*;
use super::biome::*;
use super::tile::*;
use super::utils::*;
use super::slot_drone_launcher::*;
use super::slot_broken_gps::*;
use super::slot_drill::*;
use super::slot_hammer::*;
use super::slot_purity_scanner::*;
use super::slot_energy_cell::*;
use super::slot_emptiness::*;
use super::slot_windrone::*;
use super::slot_hydrone::*;
use super::expando::*;

// The world is procedurally generated and has no theoretical bounds.
// The map retained in memory is only has big as has been visited. Any unvisited cell (or well, any
// _unchanged_ cell rather) should use the default value according to the current seed for the
// procedural generation of it.
// The world needs to be extendable on both sides efficiently but also need efficient direct access.
// Transposing the 2d world on a 1d array is therefor infeasible because extending one axis means
// moving potentially many bytes. A simple vec has the same problem in one direction.
// So we use a vec deque which supports exactly this.

pub type Grid = VecDeque<VecDeque<Cell>>;

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
  pub expandos: Vec<Expando>,
  pub fountains: Vec<Fountain>,
}

pub fn generate_cell(options: &Options, x: i32, y: i32) -> Cell {
  // For debugging: actually burn the grid into the world itself. Screws up the game but makes it less dependent on view printing logic.
  // if x == 0 && y == 0 {
  //   return Cell::ZeroZero;
  // }
  // if x % 10 == 0 || y % 10 == 0 {
  //   // Draw debug lines
  //   return Cell::TenLine;
  // }

  // println!("  generate_cell({}, {})", x, y);
  // Take the world seed and add the x as a <<32 value and y as is to the seed
  // If either x or y are negative they should subtract that value from the world seed
  // If the result is negative, it should wrap around.
  let nx: i64 = if x < 0 { -(-(x as i64) << 32) } else { (x as i64) << 32 };
  let cell_seed: u64 = ((options.seed as i64) + nx + (y as i64)) as u64;
  let mut cell_rng = Pcg64::seed_from_u64(cell_seed);

  // I guess start with the rarest stuff first, move to the common stuff, end with empty

  // some % of the cells should contain an energy container (arbitrary)
  let energy_roll = cell_rng.sample::<f32, Standard>(Standard);
  if energy_roll < 0.05 {
    if energy_roll < 0.01 {
      // For windrones. Don't need as much (but some)
      return Cell { tile: Tile::Empty, pickup: Pickup::Wind, tile_value: 0, pickup_value: 0, visited: 0 };
    }
    return Cell { tile: Tile::Empty, pickup: Pickup::Energy, tile_value: 0, pickup_value: 0, visited: 0 };
  }

  // Roughly half the cells should be filled with walls
  if cell_rng.sample::<f32, Standard>(Standard) < 0.4f32 {
    // Roughly speaking, 10% is 3, 30% is 2, 60% is 1?
    let kind_roll: f32 = cell_rng.sample::<f32, Standard>(Standard);
    let value_roll: f32 = cell_rng.sample::<f32, Standard>(Standard);
    let reward_roll: f32 = cell_rng.sample::<f32, Standard>(Standard);

    // 60% chance for wall to be common, 30% to be uncommon, 10% to be rare :shrug:
    let tile_value = if value_roll < 0.1 { 2 } else if value_roll < 0.4 { 1 } else { 0 };

    let mut pickup_value = tile_value;
    let reward_value =
      if reward_roll < 0.1 { Pickup::Diamond }
      else if reward_roll < 0.4 { Pickup::Wood }
      else if reward_roll < 0.41 {
        // Fake pickup. Causes water/gas/etc fluids.
        // Set its size between 5 and 10 cells
        pickup_value = MIN_EXPANDO_SIZE + ((MAX_EXPANDO_SIZE - MIN_EXPANDO_SIZE) as f32 * cell_rng.sample::<f32, Standard>(Standard)).round() as u32;
        // Mark the pickup as an expando. There will be special handling for this.
        Pickup::Expando
      }
      else { Pickup::Stone };

    if kind_roll < 0.1 {
      return Cell { tile: Tile::Wall3, pickup: reward_value, tile_value, pickup_value, visited: 0 };
    }

    if kind_roll < 0.4 {
      return Cell { tile: Tile::Wall2, pickup: reward_value, tile_value, pickup_value, visited: 0 };
    }

    return Cell { tile: Tile::Wall1, pickup: reward_value, tile_value, pickup_value, visited: 0 };
  }

  return Cell { tile: Tile::Empty, pickup: Pickup::Nothing, tile_value: 0, pickup_value: 0, visited: 0 };
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

  let mut ygrid: VecDeque<VecDeque<Cell>> = VecDeque::new();
  let mut xgrid: VecDeque<Cell> = VecDeque::new();
  xgrid.push_back(Cell { tile: Tile::Empty, pickup: Pickup::Nothing, tile_value: 0, pickup_value: 0, visited: 0 });
  ygrid.push_back(xgrid);

  let mut world = World {
    min_x: 0,
    min_y: 0,
    max_x: 0,
    max_y: 0,
    tiles: ygrid,
    expandos: vec!(),
    fountains: vec!(),
  };

  // Use this to prerender part of the world for inspection reasons
  ensure_cell_in_world(&mut world, options, -5, -5);
  ensure_cell_in_world(&mut world, options, 5, 5);

  return world;
}

pub fn tick_world(world: &mut World, options: &Options) {
  // Walk backwards because they may be removed when they become depleted
  for n in (0..world.expandos.len()).rev() {
    tick_expando(n, world, options);
  }
  for n in (0..world.fountains.len()).rev() {
    tick_fountain(n, world, options);
  }
}

// fn bound_ex(x: i32, y: i32, min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> bool {
//   return x >= min_x && x < max_x && y >= min_y && y < max_y;
// }

fn bound_inc(x: i32, y: i32, min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> bool {
  return x >= min_x && x <= max_x && y >= min_y && y <= max_y;
}

pub fn coord_to_index(x: i32, y: i32, world: &World) -> (i32, i32) {
  return (world.min_x.abs() - x, world.min_y.abs() - y);
}

pub fn assert_world_dimensions(world: &World) {
  assert_eq!(world.tiles.len() as i32, world.min_y.abs() + 1 + world.max_y, "World should have min_y+1+max_y rows");
  for y in 0..world.tiles.len() {
    assert_eq!(world.tiles[y].len() as i32, world.min_x.abs() + 1 + world.max_x, "World should have each row with min_x+1+max_x cells");
  }
}

pub fn assert_arr_xy_in_world(world: &World, wx: i32, wy: i32, ax: usize, ay: usize) {
  // Only assert xy (array coords, not world coords!), and not the entire rectangle

  assert!(wx >= world.min_x, "assert_arr_xy_in_world; wx underflow");
  assert!(wy >= world.min_y, "assert_arr_xy_in_world; wy underflow");
  assert!(wx <= world.max_x, "assert_arr_xy_in_world; wx overflow");
  assert!(wy <= world.max_y, "assert_arr_xy_in_world; wy overflow");
  // assert!(ax >= 0, "assert_arr_xy_in_world; ax underflow");
  // assert!(ay >= 0, "assert_arr_xy_in_world; ay underflow");
  assert!(ax < (world.min_x.abs() + 1 + world.max_x) as usize, "assert_arr_xy_in_world; ax overflow");
  assert!(ay < (world.min_y.abs() + 1 + world.max_y) as usize, "assert_arr_xy_in_world; ay overflow");

  assert!(world.tiles.len() > ay, "assert_arr_xy_in_world; tile.len <= ay");
  assert!(world.tiles[ay].len() > ax, "assert_arr_xy_in_world; tile.len <= ax");
}

fn paint_maybe(x: i32, y: i32, what: String, view: &mut Vec<Vec<String>>, viewport_offset_x: i32, viewport_offset_y: i32, viewport_size_w: usize, viewport_size_h: usize, vox: i32, voy: i32) {
  // if the viewport offsets at <-25, -25> and the miner is at <0,0> then paint it at <25,25>
  // <-25,-25> and <1,1> then <26,26>
  // <0,0> and <10,20> then <10,20>
  // <1,2> and <9,18> then <10,20>

  // Convert view and the actor to the absolute coordinates (u32)
  // Subtract the viewport coords from the actor coords
  // That's where to paint the actor

  // First confirm whether the actor is within the viewport anyways
  if bound_inc(x, y, viewport_offset_x, viewport_offset_y, viewport_offset_x + viewport_size_w as i32, viewport_offset_y + viewport_size_h as i32) {
    // Yes. Convert the coords to absolute (vec) indexes.

    // "actor view abs x/y", or "where are we painting this miner in the output data"
    let avax = vox + if x < 0 { (viewport_offset_x - x).abs() } else { viewport_offset_x.abs() + x };
    let avay = voy + if y < 0 { (viewport_offset_y - y).abs() } else { viewport_offset_y.abs() + y };

    // If the actor coord is negative, then subtract it from the viewport. Otherwise add the
    // absolute viewport coord plus one (for the 0,0 cell because the range is inclusive).
    // <-10, -10> and <-7, 3>:
    // x: (-10 - -7).abs() = -3
    // y: (-10).abs() + 1 + 3 = 14
    // -> <-3, 14>

    view[avay as usize][avax as usize] = what;
  }
}

fn paint_biome_actors(biome: &Biome, biome_index: usize, options: &Options, view: &mut Vec<Vec<String>>, viewport_offset_x: i32, viewport_offset_y: i32, viewport_size_w: usize, viewport_size_h: usize, vox: i32, voy: i32) {
  if biome_index == 0 {
    // Paint the drones first. This way the miner goes on top in case of overlap.
    for drone in &biome.miner.drones {
      if drone.movable.now_energy == 0.0 {
        // Do not paint idle drones
        continue;
      }

      let drone_visual =
        add_fg_color_with_reset(
          &format!("{} ", match drone.movable.dir {
            Direction::Up => ICON_DRONE_UP,
            Direction::Down => ICON_DRONE_DOWN,
            Direction::Left => ICON_DRONE_LEFT,
            Direction::Right => ICON_DRONE_RIGHT,
          }),
          COLOR_DRONE
        ).to_string();

      paint_maybe(drone.movable.x, drone.movable.y, drone_visual, view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);
    }
  }

  let miner_visual =
    if options.paint_miner_ids {
      match biome_index {
        0 => "00".to_string(),
        1 => "11".to_string(),
        2 => "22".to_string(),
        3 => "33".to_string(),
        4 => "44".to_string(),
        5 => "55".to_string(),
        6 => "66".to_string(),
        7 => "77".to_string(),
        8 => "88".to_string(),
        9 => "99".to_string(),
        _ => "@@".to_string(),
      }
    } else {
      if biome_index == 0 {
        add_fg_color_with_reset(
          &format!("{} ", match biome.miner.movable.dir {
            Direction::Up => ICON_MINER_UP,
            Direction::Down => ICON_MINER_DOWN,
            Direction::Left => ICON_MINER_LEFT,
            Direction::Right => ICON_MINER_RIGHT,
          }),
          COLOR_MINER
        )
      } else {
        add_fg_color_with_reset(&ICON_GHOST.to_string(), COLOR_GHOST)
      }
    };

  paint_maybe(biome.miner.movable.x, biome.miner.movable.y, miner_visual, view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);
}

pub fn serialize_world(world0: &World, biomes: &Vec<Biome>, options: &Options, best_miner_str: String, btree_str: String) -> String {
  // We assume a 150x80 terminal screen space (half my ultra wide)
  // We draw every cell twice because the terminal cells have a 1:2 w:h ratio

  assert_world_dimensions(world0);

  // Start by painting the world. Give it a border too (annoying with calculations but worth it)

  // Note: top has a forced empty line.
  let wv_margin: (usize, usize, usize, usize) = (2, 1, 1, 1);
  // assert!(wv_margin.0 >= 0);
  // assert!(wv_margin.1 >= 0);
  // assert!(wv_margin.2 >= 0);
  // assert!(wv_margin.3 >= 0);
  let wv_border: (usize, usize, usize, usize) = (1, 1, 1, 1);
  // assert!(wv_border.0 >= 0);
  // assert!(wv_border.1 >= 0);
  // assert!(wv_border.2 >= 0);
  // assert!(wv_border.3 >= 0);

  // World coordinates for the viewport:
  let viewport_offset_x: i32 = -25; // (TODO: this should make sure the character location is included in the viewport)
  let viewport_offset_y: i32 = -25; // Same ^
  let viewport_size_w: usize = 51;
  let viewport_size_h: usize = 51;

  let wv_width = wv_margin.3 + wv_border.3 + viewport_size_w + wv_border.1 + wv_margin.1;
  // let wv_height = wv_margin.3 + wv_border.3 + (viewport_size_h * 2) + wv_border.1 + wv_margin.1;

  // Create strings that form lines of margins and borders

  let mut view: Vec<Vec<String>> = vec!();

  // Forced empty line at the top
  let empty_top_line: Vec<String> = std::iter::repeat("  ".to_string()).take(wv_width + 50).collect();
  view.push(empty_top_line);

  // Top margin line
  for _ in 1..wv_margin.0 {
    let margin_top: Vec<String> = std::iter::repeat(format!("{}{}", ICON_MARGIN, ICON_MARGIN)).take(wv_width).collect();
    view.push(margin_top);
  }

  // Top border line
  // Has to take the corners into account too
  let mut border_top: Vec<String> = vec!();
  // Starts with the left-margin (if any)
  if wv_margin.3 == 1 { for _ in 0..wv_margin.3 { border_top.push(format!("{}{}", ICON_MARGIN, ICON_MARGIN)); } }
  // border; corner + horizontal line + corner
  if wv_border.0 == 1 && wv_border.3 == 1 { border_top.push(format!(" {}", ICON_BORDER_TL)); }
  if wv_border.0 == 1 { for _ in 0..viewport_size_w { border_top.push(format!("{}{}", ICON_BORDER_H, ICON_BORDER_H)); } }
  if wv_border.0 == 1 && wv_border.1 == 1 { border_top.push(format!("{} ", ICON_BORDER_TR)); }
  // append the right margin (if any)
  if wv_margin.1 == 1 { for _ in 0..wv_margin.1 { border_top.push(format!("{}{}", ICON_MARGIN, ICON_MARGIN)); } }
  view.push(border_top);

  // The middle rows contain the real world view :)
  for j in 0..viewport_size_h as i32 {
    let wy = viewport_offset_y + j;
    let mut line: Vec<String> = vec!();
    // Prepend the margin and border (if any)
    for _ in 0..wv_margin.3 { line.push(format!("{}{}", ICON_MARGIN, ICON_MARGIN)); }
    for _ in 0..wv_border.3 { line.push(format!(" {}", ICON_BORDER_V)); }
    // Paint the world background tiles
    for i in 0..viewport_size_w as i32 {
      let wx = viewport_offset_x + i;

      if options.paint_zero_zero && wx == 0 && wy == 0 { line.push(ICON_DEBUG_ORIGIN.to_string()); } // Force-paint the origin (0,0), regardless of the game world state
      if options.paint_ten_lines && (wx % 10 == 0 || wy % 10 == 0) { line.push(ten_line_cell(wx, wy)); } // Force-paint grid over world, regardless of the game world state
      else if options.paint_empty_world { line.push(format!("{}{}", ICON_DEBUG_BLANK, ICON_DEBUG_BLANK)); } // Force-paint an empty block instead of the actual world (game world is not changed)
      else {
        let (tile, pickup, value, visited) = get_cell_stuff_at(&options, &world0, wx, wy);
        let mut str = cell_to_uncolored_string(tile, pickup, wx, wy);
        if options.paint_visited && visited > 0 && matches!(tile, Tile::Empty) && matches!(pickup, Pickup::Nothing) {
          if options.paint_visited_bool {
            str = format!("⣿⣿");
          } else {
            // Increase the intensity of the dots in a "circle" from the center
            // There is a 4x4 grid of braille dots (a cell is two chars)
            // start with 1,1 to 2,1 to 2,2, to 1,2, to 0,2, 0,1 etc, clockwise
            // Braille chars are exhaustive in the 2x4 pattern so we can do this.
            // ⠐ |⠐⠂|⠐⠆|⠰⠆|⠴⠆|⠶⠆|⠷⠆|⠿⠆|⠿⠇|⠿⠏|⠿⠟|⠿⠿|⠿⢿|⠿⣿|⢿⣿|⣿⣿
            // https://en.wikipedia.org/wiki/Braille_Patterns
            str = match visited {
              0 => "".to_string(),
              1 => "⠐ ".to_string(),
              2 => "⠐⠂".to_string(),
              3 => "⠐⠆".to_string(),
              4 => "⠰⠆".to_string(),
              5 => "⠴⠆".to_string(),
              6 => "⠶⠆".to_string(),
              7 => "⠷⠆".to_string(),
              8 => "⠿⠆".to_string(),
              9 => "⠿⠇".to_string(),
              10 => "⠿⠏".to_string(),
              11 => "⠿⠟".to_string(),
              12 => "⠿⠿".to_string(),
              13 => "⠿⢿".to_string(),
              14 => "⠿⣿".to_string(),
              15 => "⢿⣿".to_string(),
              _ => "⣿⣿".to_string(),
            }
          }
        }
        str = cell_add_color(&str, tile, value, pickup);
        line.push(str);
      }
    }

    // Append the right side margin and border (if any)
    for _ in 0..wv_border.1 { line.push(format!("{} ", ICON_BORDER_V)); }
    for _ in 0..wv_margin.1 { line.push(format!("{}{}", ICON_MARGIN, ICON_MARGIN)); }
    // That is one line finished
    view.push(line);
  }

  // Now add the bottom border and margin (if any)
  // Has to take the corners into account too
  // Starts with the left-margin (if any)
  let mut border_bottom: Vec<String> = std::iter::repeat(format!("{}{}", ICON_MARGIN, ICON_MARGIN)).take(wv_margin.3).collect();
  // border; corner + horizontal line + corner
  if wv_border.2 == 1 && wv_border.3 == 1 { border_bottom.push(format!(" {}", ICON_BORDER_BL)); }
  if wv_border.2 == 1 { for _ in 0..viewport_size_w { border_bottom.push(format!("{}{}", ICON_BORDER_H, ICON_BORDER_H)); } }
  if wv_border.2 == 1 && wv_border.1 == 1 { border_bottom.push(format!("{} ", ICON_BORDER_BR)); }
  // append the right margin (if any)
  if wv_border.2 == 1 { for _ in 0..wv_margin.1 { border_bottom.push(format!("{}{}", ICON_MARGIN, ICON_MARGIN)); } }

  view.push(border_bottom);

  // Bottom margin line
  for _ in 0..wv_margin.2 {
    let margin_bottom: Vec<String> = std::iter::repeat(format!("{}{}", ICON_MARGIN, ICON_MARGIN)).take(wv_width).collect();
    view.push(margin_bottom);
  }

  // That should complete the world view. `view` should be wv_width x wv_height cells right now.
  // Remaining steps are to paint the moving actors, color some tiles, and add ui elements

  // Where is the top-left most cell that the viewport actually shows? (skip margin+border)
  let vox = (wv_margin.3 + wv_border.3) as i32;
  let voy = (wv_margin.0 + wv_border.0) as i32;

  assert!(biomes.len() >= 1, "there should be at least one biome");
  for (i, biome) in biomes.iter().enumerate() {
    if i == 0 { continue; }
    paint_biome_actors(biome, i, options, &mut view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);
  }
  paint_biome_actors(&biomes[0], 0, options, &mut view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);

  // Paint the windrone, if it's in flight
  // The windrone is incorporeal (like a ghost, unable to collide with objects or whatever). Paint on top.
  if matches!(biomes[0].miner.windrone.state, WindroneState::FlyingToGoal) || matches!(biomes[0].miner.windrone.state, WindroneState::FlyingHome) {
    paint_maybe(biomes[0].miner.windrone.movable.x, biomes[0].miner.windrone.movable.y, "##".to_string(), &mut view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);
  }

  // Paint the hydrone, if it's moving
  // The hydrone is incorporeal (like a ghost, unable to collide with objects or whatever). Paint on top. (TODO: incorporeal is tbd)
  match biomes[0].miner.hydrone.state {
    | HydroneState::MovingToOrigin
    | HydroneState::MovingToNeighborCell
    | HydroneState::BuildingArrowCell
    | HydroneState::PickingUpMiner
    | HydroneState::DeliveringMiner
    => paint_maybe(biomes[0].miner.hydrone.movable.x, biomes[0].miner.hydrone.movable.y, ui_hydrone(&biomes[0].miner.hydrone), &mut view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy),
    HydroneState::Unconstructed => {}
    HydroneState::WaitingForWater => {}
  }

  // Draw UI

  // Offsets for the UI. Start at the top of the map to the right of it.
  // let uox = (wv_margin.3 + wv_border.3 + viewport_size_w + wv_border.1 + wv_margin.1 + 1) as i32;
  // let uoy = 0; // Just the top.

  let vlen = view.len();

  // Append each line to the map
  view[1].push(format!(" Gene mutation rate: {}%  Slot mutation rate: {}%   Miner batch size: {}   Reset rate: {: <120}", options.mutation_rate_genes, options.mutation_rate_slots, options.batch_size, options.reset_rate).to_string());
  view[2].push(format!(" {: <150}", best_miner_str));
  view[3].push(format!(" {: <150}", btree_str));
  view[4].push(std::iter::repeat(' ').take(150).collect::<String>());
  view[5].push(format!(" Miner; {: <150}", ' '));
  view[6].push(std::iter::repeat(' ').take(143).collect::<String>());
  view[7].push(format!("   {: <150}", biomes[0].miner.helix));
  view[8].push(format!("   XY: {: >4}, {: <10} {: <45} Points: {: <10} Steps: {: <15} Energy {: <10}", biomes[0].miner.movable.x, biomes[0].miner.movable.y, progress_bar(30, biomes[0].miner.movable.now_energy, biomes[0].miner.movable.init_energy, true), get_points(&biomes[0].miner.meta.inventory), format!("{} ({})", biomes[0].miner.movable.history.len(), biomes[0].miner.movable.unique.len()), biomes[0].miner.movable.now_energy.round()).to_string());
  view[9].push(format!("   Inventory:   {: <100}", ui_inventory(&biomes[0].miner.meta.inventory)));
  view[10].push(std::iter::repeat(' ').take(100).collect::<String>());

  let so = 11;
  for n in 0..biomes[0].miner.slots.len() {
    let slot: &Slottable = &biomes[0].miner.slots[n];
    let (head, progress, tail) = match slot.kind {
      SlotKind::Windrone => ui_slot_windrone(slot, &biomes[0].miner.windrone, biomes[0].miner.meta.inventory.wind),
      SlotKind::Hydrone => ui_slot_hydrone(slot, &biomes[0].miner.hydrone, biomes[0].miner.meta.inventory.water),
      SlotKind::Drill => ui_slot_drill(slot),
      SlotKind::DroneLauncher => ui_slot_drone_launcher(slot, &biomes[0].miner.drones[slot.nth as usize]),
      SlotKind::Hammer => ui_slot_hammer(slot),
      SlotKind::Emptiness => ui_slot_emptiness(slot),
      SlotKind::EnergyCell => ui_slot_energy_cell(slot),
      SlotKind::PurityScanner => ui_slot_purity_scanner(slot),
      SlotKind::BrokenGps => ui_slot_broken_gps(slot),
    };
    view[so + n].push(format!(" {: <20} {: <40} {: <70}", head, progress, tail).to_string());
  }

  for y in so + biomes[0].miner.slots.len()..vlen - 5 {
    view[y].push(std::iter::repeat(' ').take(100).collect::<String>());
  }

  view[vlen - 5].push(format!(" Keys: toggle visual: v⏎   save and quite: q⏎  speed [{}]  faster: -⏎   slower: +⏎   return-stepper: x⏎ {: <50}", options.speed, ' '));
  view[vlen - 4].push(format!("       gene mutation rate [{}]  up: o⏎   up 5: oo⏎   down: p⏎   down 5: pp⏎ {: <50}", options.mutation_rate_genes, ' '));
  view[vlen - 3].push(format!("       slot mutation rate [{}]  up: l⏎   up 5: ll⏎   down: k⏎   down 5: kk⏎ {: <50}", options.mutation_rate_slots, ' '));
  view[vlen - 2].push(format!("       batch size [{}]  up: m⏎   down: n⏎   restart with random helix: r⏎   restart from best: b⏎ {: <50}", options.batch_size, ' '));
  view[vlen - 1].push(format!("       mutate [{}]: g⏎   auto reset [{}] after [{}] miners: t⏎ {: <50}", if options.mutate_from_best { "overall best" } else { "last winner" }, if options.reset_after_noop { "after noop" } else { "regardless" }, options.reset_rate, ' '));


  for row in view.iter() {
    let border_top_str: String = row.join("");
    println!("{}", border_top_str);
  }


  return "".to_owned();
}

pub fn ensure_cell_in_world(world: &mut World, options: &Options, x: i32, y: i32) {
  // Expansion occurs by pre/appending a cell to all rows/cols of the axis that expands

  // Note: the world bounds are inclusive (so they exist if x>=world.min_x and x<=world.max_x)

  // println!("ensure_cell_in_world: world box: {}X{} ~ {}X{}, target coord: {}X{}", world.min_x, world.min_y, world.max_x, world.max_y, x, y);

  if x < world.min_x {
    // OOB. We have to prepend a cell to each row
    let world_height = world.min_y.abs() + world.max_y + 1;
    assert!(x < 0, "x must be negative ({}) because its smaller than min_x ({}) which must always be <=0", x, world.min_x);
    let to_prepend = x.abs() - world.min_x.abs();
    // println!("Expanding -x (left), prepending {} cols to {} rows", to_prepend, world_height);

    for j in 0..world_height {
      let row = &mut world.tiles[j as usize];
      for i in 1..=to_prepend {
        let gx = world.min_x - i;
        let gy = world.min_y + j;
        let cell = generate_cell(&options, gx, gy);
        row.push_front(cell);
      }
    }
    world.min_x = world.min_x - to_prepend;
  } else if x > world.max_x {
    // OOB. We have to append a cell to each row
    let world_height = world.min_y.abs() + world.max_y + 1;
    assert!(x > 0, "y must be positive ({}) because its bigger than max_x ({}) which must always be >=0", x, world.max_x);
    let to_append = x - world.max_x;
    // println!("Expanding +x (right), appending {} cols to {} rows", to_append, world_height);

    for j in 0..world_height {
      let row = &mut world.tiles[j as usize];
      for i in 1..=to_append {
        let gx = world.max_x + i;
        let gy = world.min_y + j;
        let cell = generate_cell(&options, gx, gy);
        row.push_back(cell);
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
    for j in 1..=to_prepend {
      let mut new_row: VecDeque<Cell> = VecDeque::new();
      for i in 0..world_width {
        let gx = world.min_x + i;
        let gy = world.min_y - j;
        let cell = generate_cell(&options, gx, gy);
        new_row.push_back(cell);
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
    for j in 1..=to_append {
      let mut new_row: VecDeque<Cell> = VecDeque::new();
      for i in 0..world_width {
        let gx = world.min_x + i;
        let gy = world.max_y + j;
        let cell = generate_cell(&options, gx, gy);
        new_row.push_back(cell);
      }
      world.tiles.push_back(new_row);
    }

    world.max_y = world.max_y + to_append;
  }

  assert_world_dimensions(world);
}

pub fn create_cell(tile: Tile, pickup: Pickup, tile_value: u32, pickup_value: u32) -> Cell {
  return Cell { tile, pickup, tile_value, pickup_value, visited: 0 };
}

pub fn create_visited_cell(tile: Tile, pickup: Pickup, tile_value: u32, pickup_value: u32, visited: u32) -> Cell {
  return Cell { tile, pickup, tile_value, pickup_value, visited };
}

pub fn get_cell_stuff_at(options: &Options, world: &World, wx: i32, wy: i32) -> (Tile, Pickup, u32, u32) {
  // Return tile, pickup, value, visited.

  // wx/wy should be world coordinates
  assert_world_dimensions(world);

  // Is the cell explicitly stored in the world right now? If not then use the procedure.
  if wx < world.min_x || wx > world.max_x || wy < world.min_y || wy > world.max_y {
    if options.hide_world_oob {
      return (Tile::HideWorld, Pickup::Nothing, 0, 0);
    }

    // OOB. Use generated value
    let cell = generate_cell(options, wx, wy);
    return (cell.tile, cell.pickup, cell.tile_value, 0);
  }

  if options.hide_world_ib {
    return (Tile::HideWorld, Pickup::Nothing, 0, 0);
  }

  // If x is negative then the coord is `min_x.abs() + x` (ex: `abs(-10) + -5` or `10 - 5` = 5)
  // If x is positive then the coord is `min_x.abs() + x` (ex: `abs(-10) + 5` or `10 + 5` = 15)
  // If x is zero then the coord is `min_x.abs()`
  // So in all cases, the absolute coord of x is `min_x.abs() + x`
  let ax = world.min_x.abs() + wx;
  let ay = world.min_y.abs() + wy;

  // println!("real {} >= {} <= {} ({}, {})   {} >= {} <= {} ({}, {})", world.min_x, x, world.max_x, world.tiles[0].len(), world.min_x <= x && world.max_x >= x, world.min_y, y, world.max_y, world.tiles.len(), world.min_y <= y && world.max_y >= y);
  // println!("abso {} >= {} <  {} ({}, {})   {} >= {} <  {} ({}, {})", 0, ax, world.min_x.abs() + 1 + world.max_x, world.tiles[0].len(), 0 <= ax && world.tiles[0].len() as i32 >= ax, 0, ay, world.min_y.abs() + 1 + world.max_y, world.tiles.len(), 0 <= ay && world.tiles.len() as i32 >= ay);

  assert!(wx >= world.min_x);
  assert!(wy >= world.min_y);
  assert!(wx <= world.max_x);
  assert!(wy <= world.max_y);
  assert!(ax >= 0);
  assert!(ay >= 0);
  assert!(ax < (world.min_x.abs() + 1 + world.max_x));
  assert!(ay < (world.min_y.abs() + 1 + world.max_y));

  let cell = &world.tiles[ay as usize][ax as usize];
  return (cell.tile, cell.pickup, cell.tile_value, cell.visited);
}

pub fn get_cell_tile_at(options: &Options, world: &World, wx: i32, wy: i32) -> Tile {
  // wx/wy should be world coordinates
  assert_world_dimensions(world);

  // Is the cell explicitly stored in the world right now? If not then use the procedure.
  if wx < world.min_x || wx > world.max_x || wy < world.min_y || wy > world.max_y {
    if options.hide_world_oob {
      return Tile::HideWorld;
    }

    // OOB. Use generated value
    return generate_cell(options, wx, wy).tile;
  }

  if options.hide_world_ib {
    return Tile::HideWorld;
  }

  // If x is negative then the coord is `min_x.abs() + x` (ex: `abs(-10) + -5` or `10 - 5` = 5)
  // If x is positive then the coord is `min_x.abs() + x` (ex: `abs(-10) + 5` or `10 + 5` = 15)
  // If x is zero then the coord is `min_x.abs()`
  // So in all cases, the absolute coord of x is `min_x.abs() + x`
  let ax = world.min_x.abs() + wx;
  let ay = world.min_y.abs() + wy;

  // println!("real {} >= {} <= {} ({}, {})   {} >= {} <= {} ({}, {})", world.min_x, x, world.max_x, world.tiles[0].len(), world.min_x <= x && world.max_x >= x, world.min_y, y, world.max_y, world.tiles.len(), world.min_y <= y && world.max_y >= y);
  // println!("abso {} >= {} <  {} ({}, {})   {} >= {} <  {} ({}, {})", 0, ax, world.min_x.abs() + 1 + world.max_x, world.tiles[0].len(), 0 <= ax && world.tiles[0].len() as i32 >= ax, 0, ay, world.min_y.abs() + 1 + world.max_y, world.tiles.len(), 0 <= ay && world.tiles.len() as i32 >= ay);

  assert!(wx >= world.min_x);
  assert!(wy >= world.min_y);
  assert!(wx <= world.max_x);
  assert!(wy <= world.max_y);
  assert!(ax >= 0);
  assert!(ay >= 0);
  assert!(ax < (world.min_x.abs() + 1 + world.max_x));
  assert!(ay < (world.min_y.abs() + 1 + world.max_y));

  return world.tiles[ay as usize][ax as usize].tile;
}

pub fn get_cell_value_at(options: &Options, world: &World, wx: i32, wy: i32) -> u32 {
  // wx/wy should be world coordinates
  assert_world_dimensions(world);

  // Is the cell explicitly stored in the world right now? If not then use the procedure.
  if wx < world.min_x || wx > world.max_x || wy < world.min_y || wy > world.max_y {
    if options.hide_world_oob {
      return 0;
    }

    // OOB. Use generated value
    return generate_cell(options, wx, wy).tile_value;
  }

  if options.hide_world_ib {
    return 0;
  }

  // If x is negative then the coord is `min_x.abs() + x` (ex: `abs(-10) + -5` or `10 - 5` = 5)
  // If x is positive then the coord is `min_x.abs() + x` (ex: `abs(-10) + 5` or `10 + 5` = 15)
  // If x is zero then the coord is `min_x.abs()`
  // So in all cases, the absolute coord of x is `min_x.abs() + x`
  let ax = world.min_x.abs() + wx;
  let ay = world.min_y.abs() + wy;

  assert_arr_xy_in_world(world, wx, wy, ax as usize, ay as usize);

  return world.tiles[ay as usize][ax as usize].tile_value;
}

pub fn set_cell_tile_at(_options: &Options, world: &mut World, wx: i32, wy: i32, tile: Tile) {
  let ax = world.min_x.abs() + wx;
  let ay = world.min_y.abs() + wy;

  assert_arr_xy_in_world(world, wx, wy, ax as usize, ay as usize);

  world.tiles[ay as usize][ax as usize].tile = tile;
}
pub fn set_cell_tile_value_at(_options: &Options, world: &mut World, wx: i32, wy: i32, value: u32) {
  let ax = world.min_x.abs() + wx;
  let ay = world.min_y.abs() + wy;

  assert_arr_xy_in_world(world, wx, wy, ax as usize, ay as usize);

  world.tiles[ay as usize][ax as usize].tile_value = value;
}
pub fn set_cell_pickup_at(_options: &Options, world: &mut World, wx: i32, wy: i32, pickup: Pickup) {
  let ax = world.min_x.abs() + wx;
  let ay = world.min_y.abs() + wy;

  assert_arr_xy_in_world(world, wx, wy, ax as usize, ay as usize);

  world.tiles[ay as usize][ax as usize].pickup = pickup;
}
pub fn set_cell_pickup_value_at(_options: &Options, world: &mut World, wx: i32, wy: i32, value: u32) {
  let ax = world.min_x.abs() + wx;
  let ay = world.min_y.abs() + wy;

  assert_arr_xy_in_world(world, wx, wy, ax as usize, ay as usize);

  world.tiles[ay as usize][ax as usize].pickup_value = value;
}
