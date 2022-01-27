use std::collections::VecDeque;
use super::movable::*;
use crate::tile::*;
use crate::pickup::*;
use crate::options::*;
use crate::world::*;
use super::slottable::*;
use super::icons::*;
use super::color::*;
use super::values::*;

/*

The hydrone moves to 0x0 (guaranteed to be an empty cell and in view) and starts creating a structure of guided push cells.
The arrows point towards the 0x0 but always prefers to point to an empty cell. Only if that's not possible will it point to another push cell.
If the miner enters an push cell, it gets forced out that way after one tick. Potentially this gives a bonus, or perhaps empty cells that get pointed grow bonuses or something.
The hydrone only expands to known territory (withing min/max of world tiles).
Once the hydrone moved to 0x0 and built its first push cell, it only moves from push cell to empty cells or other push cells.
Perhaps it can only create more tiles as long as there is water?

 */

#[derive(Debug)]
pub enum HydroneState {
  // Not yet built
  Unconstructed,
  // Waiting for enough wind pickups
  WaitingForWater,
  // Moving to the 0x0 cell
  MovingToOrigin,
  // Make a move
  MovingToNeighborCell,
  // Converting an empty cell to a push cell
  BuildingArrowCell,
  // On its way to pick up the miner
  PickingUpMiner,
  // On its way to deliver the miner to 0x0
  DeliveringMiner,
}

#[derive(Debug)]
pub struct Hydrone {
  // A hydrone is a drone that works on water. It builds water ways that force the miner in a
  // particular direction and sculpts tracks that attempt to guide the miner into a farm of sorts.
  // Other drones can build things in this farm and we can start an economy this way.

  pub state: HydroneState,
  // TBD but right now a sort of freeform desc of what the hydrone is doing
  pub status_desc: String,
  // Each drone has its own x, y, direction, and energy
  pub movable: Movable,
  // Generated push tiles. They'll need to be ticked and we don't want to have to search for them.
  pub push_tiles: Vec<(i32, i32)>,
  // This contains the rotating order of which direction to check first. Prevents simple/short infinite loops.
  pub direction_cycle: VecDeque<(i32, i32)>,

  // Have we air lifted the miner back to 0x0 yet?
  pub air_lifted: bool,

  tmp: u32,
  // Remember the last direction for backtracking over push tiles
  last_dx: i32,
  last_dy: i32,
  pub seeking: bool,
  pub backtracking: bool,
  pub found_end: bool,
}

pub fn create_hydrone() -> Hydrone {
  let mut dc: VecDeque<(i32, i32)> = VecDeque::new();
  dc.push_back((-1, 0)); // <
  dc.push_back((0, -1)); // ^
  dc.push_back((1, 0));  // >
  dc.push_back((0, 1));  // v

  return Hydrone {
    state: HydroneState::Unconstructed,
    status_desc: "Idle. Waiting for enough water...".to_string(),
    movable: Movable {
      what: WHAT_HYDRONE,
      x: 0,
      y: 0,
      dir: Direction::Up,
      now_energy: 0.0,
      init_energy: 0.0,
      history: vec!((0,0)),
      unique: vec!((0,0)),
      disabled: false,
    },
    push_tiles: vec!(),
    direction_cycle: dc,
    air_lifted: false,
    tmp: 20,
    last_dx: 1,
    last_dy: 0,
    seeking: true,
    backtracking: false,
    found_end: false,
  };
}

pub fn set_hydrone_state(hydrone: &mut Hydrone, state: HydroneState) {
  hydrone.status_desc = match state {
    HydroneState::Unconstructed => panic!("A hydrone does not deconstruct"),
    HydroneState::WaitingForWater => format!("Waiting for enough water to take off..."),
    HydroneState::MovingToOrigin => format!("Moving to cell 0x0..."),
    HydroneState::MovingToNeighborCell => format!("Moving to neighbor cell"),
    HydroneState::BuildingArrowCell => format!("Building a push cell..."),
    HydroneState::PickingUpMiner => format!("Picking up miner..."),
    HydroneState::DeliveringMiner => format!("Delivering miner to origin..."),
  };
  hydrone.state = state;
}

pub fn tick_hydrone(_hydrone_slot: &mut Slottable, hydrone: &mut Hydrone, mminermovable: &mut Movable, water: u32, world: &mut World, options: &mut Options, biome_index: usize) {
  let mx = mminermovable.x;
  let my = mminermovable.y;

  match hydrone.state {
    HydroneState::Unconstructed => {

    }
    HydroneState::WaitingForWater => {
      if water >= 10 {
        set_hydrone_state(hydrone, HydroneState::MovingToOrigin);
        options.visual = true;
      }
    }
    HydroneState::MovingToOrigin => {
      // Find a way to get closer to 0x0
      // TODO: do we take the windrone approach of ghosting or do we make it move in the real world or maybe drill through anything it encounters in its path towards 0x0 or smth?

      if move_hydrone_towards(hydrone, 0, 0, biome_index) {
        println!("Convert 0x0 to a push tile and start generating an arrow structure");
        set_cell_tile_at(options, world, 0, 0, Tile::Push);
        hydrone.push_tiles.push((0, 0));
        set_hydrone_state(hydrone, HydroneState::MovingToNeighborCell);

        if !hydrone.air_lifted && hydrone.push_tiles.len() > 1000 {
          println!("Shutting down the hydrone");
          set_hydrone_state(hydrone, HydroneState::PickingUpMiner);
          options.visual = true;
          hydrone.air_lifted = true;
        }

        // if hydrone.push_tiles.len() > 1000 {
        //   set_hydrone_state(hydrone, HydroneState::Unconstructed);
        //   return;
        // }
      }
    }
    HydroneState::MovingToNeighborCell => {
      // Pick a neighbor cell according to an algorithm and move towards it
      // Basically look at all horizontal and vertical neighbors. Consider all empty and push cells. Then move to one of them with some rule.

      // There are some grid based expansion rules for push tiles;
      // - The cell must be empty, and
      // - This cell has at most one horizontal or vertical touching push cell, and
      // - This cell has at most two diagonally touching push cell, and
      // - All neighbouring push cells must share an axis
      //
      // Some examples:
      //   ╔vv   vvv    vvv   x═v     vx═    vxv    vvv    v║v    ╔═v    ╔═v    ╔══
      //   ║vv   ║x═    ║xv   ║xv     ║xv    ║xv    ═x═    vxv    ║xv    ║xx    ║xx
      //   ╚vv   vvv    vx═   ║vv     vxv    vx═    vvv    v║v    ╚vv    ╚═v    ╚═v
      //
      // I believe that this way there can never be a dead end and at least two open entry/exit sides

      let mut fx = hydrone.movable.x;
      let mut fy = hydrone.movable.y;
      let mut tx = fx;
      let mut ty = fy;
      let mut found = false;

      // The next step is determined in two phases;
      // - check whether there is any neighbouring cell that can be converted to push
      //   - if there is any, pick the first one in direction_cycle order
      // - when there is no neighbor cell eligible to convert to push,
      //   - take the first neighbouring push cell in direction_cycle order
      // - when there is neither; do nothing and wait (I don't think this is possible in our current setup since we start at 0,0 which is where the miner started and it craft a path out of 0,0)

      while hydrone.direction_cycle[1].0 != hydrone.last_dx || hydrone.direction_cycle[1].1 != hydrone.last_dy {
        hydrone.direction_cycle.rotate_right(1);
        // println!("  Rotated right: Cycle now: {:?}", hydrone.direction_cycle);
      }

      if !hydrone.found_end && (hydrone.seeking || !hydrone.backtracking) {
        for (dx, dy) in hydrone.direction_cycle.to_owned() {
          // println!("- Testing {},{} if hydrone can convert {},{} :: {:?}", dx, dy, fx + dx, fy + dy, get_cell_tile_at(options, world, fx + dx, fy + dy));
          if hydrone.tmp > 0 && can_convert_tile_to_push(options, world, fx + dx, fy + dy, dx, dy) {
            found = true;
            // hydrone.direction_cycle.rotate_left(1);
            hydrone.movable.x = fx + dx;
            hydrone.movable.y = fy + dy;
            // move_hydrone_towards(hydrone, fx + dx, fy + dy, biome_index);
            set_hydrone_state(hydrone, HydroneState::BuildingArrowCell);
            tx += dx;
            ty += dy;
            hydrone.last_dx = dx;
            hydrone.last_dy = dy;
            hydrone.seeking = false; // Expanded at least once since last return to origin
            // hydrone.tmp -= 1;
            // println!("Moved to empty cell to convert, from {},{} to {},{}", fx, fy, tx, ty);
            break;
          }
        }
      }
      if !found {
        if !hydrone.seeking {
          // Found a dead end while having expanded at least once since reaching origin. Move back.
          // hydrone.backtracking = true;
          hydrone.seeking = true;
          hydrone.movable.x = 0;
          hydrone.movable.y = 0;
          // fx = 0;
          // fy = 0;

          if !hydrone.air_lifted && hydrone.push_tiles.len() > 250 {
            println!("Going to pick up miner...");
            set_hydrone_state(hydrone, HydroneState::PickingUpMiner);
            options.visual = true;
            hydrone.air_lifted = true;
          }
        }

        for (dx, dy) in hydrone.direction_cycle.to_owned() {
          // println!("  - Testing d {},{} , is cell {},{} is push cell? :: {:?}", dx, dy, fx + dx, fy + dy, get_cell_tile_at(options, world, fx + dx, fy + dy));
          if is_push_cell(options, world, fx + dx, fy + dy) {
            // move_hydrone_towards(hydrone, fx + dx, fy + dy, biome_index);
            hydrone.movable.x = fx + dx;
            hydrone.movable.y = fy + dy;
            // set_hydrone_state(hydrone, HydroneState::BuildingArrowCell);
            tx += dx;
            ty += dy;
            if tx == 0 && ty == 0 {
              hydrone.backtracking = false;
              // Ignore dead ends until having expanded at least once.
              hydrone.seeking = true;

              if !hydrone.air_lifted && hydrone.push_tiles.len() > 500 {
                println!("Going to pick up miner...");
                set_hydrone_state(hydrone, HydroneState::PickingUpMiner);
                options.visual = true;
                hydrone.air_lifted = true;
              }
            }
            hydrone.last_dx = dx;
            hydrone.last_dy = dy;
            // println!("Walking back over push tiles, from {},{} to {},{}", fx, fy, tx, ty);
            break;
          // } else {
          //   hydrone.direction_cycle.rotate_left(1);
          }
        }
      }

      hydrone.movable.x = tx;
      hydrone.movable.y = ty;

    }
    HydroneState::BuildingArrowCell => {
      // Convert the current cell, which ought to be empty, to a push cell
      // println!("Convert {}x{} to a push tile", hydrone.movable.x, hydrone.movable.y);
      set_cell_tile_at(options, world, hydrone.movable.x, hydrone.movable.y, Tile::Push);
      hydrone.push_tiles.push((hydrone.movable.x, hydrone.movable.y));
      set_hydrone_state(hydrone, HydroneState::MovingToNeighborCell);
    }
    HydroneState::PickingUpMiner => {
      // Home in on the miner. Whereever it is.
      println!("HydroneState::PickingUpMiner at {}x{}", hydrone.movable.x, hydrone.movable.y);
      if move_hydrone_towards(hydrone, mx, my, biome_index) {
        println!("  gottem!");
        hydrone.state = HydroneState::DeliveringMiner;
        hydrone.status_desc = format!("Delivering miner to origin...");
      }
    }
    HydroneState::DeliveringMiner => {
      println!("HydroneState::DeliveringMiner at {}x{}", hydrone.movable.x, hydrone.movable.y);
      // Home in on the miner. Whereever it is.
      if move_hydrone_towards(hydrone, 0, 0, biome_index) {
        hydrone.state = HydroneState::MovingToNeighborCell;
        hydrone.status_desc = format!("Idle. Waiting for enough wind...");
        mminermovable.disabled = false;

        // println!("Return to move enabled. Press ⏎ to tick forward. Press x⏎ to exit this mode.");
        // options.return_to_move = true;
        options.visual = true;

        println!("Putting hydrone in permanent seek mode");
        hydrone.found_end = true;
      }
      mminermovable.x = hydrone.movable.x;
      mminermovable.y = hydrone.movable.y;
    }
  }
}

pub fn ui_hydrone(hydrone: &Hydrone) -> String {
  if hydrone.backtracking {
    return add_fg_color_with_reset(&format!("{}", ICON_HYDRONE), COLOR_DARK_GREEN);
  }

  return add_fg_color_with_reset(&format!("{}", ICON_HYDRONE), COLOR_DARK_RED);
}

fn can_empty_cell_be_push_cell(options: &Options, world: &World, tx: i32, ty: i32, dx: i32, dy: i32) -> bool {
  // An empty cell can become a push cell iif all the only neighboring push cells all share one axis
  // It is assumed this is part of a hydrone move check, in which case we can assume that the
  // origin (where the hydrone currently is) is already a push cell. In that case only the diagonal
  // cells that border the origin can be push cells (so we can ignore them) and we have to assert
  // that the other 5 surrounding cells are not push cells.

  if dx == -1 {
    assert_eq!(dy, 0);
    // x-1 is moving to the left so we must check fx-1,fy-1 fx-2,fy-1 fx-2,fy-2 fx-2,fy-3 fx-1,fy-3
    !matches!(get_cell_tile_at(options, world, tx,   ty-1), Tile::Push | Tile::Impassible) && // up
    !matches!(get_cell_tile_at(options, world, tx-1, ty-1), Tile::Push | Tile::Impassible) && // up-left
    !matches!(get_cell_tile_at(options, world, tx-1, ty), Tile::Push | Tile::Impassible) && // left
    !matches!(get_cell_tile_at(options, world, tx-1, ty+1), Tile::Push | Tile::Impassible) && // down-left
    !matches!(get_cell_tile_at(options, world, tx,   ty+1), Tile::Push | Tile::Impassible) // down
  } else if dx == 1 {
    assert_eq!(dy, 0);
    // x+1 is moving to the right
    !matches!(get_cell_tile_at(options, world, tx,   ty-1), Tile::Push | Tile::Impassible) && // up
    !matches!(get_cell_tile_at(options, world, tx+1, ty-1), Tile::Push | Tile::Impassible) && // up-right
    !matches!(get_cell_tile_at(options, world, tx+1, ty), Tile::Push | Tile::Impassible) && // right
    !matches!(get_cell_tile_at(options, world, tx+1, ty+1), Tile::Push | Tile::Impassible) && // down-right
    !matches!(get_cell_tile_at(options, world, tx,   ty+1), Tile::Push | Tile::Impassible) // down
  } else if dy == -1 {
    assert_eq!(dx, 0);
    // y-1 is moving up
    !matches!(get_cell_tile_at(options, world, tx-1, ty), Tile::Push | Tile::Impassible) && // left
    !matches!(get_cell_tile_at(options, world, tx-1, ty-1), Tile::Push | Tile::Impassible) && // up-left
    !matches!(get_cell_tile_at(options, world, tx,   ty-1), Tile::Push | Tile::Impassible) && // up
    !matches!(get_cell_tile_at(options, world, tx+1, ty-1), Tile::Push | Tile::Impassible) && // up-right
    !matches!(get_cell_tile_at(options, world, tx+1, ty), Tile::Push | Tile::Impassible) // right
  } else {
    assert_eq!(dy, 1);
    assert_eq!(dx, 0);
    // y-1 is moving up
    !matches!(get_cell_tile_at(options, world, tx-1, ty), Tile::Push | Tile::Impassible) && // left
    !matches!(get_cell_tile_at(options, world, tx-1, ty+1), Tile::Push | Tile::Impassible) && // down-left
    !matches!(get_cell_tile_at(options, world, tx,   ty+1), Tile::Push | Tile::Impassible) && // down
    !matches!(get_cell_tile_at(options, world, tx+1, ty+1), Tile::Push | Tile::Impassible) && // down-right
    !matches!(get_cell_tile_at(options, world, tx+1, ty), Tile::Push | Tile::Impassible) // right
  }
}

fn hydrone_can_move_to(options: &Options, world: &World, tx: i32, ty: i32, dx: i32, dy: i32) -> bool {
  // A hydrone can always move to another push tile or to a tile that is eligible to come a push tile
  // println!("hydrone_can_move_to({}, {}) -> {:?}", tx, ty, get_cell_tile_at(options, world, tx, ty));
  return match get_cell_tile_at(options, world, tx, ty) {
    Tile::Push => true,
    Tile::Empty => can_empty_cell_be_push_cell(options, world, tx, ty, dx, dy),
    // Can not move to any other kind of cell
    _ => false,
  };
}

fn can_convert_tile_to_push(options: &Options, world: &World, tx: i32, ty: i32, dx: i32, dy: i32) -> bool {
  // A cell can be converted to a push tile when it is empty and when it borders horizontally
  // or vertically to exactly one push/impassable cell and when all diagonal cells that are push cells are also
  // bordering that one cell (share the same axis). The origin is implied to be a push cell.
  return matches!(get_cell_tile_at(options, world, tx, ty), Tile::Empty) && matches!(get_cell_stuff_at(options, world, tx, ty).1, Pickup::Nothing) && can_empty_cell_be_push_cell(options, world, tx, ty, dx, dy);
}

fn is_push_cell(options: &Options, world: &World, tx: i32, ty: i32) -> bool {
  return matches!(get_cell_tile_at(options, world, tx, ty), Tile::Push);
}

fn move_hydrone_towards(hydrone: &mut Hydrone, to_x: i32, to_y: i32, _biome_index: usize) -> bool {
  let bx = hydrone.movable.x;
  let by = hydrone.movable.y;

  if bx == to_x && by == to_y {
    return true;
  } else {
    // Now move closer to the closest target
    let x1 = to_x as f64;
    let x2 = bx as f64;
    let y1 = to_y as f64;
    let y2 = by as f64;

    // Hydrone still on its way to a target...
    let dx = x1 - x2;
    let dy = y1 - y2;
    if dx.abs() > dy.abs() {
      hydrone.movable.x += if dx < 0.0 { -1 } else { 1 };
    } else {
      hydrone.movable.y += if dy < 0.0 { -1 } else { 1 };
    }
    return false;
  }
}
