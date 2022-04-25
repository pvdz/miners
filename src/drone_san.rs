use std::collections::VecDeque;

use super::movable::*;
use super::{bridge};
use super::tile::*;
use super::pickup::*;
use super::options::*;
use super::world::*;
use super::icons::*;
use super::color::*;
use super::values::*;
use super::biome::*;
use super::miner::*;
use super::app_state::*;

/*

The sandrone moves to 0x0 (guaranteed to be an empty cell and in view) and starts creating a structure of guided castle walls (building sand castle, hur hur).
The arrows point towards the 0x0 but always prefers to point to an empty cell. Only if that's not possible will it point to another push cell.
If the miner enters an push cell, it gets forced out that way after one tick. Potentially this gives a bonus, or perhaps empty cells that get pointed grow bonuses or something.
TODO: revisit the rest of the desc
The sandrone only expands to known territory (withing min/max of world tiles).
Once the sandrone moved to 0x0 and built its first push cell, it only moves from push cell to empty cells or other push cells.
Perhaps it can only create more tiles as long as there is sand?
*/

#[derive(Debug)]
pub enum SandroneState {
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
  // Post castle building, the sandrone now exclusively moves around on impassible tiles and pushes.
  Redecorating,

}

#[derive(Debug)]
pub struct Sandrone {
  // A sandrone is a drone that works on sand. It builds sand castles.

  pub state: SandroneState,
  // TBD but right now a sort of freeform desc of what the sandrone is doing
  pub status_desc: String,
  // Each drone has its own x, y, direction, and energy
  pub movable: Movable,
  // Generated tiles. They'll need to be ticked and we don't want to have to search for them.
  pub push_tiles: Vec<(i32, i32)>,
  pub impassable_tiles: Vec<(i32, i32)>,
  // This contains the rotating order of which direction to check first. Prevents simple/short infinite loops.
  pub direction_cycle: VecDeque<(i32, i32)>,

  // Have we air lifted the miner back to 0x0 yet?
  pub air_lifting: bool,
  pub air_lifted: bool,
  pub post_castle: u32, // Tick at which castle completed

  tmp: u32,
  // Remember the last direction for backtracking over push tiles
  last_dx: i32,
  last_dy: i32,
  pub seeking: bool,
  pub backtracking: bool,
  pub found_end: bool,

  last_expansion_x: i32,
  last_expansion_y: i32,

  // Track seen cell exits
  pub last_empty_castle_exit_x: i32,
  pub last_empty_castle_exit_y: i32,

  // Maintain a rectangle of the size of the castle
  pub expansion_min_x: i32,
  pub expansion_min_y: i32,
  pub expansion_max_x: i32,
  pub expansion_max_y: i32,
  // Did the hydrone add an impassable tile to the end of a
  pub plugged_a_hole: bool,
}

pub fn create_sandrone() -> Sandrone {
  let mut dc: VecDeque<(i32, i32)> = VecDeque::new();
  dc.push_back((-1, 0)); // <
  dc.push_back((0, -1)); // ^
  dc.push_back((1, 0));  // >
  dc.push_back((0, 1));  // v

  return Sandrone {
    state: SandroneState::Unconstructed,
    status_desc: "Idle. Waiting for enough water...".to_string(),
    movable: Movable {
      what: WHAT_SANDRONE,
      x: 0,
      y: 0,
      dir: Direction::Up,
      now_energy: 0.0,
      init_energy: 0.0,
      disabled: false,
    },
    push_tiles: vec!(),
    impassable_tiles: vec!(),
    direction_cycle: dc,
    air_lifting: false,
    air_lifted: false,
    post_castle: 0,
    tmp: 20,
    last_dx: 1,
    last_dy: 0,
    seeking: true,
    backtracking: false,
    found_end: false,
    last_expansion_x: 0,
    last_expansion_y: 0,
    last_empty_castle_exit_x: 0,
    last_empty_castle_exit_y: 0,
    expansion_min_x: 0,
    expansion_min_y: 0,
    expansion_max_x: 0,
    expansion_max_y: 0,
    plugged_a_hole: false,
  };
}

pub fn set_sandrone_state(sandrone: &mut Sandrone, state: SandroneState) {
  sandrone.status_desc = match state {
    SandroneState::Unconstructed => panic!("A sandrone does not deconstruct"),
    SandroneState::WaitingForWater => format!("Waiting for enough water to take off..."),
    SandroneState::MovingToOrigin => format!("Moving to cell 0x0..."),
    SandroneState::MovingToNeighborCell => format!("Moving to neighbor cell"),
    SandroneState::BuildingArrowCell => format!("Building a push cell..."),
    SandroneState::PickingUpMiner => format!("Picking up miner..."),
    SandroneState::DeliveringMiner => format!("Delivering miner to origin..."),
    SandroneState::Redecorating => format!("Redecorating the castle"),
  };
  sandrone.state = state;
}

pub fn tick_sandrone(options: &mut Options, _state: &mut AppState, biome: &mut Biome, _slot_index: usize) {
  let sandrone: &mut Sandrone = &mut biome.miner.sandrone;

  match sandrone.state {
    SandroneState::Unconstructed => {

    }
    SandroneState::WaitingForWater => {
      if biome.miner.meta.inventory.sand >= 10 {
        set_sandrone_state(sandrone, SandroneState::MovingToOrigin);
        // start of building a sand castle
      }
    }
    SandroneState::MovingToOrigin => {
      // Find a way to get closer to 0x0
      // TODO: do we take the windrone approach of ghosting or do we make it move in the real world or maybe drill through anything it encounters in its path towards 0x0 or smth?

      if move_sandrone_towards(sandrone, 0, 0) {
        // println!("Convert 0x0 to a push tile and start generating an sand castle");
        set_cell_tile_at(options, &mut biome.world, 0, 0, Tile::Push);
        sandrone.push_tiles.push((0, 0));
        set_sandrone_state(sandrone, SandroneState::MovingToNeighborCell);

        if !sandrone.air_lifted && !sandrone.post_castle > 0 && !sandrone.air_lifting && sandrone.push_tiles.len() > 1000 {
          set_sandrone_state(sandrone, SandroneState::PickingUpMiner);
          // bridge::focus_weak(options, biome.index, biome.miner.meta.phase, "shutting down sandrone after building enough tiles");
          sandrone.air_lifting = true;
          biome.miner.meta.phase = Phase::FinishedCastleWalls_3;
        }

        // if sandrone.push_tiles.len() > 1000 {
        //   set_sandrone_state(sandrone, SandroneState::Unconstructed);
        //   return;
        // }
      }
    }
    SandroneState::MovingToNeighborCell => {
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

      let fx = sandrone.movable.x;
      let fy = sandrone.movable.y;
      let mut tx = fx;
      let mut ty = fy;
      let mut found = false;

      if sandrone.post_castle > 0 {
        // Move to a neighboring Impassable cell. There must be multiple (inc dia). Juts pick one.
        // Then move like normal movables except the sandrone can only move over Impassible tiles.
        // The sandrone will now be redecorating the castle.
        set_sandrone_state(sandrone, SandroneState::Redecorating);
        biome.miner.meta.phase = Phase::FilledCastle_6;
        biome.miner.meta.dying_since = biome.ticks;

        // With the current rules, it is not possible to create a 2x2 of castle walls. So
        // checking any quadrant should suffice to guarantee at least one available position.

        if matches!(get_cell_tile_at(options, &biome.world, fx, fy - 1), Tile::Soil) {
          // sandrone.movable.x = fx;
          sandrone.movable.y = fy - 1;
        } else if matches!(get_cell_tile_at(options, &biome.world, fx - 1, fy - 1), Tile::Soil) {
          sandrone.movable.x = fx + 1;
          sandrone.movable.y = fy + 1;
        } else if matches!(get_cell_tile_at(options, &biome.world, fx - 1, fy), Tile::Soil) {
          sandrone.movable.x = fx + 1;
          // sandrone.movable.y = fy;
        } else {
          println!("Assumption does not hold; it is not sufficient to scan for a quadrant and guarantee an impassable tile. uups {:?} {:?} {:?}",
            get_cell_tile_at(options, &biome.world, fx, fy - 1),
            get_cell_tile_at(options, &biome.world, fx - 1, fy - 1),
            get_cell_tile_at(options, &biome.world, fx - 1, fy),
          );
          options.return_to_move = true;
        }

        // The sandrone is now in redecorator mode and it is on an Impassible tile. Time to go.
        return;
      }

      // The next step is determined in two phases;
      // - check whether there is any neighbouring cell that can be converted to push
      //   - if there is any, pick the first one in direction_cycle order
      // - when there is no neighbor cell eligible to convert to push,
      //   - take the first neighbouring push cell in direction_cycle order
      // - when there is neither; do nothing and wait (I don't think this is possible in our current setup since we start at 0,0 which is where the miner started and it craft a path out of 0,0)

      while sandrone.direction_cycle[1].0 != sandrone.last_dx || sandrone.direction_cycle[1].1 != sandrone.last_dy {
        sandrone.direction_cycle.rotate_right(1);
        // println!("  Rotated right: Cycle now: {:?}", sandrone.direction_cycle);
      }

      if biome.miner.meta.inventory.sand < 10 {
        sandrone.backtracking = true;
      } else if sandrone.last_expansion_x == sandrone.movable.x && sandrone.last_expansion_y == sandrone.movable.y {
        sandrone.backtracking = false;
      }

      if !sandrone.backtracking && biome.miner.meta.inventory.sand >= 10 && !sandrone.found_end {
        for (dx, dy) in sandrone.direction_cycle.to_owned() {
          // println!("- Testing {},{} if sandrone can convert {},{} :: {:?}", dx, dy, fx + dx, fy + dy, get_cell_tile_at(options, &biome.world, fx + dx, fy + dy));
          if sandrone.tmp > 0 && can_convert_tile_to_push(options, &biome.world, fx + dx, fy + dy, dx, dy, sandrone) {
            found = true;
            // sandrone.direction_cycle.rotate_left(1);
            sandrone.movable.x = fx + dx;
            sandrone.movable.y = fy + dy;
            // move_sandrone_towards(sandrone, fx + dx, fy + dy);
            set_sandrone_state(sandrone, SandroneState::BuildingArrowCell);
            tx += dx;
            ty += dy;
            sandrone.last_dx = dx;
            sandrone.last_dy = dy;
            sandrone.seeking = false; // Expanded at least once since last return to origin
            // sandrone.tmp -= 1;
            // println!("Moved to empty cell to convert, from {},{} to {},{}", fx, fy, tx, ty);
            break;
          }
        }
      }
      if !found {
        if !sandrone.seeking {
          // println!("dead end. last tile is {},{}", sandrone.last_expansion_x, sandrone.last_expansion_y);
          // Found a dead end while having expanded at least once since reaching origin. Move back.
          // sandrone.backtracking = true;
          sandrone.seeking = true;
          sandrone.movable.x = 0;
          sandrone.movable.y = 0;
          // fx = 0;
          // fy = 0;

          if !sandrone.air_lifted && !sandrone.post_castle > 0 && !sandrone.air_lifting && sandrone.push_tiles.len() > options.sandrone_pickup_count as usize{
            // println!("Going to pick up miner...");
            set_sandrone_state(sandrone, SandroneState::PickingUpMiner);
            sandrone.air_lifting = true;
            biome.miner.meta.phase = Phase::FinishedCastleWalls_3;
            // Make sure it doesn't branch before it's back at the end...
            sandrone.backtracking = true;

            sandrone.found_end = true;
          }
        }

        if sandrone.backtracking {
          if sandrone.last_expansion_x == sandrone.movable.x && sandrone.last_expansion_y == sandrone.movable.y {
            // sandrone.backtracking = false;
            sandrone.seeking = true;
          }
        }

        // Must plug at least one hole (fill single tile gap between Tile::Push and magic wall
        // with a Tile::Impassible) because otherwise the miner may end up running in circles.
        // I think this should not endanger locking up the miner (since that would mean blocking
        // an exit and the hole plugged here is between a push block and the wall). Additionally
        // due to the way the wall is positioned there should always be at least four such gaps;
        // one on each side. Usually there's many. Blocking the path once should prevent looping.
        if sandrone.air_lifted && !sandrone.plugged_a_hole {
          if fx - 1 == sandrone.expansion_min_x {
            // println!("sandrone setting impassible tile");
            set_cell_tile_at(options, &mut biome.world, fx - 1, fy, Tile::Impassible);
            sandrone.plugged_a_hole = true;
            sandrone.impassable_tiles.push((fx - 1, fy));
            // println!("Sandrone setting impassible at {}, {} minmax {},{} {},{}", fx - 1, fy, sandrone.expansion_min_x, sandrone.expansion_min_y, sandrone.expansion_max_x, sandrone.expansion_max_y);
          } else if fx + 1 == sandrone.expansion_max_x {
            // println!("sandrone setting impassible tile");
            set_cell_tile_at(options, &mut biome.world, fx + 1, fy, Tile::Impassible);
            sandrone.plugged_a_hole = true;
            sandrone.impassable_tiles.push((fx + 1, fy));
            // println!("Sandrone setting impassible at {}, {} minmax {},{} {},{}", fx + 1, fy, sandrone.expansion_min_x, sandrone.expansion_min_y, sandrone.expansion_max_x, sandrone.expansion_max_y);
          } else if fy - 1 == sandrone.expansion_min_y {
            // println!("sandrone setting impassible tile");
            set_cell_tile_at(options, &mut biome.world, fx, fy - 1, Tile::Impassible);
            sandrone.plugged_a_hole = true;
            sandrone.impassable_tiles.push((fx, fy - 1));
            // println!("Sandrone setting impassible at {}, {} minmax {},{} {},{}", fx, fy - 1, sandrone.expansion_min_x, sandrone.expansion_min_y, sandrone.expansion_max_x, sandrone.expansion_max_y);
          } else if fy + 1 == sandrone.expansion_max_y {
            // println!("sandrone setting impassible tile");
            set_cell_tile_at(options, &mut biome.world, fx, fy + 1, Tile::Impassible);
            sandrone.plugged_a_hole = true;
            sandrone.impassable_tiles.push((fx, fy + 1));
            // println!("Sandrone setting impassible at {}, {} minmax {},{} {},{}", fx, fy + 1, sandrone.expansion_min_x, sandrone.expansion_min_y, sandrone.expansion_max_x, sandrone.expansion_max_y);
          }
        }

        for (dx, dy) in sandrone.direction_cycle.to_owned() {
          // println!("  - Testing d {},{} , is cell {},{} is push cell? :: {:?}", dx, dy, fx + dx, fy + dy, get_cell_tile_at(options, &biome.world, fx + dx, fy + dy));
          if is_push_cell(options, &biome.world, fx + dx, fy + dy) {
            // move_sandrone_towards(sandrone, fx + dx, fy + dy);
            sandrone.movable.x = fx + dx;
            sandrone.movable.y = fy + dy;
            // set_sandrone_state(sandrone, SandroneState::BuildingArrowCell);
            tx += dx;
            ty += dy;
            sandrone.last_dx = dx;
            sandrone.last_dy = dy;
            // println!("Walking back over push tiles, from {},{} to {},{}", fx, fy, tx, ty);
            break;
          }
        }
      }

      sandrone.movable.x = tx;
      sandrone.movable.y = ty;

    }
    SandroneState::BuildingArrowCell => {
      // Convert the current cell, which ought to be empty, to a push cell
      // println!("Convert {}x{} to a push tile", sandrone.movable.x, sandrone.movable.y);

      ensure_cell_in_world(&mut biome.world, options, sandrone.movable.x, sandrone.movable.y);
      set_cell_tile_at(options, &mut biome.world, sandrone.movable.x, sandrone.movable.y, Tile::Push);
      sandrone.last_expansion_x = sandrone.movable.x;
      sandrone.last_expansion_y = sandrone.movable.y;

      // Set the expansion rectangle. This determines the magic walls. Make it one wider such that
      // the magic wall always has at least one space between the outer wall and the magic wall.
      // This way the sandrone never gets trapped in a pocket caused by the magic wall.
      // The left wall will not have this buffer which prevents the miner from running in circles.
      if sandrone.last_expansion_x <= sandrone.expansion_min_x {
        sandrone.expansion_min_x = sandrone.last_expansion_x - 1;
      } else if sandrone.last_expansion_x >= sandrone.expansion_max_x {
        sandrone.expansion_max_x = sandrone.last_expansion_x + 1;
      }
      if sandrone.last_expansion_y <= sandrone.expansion_min_y {
        sandrone.expansion_min_y = sandrone.last_expansion_y - 1;
      } else if sandrone.last_expansion_y >= sandrone.expansion_max_y {
        sandrone.expansion_max_y = sandrone.last_expansion_y + 1;
      }

      sandrone.push_tiles.push((sandrone.movable.x, sandrone.movable.y));
      set_sandrone_state(sandrone, SandroneState::MovingToNeighborCell);
      biome.miner.meta.inventory.sand = ((biome.miner.meta.inventory.sand as i32) - 10).max(0) as u32;

      if ((sandrone.expansion_max_x - sandrone.expansion_min_x) * (sandrone.expansion_max_y - sandrone.expansion_min_y)) as u32 > options.sandcastle_area_limit {
        // bridge::focus_weak(options, biome.index, biome.miner.meta.phase, "castle is big enough to bring in the miner");
        // println!("Castle area is now over 1000 cells. It is finished. Waiting for miner to complete filling.");
        sandrone.status_desc = format!("Idle. Waiting for completed castle.");
        sandrone.found_end = true;
        sandrone.seeking = false;
        sandrone.backtracking = false;
        if !sandrone.air_lifted && !sandrone.post_castle > 0 {
          set_sandrone_state(sandrone, SandroneState::PickingUpMiner);
          sandrone.air_lifting = true;
          biome.miner.meta.phase = Phase::FinishedCastleWalls_3;
          // Make sure it doesn't branch before it's back at the end...
          sandrone.backtracking = true;
        }
      }
    }
    SandroneState::PickingUpMiner => {
      // Home in on the miner. Whereever it is.
      // println!("SandroneState::PickingUpMiner at {}x{}", sandrone.movable.x, sandrone.movable.y);
      if move_sandrone_towards(sandrone, biome.miner.movable.x, biome.miner.movable.y) {
        // println!("  gottem!");
        biome.miner.meta.phase = Phase::PickedUpMiner_4;
        sandrone.state = SandroneState::DeliveringMiner;
        sandrone.status_desc = format!("Delivering miner to origin...");
      }
    }
    SandroneState::DeliveringMiner => {
      // println!("SandroneState::DeliveringMiner at {}x{}", sandrone.movable.x, sandrone.movable.y);
      // Home in on the miner. Whereever it is.
      if move_sandrone_towards(sandrone, 0, 0) {
        // bridge::focus_weak(options, biome.index, biome.miner.meta.phase, "delivered miner, will now be filling");
        // state.pause_after_ticks = 50;

        sandrone.state = SandroneState::MovingToNeighborCell;
        sandrone.status_desc = format!("Idle. Waiting for enough wind...");
        sandrone.air_lifting = false;
        sandrone.air_lifted = true;
        biome.miner.movable.disabled = false;
        biome.miner.meta.phase = Phase::FillingCastle_5;
        // Set the top-left corner as the initial exit tile. The exit tile is not to be filled if no other exit tiles have been seen.
        sandrone.last_empty_castle_exit_x = sandrone.expansion_min_x;
        sandrone.last_empty_castle_exit_y = sandrone.expansion_min_y;

        // println!("Return to move enabled. Press ⏎ to tick forward. Press x⏎ to exit this mode.");
        // options.return_to_move = true;

        // println!("Putting sandrone in permanent seek mode");
        // sandrone.found_end = true;
      }
      biome.miner.movable.x = sandrone.movable.x;
      biome.miner.movable.y = sandrone.movable.y;
    }
    SandroneState::Redecorating => {
      // - Move on impassible tiles only.
      //   - Randomly (?) move forward, left, or right
      //   - Only move back when the other three are blocked
      //   - Should not be possible to lock yourself in
      // - Push around Push tiles
      //   - If you walk into a push block, push it if there's an Impassible tile behind it.
      // - Push tiles can only move over Impassible tiles
      //   - If you can't push, don't.

      // Generic turning relative to current direction:
      //   dx  dy   ->     back      left      right     tl-corner   tr-corner   bl-corner   br-corner       tl-corner   tr-corner  bl-corner   br-corner
      // ^  0, -1:        0,  1     -1,  0     1,  0      -1, -1       1, -1      -1,  1       1,  1           y,  y      -y,  y      y, -y      -y, -y
      // >  1,  0:       -1,  0      0, -1     0,  1       1, -1       1,  1      -1, -1      -1,  1           x, -x       x,  x     -x, -x      -x,  x
      // < -1,  0:        1,  0      0,  1     0, -1      -1,  1      -1, -1       1,  1       1, -1           x, -x       x,  x     -x, -x      -x,  x
      // v  0,  1:        0, -1      1,  0    -1,  0       1,  1      -1,  1       1, -1      -1, -1           y,  y      -y,  y      y, -y      -y, -y
      // ---------       ------    -------    ------      ------      ------      ------      ------          ------      ------     ------      ------
      //   dx, dy       -dx,-dy     dy,-dx    -dy,dx     y+x,y-x     x-y,x+y     y-x,-y-x    -y-x,-y+x        y+x,y-x     x-y,x+y   y-x,-y-x    -y-x,-y+x


      let fx = sandrone.movable.x;
      let fy = sandrone.movable.y;
      let mut ways: Vec<(i32, i32, i32, i32, u32, bool)> = vec!();

      if fx - 1 >= sandrone.expansion_min_x {
        if matches!(get_cell_tile_at(options, &biome.world, fx - 1, fy), Tile::Soil) {
          // Can move to empty tile
          ways.push((fx - 1, fy, 0, 0, get_cell_tile_value_at(options, &biome.world, fx - 1, fy), false));
        } else if fx - 2 > sandrone.expansion_min_x && matches!(get_cell_tile_at(options, &biome.world, fx - 2, fy), Tile::Soil) {
          // If it's not impassable then it must be push. Check if you can push it to the next tile.
          ways.push((fx -1, fy, fx - 2, fy, get_cell_tile_value_at(options, &biome.world, fx - 1, fy), true));
        }
      }

      if fy - 1 >= sandrone.expansion_min_y {
        if matches!(get_cell_tile_at(options, &biome.world, fx, fy - 1), Tile::Soil) {
          // Can move to empty tile
          ways.push((fx, fy - 1, 0, 0, get_cell_tile_value_at(options, &biome.world, fx, fy - 1), false));
        } else if fy - 2 > sandrone.expansion_min_y && matches!(get_cell_tile_at(options, &biome.world, fx, fy - 2), Tile::Soil) {
          // If it's not impassable then it must be push. Check if you can push it to the next tile.
          ways.push((fx, fy - 1, fx, fy - 2, get_cell_tile_value_at(options, &biome.world, fx, fy - 1), true));
        }
      }

      if fx + 1 <= sandrone.expansion_max_x {
        if matches!(get_cell_tile_at(options, &biome.world, fx + 1, fy), Tile::Soil) {
          // Can move to empty tile
          ways.push((fx + 1, fy, 0, 0, get_cell_tile_value_at(options, &biome.world, fx + 1, fy), false));
        } else if fx + 2 < sandrone.expansion_max_x && matches!(get_cell_tile_at(options, &biome.world, fx + 2, fy), Tile::Soil) {
          // If it's not impassable then it must be push. Check if you can push it to the next tile.
          ways.push((fx + 1, fy, fx + 2, fy, get_cell_tile_value_at(options, &biome.world, fx + 1, fy), true));
        }
      }

      if fy + 1 <= sandrone.expansion_max_y {
        if matches!(get_cell_tile_at(options, &biome.world, fx, fy + 1), Tile::Soil) {
          // Can move to empty tile
          ways.push((fx, fy + 1, 0, 0, get_cell_tile_value_at(options, &biome.world, fx, fy + 1), false));
        } else if fy + 2 < sandrone.expansion_max_y && matches!(get_cell_tile_at(options, &biome.world, fx, fy + 2), Tile::Soil) {
          // If it's not impassable then it must be push. Check if you can push it to the next tile.
          ways.push((fx, fy + 1, fx, fy + 2, get_cell_tile_value_at(options, &biome.world, fx, fy + 1), true));
        }
      }

      // I think there must always be at least one?
      let some = ways[ways.len() - 1];
      ways.pop();
      let mut tx = some.0;
      let mut ty = some.1;
      let mut txx = some.2;
      let mut tyy = some.3;
      let mut v = some.4;
      let mut mv = some.5;

      for o in ways {
        match o {
          (a, b, aa, bb, c, d) => {
            if c < v {
              tx = a;
              ty = b;
              txx = aa;
              tyy = bb;
              v = c;
              mv = d;
            }
          }
        }
      }

      sandrone.movable.x = tx;
      sandrone.movable.y = ty;
      set_cell_tile_value_at(options, &mut biome.world, tx, ty, v + 1);
      if mv {
        set_cell_tile_at(options, &mut biome.world, tx, ty, Tile::Soil);
        set_cell_tile_at(options, &mut biome.world, txx, tyy, Tile::Push);
      }
    }
  }
}

pub fn ui_sandrone(sandrone: &Sandrone, options: &Options) -> String {
  if sandrone.backtracking {
    return add_fg_color_with_reset(&format!("{}", ICON_SANDRONE), COLOR_DARK_GREEN, options);
  }

  return add_fg_color_with_reset(&format!("{}", ICON_SANDRONE), COLOR_DARK_RED, options);
}

pub fn is_push_impossible_cell(options: &Options, world: &World, x: i32, y: i32) -> bool {
  return matches!(get_cell_tile_at(options, world, x, y), Tile::Push | Tile::Impassible);
}

fn can_empty_cell_be_push_cell(options: &Options, world: &World, tx: i32, ty: i32, dx: i32, dy: i32) -> bool {
  // An empty cell can become a push cell iif all the only neighboring push cells all share one axis
  // It is assumed this is part of a sandrone move check, in which case we can assume that the
  // origin (where the sandrone currently is) is already a push cell. In that case only the diagonal
  // cells that border the origin can be push cells (so we can ignore them) and we have to assert
  // that the other 5 surrounding cells are not push cells.

  // abc
  // def  <- xy is at e
  // ghi

  let e = is_push_impossible_cell(options, world, tx,   ty);

  if e {
    return false;
  }

  let a = is_push_impossible_cell(options, world, tx - 1,   ty - 1);
  let b = is_push_impossible_cell(options, world, tx,   ty - 1);
  let c = is_push_impossible_cell(options, world, tx + 1,   ty - 1);
  let d = is_push_impossible_cell(options, world, tx - 1,   ty);
  let f = is_push_impossible_cell(options, world, tx + 1,   ty);
  let g = is_push_impossible_cell(options, world, tx - 1,   ty + 1);
  let h = is_push_impossible_cell(options, world, tx,   ty + 1);
  let i = is_push_impossible_cell(options, world, tx + 1,   ty + 1);

  // if
  //   (b && !(d || f || g || h || i)) ||
  //   (f && !(b || a || d || g || h)) ||
  //   (h && !(d || a || b || c || f)) ||
  //   (d && !(b || c || f || i || h))
  // {
  //   // One horizontal or vertical cell is full while the opposite half-moon is empty
  //   // It should be safe to fill the current tile because there is still a path to the empty
  //   // neighbors currently and it will neighbor an existing tile.
  //   return true;
  // }


  if dx == -1 {
    assert_eq!(dy, 0);
    // x-1 is moving to the left so we must check fx-1,fy-1 fx-2,fy-1 fx-2,fy-2 fx-2,fy-3 fx-1,fy-3
    !b && // up
    !a && // up-left
    !d && // left
    !g && // down-left
    !h // down
  } else if dx == 1 {
    assert_eq!(dy, 0);
    // x+1 is moving to the right
    !b && // up
    !c && // up-right
    !f && // right
    !i && // down-right
    !h // down
  } else if dy == -1 {
    assert_eq!(dx, 0);
    // y-1 is moving up
    !d && // left
    !a && // up-left
    !b && // up
    !c && // up-right
    !f  // right
  } else {
    assert_eq!(dy, 1);
    assert_eq!(dx, 0);
    // y-1 is moving up
    !d && // left
    !g && // down-left
    !h && // down
    !i && // down-right
    !f // right
  }
}

// fn sandrone_can_move_to(options: &Options, world: &World, tx: i32, ty: i32, dx: i32, dy: i32) -> bool {
//   // A sandrone can always move to another push tile or to a tile that is eligible to come a push tile
//   // println!("sandrone_can_move_to({}, {}) -> {:?}", tx, ty, get_cell_tile_at(options, world, tx, ty));
//   return match get_cell_tile_at(options, world, tx, ty) {
//     Tile::Push => true,
//     Tile::Empty => can_empty_cell_be_push_cell(options, world, tx, ty, dx, dy),
//     // Can not move to any other kind of cell
//     _ => false,
//   };
// }

fn can_convert_tile_to_push(options: &Options, world: &World, tx: i32, ty: i32, dx: i32, dy: i32, _sandrone: &Sandrone) -> bool {
  // A cell can be converted to a push tile when it is empty and when it borders horizontally
  // or vertically to exactly one push/impassable cell and when all diagonal cells that are push cells are also
  // bordering that one cell (share the same axis). The origin is implied to be a push cell.
  // Additionally, the magic wall counts a special non-passable tile with slightly different rules.
  return matches!(get_cell_tile_at(options, world, tx, ty), Tile::Empty) && matches!(get_cell_stuff_at(options, world, tx, ty).1, Pickup::Nothing) && can_empty_cell_be_push_cell(options, world, tx, ty, dx, dy);
}

fn is_push_cell(options: &Options, world: &World, tx: i32, ty: i32) -> bool {
  return matches!(get_cell_tile_at(options, world, tx, ty), Tile::Push);
}

fn move_sandrone_towards(sandrone: &mut Sandrone, to_x: i32, to_y: i32) -> bool {
  let bx = sandrone.movable.x;
  let by = sandrone.movable.y;

  if bx == to_x && by == to_y {
    return true;
  } else {
    // Now move closer to the closest target
    let x1 = to_x as f64;
    let x2 = bx as f64;
    let y1 = to_y as f64;
    let y2 = by as f64;

    // sandrone still on its way to a target...
    let dx = x1 - x2;
    let dy = y1 - y2;
    if dx.abs() > dy.abs() {
      sandrone.movable.x += if dx < 0.0 { -1 } else { 1 };
    } else {
      sandrone.movable.y += if dy < 0.0 { -1 } else { 1 };
    }
    return false;
  }
}
