// use std::fmt::Write;
use std::collections::VecDeque;

use rand::prelude::*;
use rand_pcg::Pcg64;
use rand::distributions::{Standard};
use crate::slot_jacks_compass::ui_slot_jacks_compass;

use super::cell::*;
use super::helix::*;
use super::fountain::*;
use super::drone_win::*;
use super::drone_san::*;
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
use super::slot_magnet::*;
use super::slot_purity_scanner::*;
use super::slot_energy_cell::*;
use super::slot_emptiness::*;
use super::slot_windrone::*;
use super::slot_sandrone::*;
use super::expando::*;
use super::app_state::*;

// The world is procedurally generated and has no theoretical bounds.
// The map retained in memory is only has big as has been visited. Any unvisited cell (or well, any
// _unchanged_ cell rather) should use the default value according to the current seed for the
// procedural generation of it.
// The world needs to be extendable on both sides efficiently but also need efficient direct access.
// Transposing the 2d world on a 1d array is therefor infeasible because extending one axis means
// moving potentially many bytes. A simple vec has the same problem in one direction.
// So we use a vec deque which supports exactly this.


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

// - Item to slowly construct paths (or ability? or auto? resource cost?) which reduce energy spent on that tile

// Miners should perhaps be simply helix:points pairs. Paths are going to be ambiguous anyways so unique paths don't matter much. Points is an easy concept to grasp and aim for and it kinda limits the max number of miners to track. More so than paths.


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

pub fn tick_world(options: &mut Options, _state: &mut AppState, biome: &mut Biome) {
  // world: &mut World, options: &Options, sandrone: &Sandrone

  // Walk backwards because they may be removed when they become depleted
  for n in (0..biome.world.expandos.len()).rev() {
    tick_expando(n, &mut biome.world, options);
  }
  for n in (0..biome.world.fountains.len()).rev() {
    tick_fountain(n, &mut biome.world, options);
  }

  // Game of life the castle
  if false && biome.miner.sandrone.post_castle > 0 {
    // Figure out the rectangle and CA them
    let magic_min_x = biome.miner.sandrone.expansion_min_x;
    let magic_min_y = biome.miner.sandrone.expansion_min_y;
    let magic_max_x = biome.miner.sandrone.expansion_max_x;
    let magic_max_y = biome.miner.sandrone.expansion_max_y;

    for y in magic_min_y+1..magic_max_y {
      for x in magic_min_x+1..magic_max_x {
        // Each cell can have one of two tiles; wall or empty. In addition it can have three
        // sub-states; empty, medrone, or sandrone.
        // Apply a CA rule for each. Maybe wrap-around, maybe not.

        // abc
        // def
        // ghi

        let a = matches!(get_cell_tile_at(options, &biome.world, x - 1, y - 1), Tile::Push);
        let b = matches!(get_cell_tile_at(options, &biome.world, x + 0, y - 1), Tile::Push);
        let c = matches!(get_cell_tile_at(options, &biome.world, x + 1, y - 1), Tile::Push);
        let d = matches!(get_cell_tile_at(options, &biome.world, x - 1, y + 0), Tile::Push);
        let e = matches!(get_cell_tile_at(options, &biome.world, x + 0, y + 0), Tile::Push);
        let f = matches!(get_cell_tile_at(options, &biome.world, x + 1, y + 0), Tile::Push);
        let g = matches!(get_cell_tile_at(options, &biome.world, x - 1, y + 1), Tile::Push);
        let h = matches!(get_cell_tile_at(options, &biome.world, x + 0, y + 1), Tile::Push);
        let i = matches!(get_cell_tile_at(options, &biome.world, x + 1, y + 1), Tile::Push);

        // Find patterns, including rotations and mirrors

        // // Clean up entirely
        // if e && (
        //   (d && !(b || c || f || i || h)) ||
        //   (b && !(f || i || h || g || d)) ||
        //   (f && !(h || g || d || a || b)) ||
        //   (h && !(d || a || b || c || f))
        // ) {
        //   //  xxx   xox   xxx   xxx      xxx   oox   xxo   xxx      oxx   xoo   xxx   xxx      oxx   ooo   xxo   xxx
        //   //  oox   xox   xoo   xox      oox   xox   xoo   xox      oox   xox   xoo   xox      oox   xox   xoo   xox
        //   //  xxx   xxx   xxx   xox      oxx   xxx   xxx   xoo      xxx   xxx   xxo   oox      oxx   xxx   xxo   ooo
        //
        //   set_cell_tile_at(options, &mut biome.world, x, y, Tile::Impassible);
        // }

        // Game of life, part 1
        // - any live cell with 2 or 3 live neighbors, survives
        // - any dead cell with 3 neighbors, lives
        // - anything else is ded
        let n = (a as u8) + (b as u8) + (c as u8) + (d as u8) + (f as u8) + (g as u8) + (h as u8) + (i as u8);
        let v = if e {
          if n == 2 || n == 3 { 1 } else { 0 }
        } else {
          if n == 3 { 1 } else { 0 }
        };
        set_cell_tile_value_at(options, &mut biome.world, x, y, v);
      }
    }

    for y in magic_min_y+1..magic_max_y {
      for x in magic_min_x+1..magic_max_x {
        // Game of life; part 2
        let v = if get_cell_tile_value_at(options, &biome.world, x, y) == 0 { Tile::Impassible } else { Tile::Push };
        set_cell_tile_at(options, &mut biome.world, x, y, v);
      }
    }
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

pub fn assert_arr_xy_in_world(world: &World, wx: i32, wy: i32, ax: usize, ay: usize) {
  // Only assert xy (array coords, not world coords!), and not the entire rectangle

  assert!(wx >= world.min_x, "assert_arr_xy_in_world; wx underflow {} {}", wx, world.min_x);
  assert!(wy >= world.min_y, "assert_arr_xy_in_world; wy underflow {} {}", wy, world.min_y);
  assert!(wx <= world.max_x, "assert_arr_xy_in_world; wx overflow {} {}", wx, world.max_x);
  assert!(wy <= world.max_y, "assert_arr_xy_in_world; wy overflow {} {}", wy, world.min_y);
  // assert!(ax >= 0, "assert_arr_xy_in_world; ax underflow");
  // assert!(ay >= 0, "assert_arr_xy_in_world; ay underflow");
  assert!(ax < (world.min_x.abs() + 1 + world.max_x) as usize, "assert_arr_xy_in_world; ax overflow {} < {} + 1 + {}", ax, world.min_x.abs(), world.max_x);
  assert!(ay < (world.min_y.abs() + 1 + world.max_y) as usize, "assert_arr_xy_in_world; ay overflow {} < {} + 1 + {}", ay, world.min_y.abs(), world.max_y);

  assert!(world.tiles.len() > ay, "assert_arr_xy_in_world; tile.len <= ay; {} > {}", world.tiles.len(), ay);
  assert!(world.tiles[ay].len() > ax, "assert_arr_xy_in_world; tile.len <= ax; {} > {}", world.tiles[ay].len(), ax);
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
    // Yes it is. Convert the coords to absolute (vec) indexes.

    // "actor view abs x/y", or "where are we painting this miner in the output data"
    // vox/y is the offset where the top-left most tile is painted, past the margin and borders
    let avax = vox + (x - viewport_offset_x);
    let avay = voy + (y - viewport_offset_y);

    if view.len() <= avay as usize {
      let v = view.iter().map(|x| x.len()).collect::<Vec<_>>();
      panic!(
        "okay wtf1? pos: {}x{} ava: {}x{} min: {}x{} lens: ?x{} voyx: {}x{} max: {}x{}, {:?}",
        x, y, avax, avay, viewport_offset_x, viewport_offset_y, view.len(), vox, voy, viewport_offset_x + viewport_size_w as i32, viewport_offset_y + viewport_size_h as i32,
        v
      );
    }
    if view[avay as usize].len() <= avax as usize {
      panic!("okay wtf2? {}x{} {}x{} {}x{} {}x{} voyx: {}x{} {}x{}", x, y, avax, avay, viewport_offset_x, viewport_offset_y, view[avay as usize].len(), view.len(), vox, voy, viewport_offset_x + viewport_size_w as i32, viewport_offset_y + viewport_size_h as i32);
    }

    view[avay as usize][avax as usize] = what;
  }
}

fn paint_biome_actors(biome: &Biome, options: &Options, view: &mut Vec<Vec<String>>, viewport_offset_x: i32, viewport_offset_y: i32, viewport_size_w: usize, viewport_size_h: usize, vox: i32, voy: i32) {
  if biome.index == options.visible_index {
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
          COLOR_DRONE,
          options
        ).to_string();

      paint_maybe(drone.movable.x, drone.movable.y, drone_visual, view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);
    }
  }

  let miner_visual =
    if options.paint_miner_ids {
      match biome.index {
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
      if biome.index == options.visible_index {
        add_fg_color_with_reset(
          &format!("{} ", match biome.miner.movable.dir {
            Direction::Up => ICON_MINER_UP,
            Direction::Down => ICON_MINER_DOWN,
            Direction::Left => ICON_MINER_LEFT,
            Direction::Right => ICON_MINER_RIGHT,
          }),
          COLOR_MINER,
          options
        )
      } else {
        add_fg_color_with_reset(&ICON_GHOST.to_string(), COLOR_GHOST, options)
      }
    };

  paint_maybe(biome.miner.movable.x, biome.miner.movable.y, miner_visual, view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);
}

pub fn serialize_world(world0: &World, biomes: &Vec<Biome>, options: &Options, state: &mut AppState, best_miner_str: String, hmap_str: String) -> String {
  // We assume a 150x80 terminal screen space (half my ultra wide)
  // We draw every cell twice because the terminal cells have a 1:2 w:h ratio

  // Start by painting the world. Give it a border too (annoying with calculations but worth it)

  let ticks0 = biomes[options.visible_index].ticks;

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

  let viewport_size_w = state.viewport_size_w;
  let viewport_size_h = state.viewport_size_h;

  if state.center_on_miner_next {
    state.viewport_offset_x = biomes[options.visible_index].miner.movable.x - (viewport_size_w as i32) / 2;
    state.viewport_offset_y = biomes[options.visible_index].miner.movable.y - (viewport_size_h as i32) / 2;
    state.center_on_miner_next = false;
  }

  if state.auto_follow_miner {
    // Force the miner to be painted at least m and at most n tiles away from any viewport edge
    let x = biomes[options.visible_index].miner.movable.x;
    let y = biomes[options.visible_index].miner.movable.y;

    if x < state.viewport_offset_x + state.auto_follow_buffer_min {
      // Move viewport to make sure the miner is max tiles away from the left
      state.viewport_offset_x = x - state.auto_follow_buffer_max;
    }

    if y < state.viewport_offset_y + state.auto_follow_buffer_min {
      // Move viewport to make sure the miner is max tiles away from the top
      state.viewport_offset_y = y - state.auto_follow_buffer_max;
    }

    if x > state.viewport_offset_x + (state.viewport_size_w as i32) - state.auto_follow_buffer_min {
      // Move viewport to make sure the miner is max tiles away from the right
      state.viewport_offset_x = x - ((state.viewport_size_w as i32) - state.auto_follow_buffer_max);
    }

    if y > state.viewport_offset_y + (state.viewport_size_h as i32) - state.auto_follow_buffer_min {
      // Move viewport to make sure the miner is max tiles away from the bottom
      state.viewport_offset_y = y - ((state.viewport_size_h as i32) - state.auto_follow_buffer_max);
    }
  }

  // World coordinates for the viewport:
  let viewport_offset_x = state.viewport_offset_x;
  let viewport_offset_y = state.viewport_offset_y;

  let wv_width = wv_margin.3 + wv_border.3 + viewport_size_w + wv_border.1 + wv_margin.1;
  let wv_height = wv_margin.3 + wv_border.3 + (viewport_size_h) + wv_border.1 + wv_margin.1;

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
        let (tile, pickup, tile_value, visited) = get_cell_stuff_at(&options, &world0, wx, wy);
        let mut str = cell_to_uncolored_string(tile, pickup, tile_value, wx, wy);
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
        str = cell_add_color(&str, tile, tile_value, pickup, options);
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
  if options.show_biomes {
    for (i, biome) in &mut biomes.iter().enumerate() {
      if i == 0 { continue; }
      paint_biome_actors(biome, options, &mut view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);
    }
  }
  paint_biome_actors(&biomes[options.visible_index], options, &mut view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);

  // Paint the windrone, if it's in flight
  // The windrone is incorporeal (like a ghost, unable to collide with objects or whatever). Paint on top.
  if matches!(biomes[options.visible_index].miner.windrone.state, WindroneState::FlyingToGoal) || matches!(biomes[options.visible_index].miner.windrone.state, WindroneState::FlyingHome) {
    paint_maybe(biomes[options.visible_index].miner.windrone.movable.x, biomes[options.visible_index].miner.windrone.movable.y, ui_windrone(&biomes[options.visible_index].miner.windrone, options), &mut view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);
  }

  // Paint the sandrone, if it's moving
  // The sandrone is incorporeal (like a ghost, unable to collide with objects or whatever). Paint on top. (TODO: incorporeal is tbd)
  match biomes[options.visible_index].miner.sandrone.state {
    | SandroneState::MovingToOrigin
    | SandroneState::MovingToNeighborCell
    | SandroneState::BuildingArrowCell
    | SandroneState::PickingUpMiner
    | SandroneState::DeliveringMiner
    | SandroneState::Redecorating
    => paint_maybe(biomes[options.visible_index].miner.sandrone.movable.x, biomes[options.visible_index].miner.sandrone.movable.y, ui_sandrone(&biomes[options.visible_index].miner.sandrone, options), &mut view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy),
    SandroneState::Unconstructed => {}
    SandroneState::WaitingForWater => {}
  }

  // This is used to compute the position of the magic wall stars after they explode
  fn pump(post: bool, px: f32, py: f32, dd: f32) -> (i32, i32) {
    if !post { return (px as i32, py as i32); }

    // Now compute where it would be n ticks after the castle completes
    // Assume a starting vector from 0,0 to px,py and extrapolate the
    // current position on that vector when moving away from 0,0
    if px.abs() < py.abs() {
      let ratio = (px / py).abs();
      let qx = (px.signum() * dd * ratio) + px;
      let qy = (py.signum() * dd * 1.000) + py;
      return (qx as i32,  qy as i32);
    } else {
      let ratio = (py / px).abs();
      let qx = (px.signum() * dd * 1.000) + px;
      let qy = (py.signum() * dd * ratio) + py;
      return (qx as i32,  qy as i32);
    }
  }

  let a = '🍁';
  let b = '🌟';
  let speed = 0.4;
  let time_since_castle_complete = (ticks0 - biomes[options.visible_index].miner.sandrone.post_castle) as f32;
  let dd = time_since_castle_complete * speed;

  let lifted = biomes[options.visible_index].miner.sandrone.air_lifted;
  let post = biomes[options.visible_index].miner.sandrone.post_castle > 0;
  if lifted || (post && time_since_castle_complete < 50.0) {
    // Paint the castle rectangle. Once the castle is finished, the magic wall "explodes"
    for i in biomes[options.visible_index].miner.sandrone.expansion_min_x-1..biomes[options.visible_index].miner.sandrone.expansion_max_x+1 {
      if lifted || i % 3 == 0 {
        let p = pump(post, i as f32, (biomes[options.visible_index].miner.sandrone.expansion_min_y - 1) as f32, dd);
        paint_maybe(p.0, p.1, if ticks0 % 2 == 1 {a.to_string() } else {b.to_string()}, &mut view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);

        let q = pump(post, i as f32, (biomes[options.visible_index].miner.sandrone.expansion_max_y + 1) as f32, dd);
        paint_maybe(q.0, q.1, if ticks0 % 2 == 1 {a.to_string() } else {b.to_string()}, &mut view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);
      }
    }
    for j in biomes[options.visible_index].miner.sandrone.expansion_min_y-1..biomes[options.visible_index].miner.sandrone.expansion_max_y + 2 {
      if lifted || j % 3 == 0 {
        let p = pump(post, (biomes[options.visible_index].miner.sandrone.expansion_min_x - 1) as f32, j as f32, dd);
        paint_maybe(p.0, p.1, if ticks0 % 2 == 1 {a.to_string() } else {b.to_string()}, &mut view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);
        let q = pump(post, (biomes[options.visible_index].miner.sandrone.expansion_max_x + 1) as f32, j as f32, dd);
        paint_maybe(q.0, q.1, if ticks0 % 2 == 1 {a.to_string() } else {b.to_string()}, &mut view, viewport_offset_x, viewport_offset_y, viewport_size_w, viewport_size_h, vox, voy);
      }
    }
  }

  // World is finished now.
  // For web view, wrap all cells in a span to fix their width/height and prevent sub-pixel magic.

  if options.html_mode { // Could do this with a macro but why, :shrug:
    for y in 0..wv_height+1 {
      let line = &mut view[y];
      let len = line.len();
      for x in 0..wv_width {
        line[x] = format!("<span class='cell'>{}</span>", line[x]);
      }
      // Wrap the world view in another span
      line[0] = format!("<span class='world-line'>{}", line[0]);
      line[len - 1] = format!("{}</span>", line[len - 1]);
    }
  }

  // Draw UI

  // Offsets for the UI. Start at the top of the map to the right of it.
  // let uox = (wv_margin.3 + wv_border.3 + viewport_size_w + wv_border.1 + wv_margin.1 + 1) as i32;
  // let uoy = 0; // Just the top.

  let vlen = view.len();

  // Append each line to the map
  view[1].push(format!(" Gene mutation rate: {}%  Slot mutation rate: {}%   Miner batch size: {}   Reset rate: {: <120}", options.mutation_rate_genes, options.mutation_rate_slots, options.batch_size, options.reset_rate).to_string());
  view[2].push(format!(" {: <150}", best_miner_str));
  view[3].push(format!(" {: <150}", hmap_str));
  view[4].push(format!(" Batch tick: {} Decay interval: {} Decay rate: {} Current decay value: {: <100}", state.batch_ticks, options.cost_increase_interval, options.cost_increase_rate, state.cost_increase_value));
  view[5].push(std::iter::repeat(' ').take(150).collect::<String>());
  view[6].push(format!(" Miner {}; {: <150}", options.visible_index, ' '));
  view[7].push(std::iter::repeat(' ').take(143).collect::<String>());
  view[8].push(format!("   {: <150}", biomes[options.visible_index].miner.helix));
  view[9].push(format!("   XY: {: >4}, {: <10} {: <45} Points: {: <10} Energy {: <10}", biomes[options.visible_index].miner.movable.x, biomes[options.visible_index].miner.movable.y, progress_bar(30, biomes[options.visible_index].miner.movable.now_energy, biomes[options.visible_index].miner.movable.init_energy, true), get_points(&biomes[options.visible_index].miner.meta.inventory), biomes[options.visible_index].miner.movable.now_energy.round()).to_string());
  view[10].push(format!("   Inventory:   {: <100}", ui_inventory(&biomes[options.visible_index].miner.meta.inventory, options)));
  let t = helix_serialize(&biomes[options.visible_index].miner.helix);
  view[11].push(add_fg_color_with_reset(&format!("   Current miner code: `{}`", serde_json::to_string(&t).unwrap()).to_string(), COLOR_GREY, options));
  view[12].push(std::iter::repeat(' ').take(100).collect::<String>());

  let so = 13;
  for n in 0..biomes[options.visible_index].miner.slots.len() {
    let slot: &Slottable = &biomes[options.visible_index].miner.slots[n];
    let (head, progress, tail) = match slot.kind {
      SlotKind::BrokenGps => ui_slot_broken_gps(slot),
      SlotKind::Drill => ui_slot_drill(slot),
      SlotKind::DroneLauncher => ui_slot_drone_launcher(slot, &biomes[options.visible_index].miner.drones[slot.nth as usize]),
      SlotKind::Emptiness => ui_slot_emptiness(slot),
      SlotKind::EnergyCell => ui_slot_energy_cell(slot),
      SlotKind::Hammer => ui_slot_hammer(slot),
      SlotKind::JacksCompass => ui_slot_jacks_compass(slot),
      SlotKind::Magnet => ui_slot_magnet(slot),
      SlotKind::PurityScanner => ui_slot_purity_scanner(slot),
      SlotKind::RandomStart => panic!("Running miners should not get the RandomStart slot"),
      SlotKind::Sandrone => ui_slot_sandrone(slot, &biomes[options.visible_index].miner.sandrone, biomes[options.visible_index].miner.meta.inventory.sand),
      SlotKind::Windrone => ui_slot_windrone(slot, &biomes[options.visible_index].miner.windrone, biomes[options.visible_index].miner.meta.inventory.wind),
    };
    view[so + n].push(format!(" {: <20} {: <40} {: <70}", head, progress, tail).to_string());
  }

  let input_hint_lines = 7;

  for y in so + biomes[options.visible_index].miner.slots.len()..vlen - input_hint_lines {
    view[y].push(std::iter::repeat(' ').take(100).collect::<String>());
  }

  view[vlen - 6].push(format!(" Keys: toggle visual: v⏎   save and quite: q⏎  speed [{}]  faster: -⏎   slower: +⏎   return-stepper: x⏎ {: <50}", options.speed, ' '));
  view[vlen - 5].push(format!("       gene mutation rate [{}]  up: o⏎   up 5: oo⏎   down: p⏎   down 5: pp⏎ {: <50}", options.mutation_rate_genes, ' '));
  view[vlen - 4].push(format!("       slot mutation rate [{}]  up: l⏎   up 5: ll⏎   down: k⏎   down 5: kk⏎ {: <50}", options.mutation_rate_slots, ' '));
  view[vlen - 3].push(format!("       batch size [{}]  up: m⏎   down: n⏎   restart with random helix: r⏎   restart from best: b⏎ {: <50}", options.batch_size, ' '));
  view[vlen - 2].push(format!("       mutate [{}]: g⏎   auto reset [{}] after [{}] miners: t⏎ {: <50}", if options.mutate_from_best { "overall best" } else { "last winner" }, if options.reset_after_noop { "after noop" } else { "regardless" }, options.reset_rate, ' '));
  view[vlen - 1].push(format!("       arrow keys move viewport. c: center. f: toggle auto-follow. h: home"));

  if options.html_mode { // Could do this with a macro but why, :shrug:
    for y in 0..view.len() {
      let line = &mut view[y];
      let len = line.len();
      // Wrap each line in a div so we can control its height proper
      line[0] = format!("<div class='view-line'>{}", line[0]);
      line[len - 1] = format!("{}</div>", line[len - 1]);
    }
  }

  let frame = view.iter().map(|row| {
    return row.join("");
  }).collect::<Vec<String>>().join(if options.html_mode{ "" } else { "\n" });


  // for row in view.iter() {
  //   let border_top_str: String = row.join("");
  //   println!("{}", border_top_str);
  // }

  return frame;
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
}

pub fn create_unvisited_cell(tile: Tile, pickup: Pickup, tile_value: u32, pickup_value: u32) -> Cell {
  return Cell { tile, pickup, tile_value, pickup_value, visited: 0 };
}

pub fn create_visited_cell(tile: Tile, pickup: Pickup, tile_value: u32, pickup_value: u32, visited: u32) -> Cell {
  return Cell { tile, pickup, tile_value, pickup_value, visited };
}

pub fn get_cell_stuff_at(options: &Options, world: &World, wx: i32, wy: i32) -> (Tile, Pickup, u32, u32) {
  // Return tile, pickup, value, visited.

  // wx/wy should be world coordinates

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
pub fn get_cell_pickup_at(options: &Options, world: &World, wx: i32, wy: i32) -> Pickup {
  // wx/wy should be world coordinates

  // Is the cell explicitly stored in the world right now? If not then use the procedure.
  if wx < world.min_x || wx > world.max_x || wy < world.min_y || wy > world.max_y {
    // OOB. Use generated value
    return generate_cell(options, wx, wy).pickup;
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

  return world.tiles[ay as usize][ax as usize].pickup;
}

pub fn get_cell_value_at(options: &Options, world: &World, wx: i32, wy: i32) -> u32 {
  // wx/wy should be world coordinates

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
pub fn get_cell_tile_value_at(_options: &Options, world: &World, wx: i32, wy: i32) -> u32 {
  let ax = world.min_x.abs() + wx;
  let ay = world.min_y.abs() + wy;

  assert_arr_xy_in_world(world, wx, wy, ax as usize, ay as usize);

  return world.tiles[ay as usize][ax as usize].tile_value;
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

pub fn oob(x: i32, y: i32, minx: i32, miny: i32, maxx: i32, maxy: i32) -> bool {
  return x < minx || x > maxx || y < miny || y > maxy;
}
