use crate::slottable::SlotKind;
use super::miner::*;
use super::expando::*;
use super::world::*;
use super::values::*;
use super::cell::*;
use super::options::*;
use super::tile::*;
use super::pickup::*;

// use rand::distributions::Uniform;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
  Up = 0,
  Right = 1,
  Down = 2,
  Left = 3,
}

#[derive(Debug)]
pub struct Movable {
  pub what: i32,
  pub x: i32,
  pub y: i32,
  pub dir: Direction,
  pub now_energy: f32,
  pub init_energy: f32,
  pub history: Vec<(i32, i32)>,
  pub unique: Vec<(i32, i32)>,

  // Do not move while disabled.
  pub disabled: bool,
}

fn drill_deeper(drills: i32, hammers: i32, x: i32, y: i32, dx: i32, dy: i32, world: &mut World, options: &Options) {
  // From where you're standing, move drill count+1 steps into dx and dy direction
  // For every block encountered decrease the drill count by one
  // For every block encountered passed the first, apply a bump of the drill count left
  // Respect the world wrapping around edges

  // Offset the first block. No action here, this is the one we already bumped
  let mut next_x = x + dx;
  let mut next_y = y + dy;
  let mut strength = if hammers > 0 { hammers - 1 } else { 0 }; // Start with the hammer strength - 1
  let mut remaining = drills; // Stop after punching through this many blocks

  // Now for each step and as long as there are drills and as long as the next step is a block
  while remaining > 0 && strength > 0 {
    ensure_cell_in_world(world, options, next_x, next_y);

    let unext_x = (world.min_x.abs() + next_x) as usize;
    let unext_y = (world.min_y.abs() + next_y) as usize;

    // Apply the drill power
    match world.tiles[unext_y][unext_x] {
      Cell { tile: Tile::Wall4, pickup, tile_value, pickup_value, .. } => {
        // let multiplier_percent: Uniform<f32> = Uniform::from(0.0..100.0);
        // let r = multiplier_percent.sample(.rng).round();

        world.tiles[unext_y][unext_x] = match strength {
          1 => create_cell(Tile::Wall3, pickup, tile_value, pickup_value),
          2 => create_cell(Tile::Wall2, pickup, tile_value, pickup_value),
          3 => create_cell(Tile::Wall1, pickup, tile_value, pickup_value),
          _ => {
            remaining = 1;

            create_cell(Tile::Empty, pickup, tile_value, pickup_value)
          },
        };
      },
      Cell { tile: Tile::Wall3, pickup, tile_value, pickup_value, .. } => {
        world.tiles[unext_y][unext_x] = match strength {
          1 => create_cell(Tile::Wall2, pickup, tile_value, pickup_value),
          2 => create_cell(Tile::Wall1, pickup, tile_value, pickup_value),
          _ => {
            remaining = 1;
            create_cell(Tile::Empty, pickup, tile_value, pickup_value)
          },
        };
      },
      Cell { tile: Tile::Wall2, pickup, tile_value, pickup_value, .. } => {
        world.tiles[unext_y][unext_x] = match strength {
          1 => create_cell(Tile::Wall1, pickup, tile_value, pickup_value),
          _ => {
            remaining = 1;
            create_cell(Tile::Empty, pickup, tile_value, pickup_value)
          },
        };
      },
      Cell { tile: Tile::Wall1, pickup, tile_value, pickup_value, .. } => {
        world.tiles[unext_y][unext_x] = create_cell(Tile::Empty, pickup, tile_value, pickup_value); // Or a different powerup?
        remaining = 1;
      },
      _ => {
        remaining = 1;
      }
    }

    next_x = next_x + dx;
    next_y = next_y + dy;

    remaining = remaining - 1;
    strength = strength - 1;
  }
}

fn get_most_visited_dir_from_xydir(options: &Options, world: &World, wx: i32, wy: i32, dir: Direction) -> Direction {
  // Given a direction and a coord, assuming a 90 degree turn must be made, which direction
  // turns us toward the cell that is visited more? In case of a tie turn clockwise (TBD).

  let (_, _, _, visited_left) = get_cell_stuff_at(options, world, wx + match dir {
    Direction::Up => -1,
    Direction::Right => 0,
    Direction::Down => 1,
    Direction::Left => 0,
  }, wy + match dir {
    Direction::Up => 0,
    Direction::Right => -1,
    Direction::Down => 0,
    Direction::Left => 1,
  });
  let (_, _, _, visited_right) = get_cell_stuff_at(options, world, wx + match dir {
    Direction::Up => 1,
    Direction::Right => 0,
    Direction::Down => -1,
    Direction::Left => 0,
  }, wy + match dir {
    Direction::Up => 0,
    Direction::Right => 1,
    Direction::Down => 0,
    Direction::Left => -1,
  });

  if visited_left > visited_right {
    // Turn counter-clockwise
    match dir {
      Direction::Up => Direction::Left,
      Direction::Right => Direction::Up,
      Direction::Down => Direction::Right,
      Direction::Left => Direction::Down,
    }
  } else {
    // Turn clockwise
    match dir {
      Direction::Up => Direction::Right,
      Direction::Right => Direction::Down,
      Direction::Down => Direction::Left,
      Direction::Left => Direction::Up,
    }
  }
}

fn push_corner_move(options: &Options, world: &World, mx: i32, my: i32, dx: i32, dy: i32, back_case: bool) -> (i32, i32, bool ) {

  // Coerce the miner into a subset of directions in some cases, depending on
  // how the miner is surrounded by push tiles. There are two discernible patterns which are
  // mirrored and rotated as well.
  // One case is the "corner":
  // - Miner hits push block P
  // - Left OR right (but not both) to the miner is a push block Q
  // - Diagonally behind of the miner, on the other side of Q, is another push block R
  // -> In that case the miner can only move away from P or Q.
  // The other case is "dead end";
  // - Miner hits push block P
  // - Left AND right of the miner are also push blocks
  // -> The miner must move away from P
  // When neither left nor right of the block is a push block, the miner is free to move
  // according to its regular rules.
  // Due to the nature of how the push blocks are generated this may still lead to infinite
  // loops. These need to be broken by the broken GPS or something. Hopefully that's enough.

  // Generic turning relative to current direction:
  //   dx  dy   ->     back      left      right     tl-corner   tr-corner   bl-corner   br-corner       tl-corner   tr-corner  bl-corner   br-corner
  // ^  0, -1:        0,  1     -1,  0     1,  0      -1, -1       1, -1      -1,  1       1,  1           y,  y      -y,  y      y, -y      -y, -y
  // >  1,  0:       -1,  0      0, -1     0,  1       1, -1       1,  1      -1, -1      -1,  1           x, -x       x,  x     -x, -x      -x,  x
  // < -1,  0:        1,  0      0,  1     0, -1      -1,  1      -1, -1       1,  1       1, -1           x, -x       x,  x     -x, -x      -x,  x
  // v  0,  1:        0, -1      1,  0    -1,  0       1,  1      -1,  1       1, -1      -1, -1           y,  y      -y,  y      y, -y      -y, -y
  // ---------       ------    -------    ------      ------      ------      ------      ------          ------      ------     ------      ------
  //   dx, dy       -dx,-dy     dy,-dx    -dy,dx     y+x,y-x     x-y,x+y     y-x,-y-x    -y-x,-y+x        y+x,y-x     x-y,x+y   y-x,-y-x    -y-x,-y+x

  // If deltax/y were both 0 then there would be no movement
  assert!(dy != 0 || dx == -1 || dx == 1, "if deltay is 0 then deltax should be nonzero left or right {}", dx);
  assert!(dx != 0 || dy == -1 || dy == 1, "if deltax is 0 then deltay should be nonzero left or right {}", dy);

  // Check the cells in all eight directions of the current location
  let blocked_xy = match get_cell_tile_at(options, world, mx, my) { Tile::Push | Tile::Impassible => true, _ => false };
  let blocked_fl = match get_cell_tile_at(options, world, mx + dy + dx, my + dy + -dx) { Tile::Push | Tile::Impassible => true, _ => false };
  let blocked_fwd = if back_case { false } else { match get_cell_tile_at(options, world, mx + dx, my + dy) { Tile::Push | Tile::Impassible => true, _ => false } };
  let blocked_fr = match get_cell_tile_at(options, world, mx + dx + -dy, my + dx + dy) { Tile::Push | Tile::Impassible => true, _ => false };
  let blocked_right = match get_cell_tile_at(options, world, mx + -dy, my + dx) { Tile::Push | Tile::Impassible => true, _ => false };
  let blocked_br = match get_cell_tile_at(options, world, mx + -dy + -dx, my + dx + -dy) { Tile::Push | Tile::Impassible => true, _ => false };
  let blocked_back = back_case || match get_cell_tile_at(options, world, mx + -dx, my + -dy) { Tile::Push | Tile::Impassible => true, _ => false };
  let blocked_bl = match get_cell_tile_at(options, world, mx + dy + -dx, my + -dx + -dy) { Tile::Push | Tile::Impassible => true, _ => false };
  let blocked_left = match get_cell_tile_at(options, world, mx + dy, my + -dx) { Tile::Push | Tile::Impassible => true, _ => false };

  // println!("  blocked; fl: {}, fwd: {}, fr: {}, right: {}, br: {}, back: {}, bl: {}, left: {}, n={}", blocked_fl, blocked_fwd, blocked_fr, blocked_right, blocked_br, blocked_back, blocked_bl, blocked_left, (blocked_fwd as u8) + (blocked_left as u8) + (blocked_back as u8) + (blocked_right as u8));

  // If already blocked then there's no point in detecting whether we need to block
  let blocked_count = (blocked_fwd as u8) + (blocked_left as u8) + (blocked_back as u8) + (blocked_right as u8);

  if blocked_count == 4 {
    // Boxed in. Ideally this shouldn't happen. Should we instead destroy a block?
    // May as well fill this hole but I'd rather fix the bug.
    // println!("  miner locked in");
    return ( dx, dy, !blocked_xy );
  }

  if blocked_count == 3 {
    // println!("  miner in dead end, d {},{}", dx, dy);

    // This is a dead end. Seal it off. Turn towards the open end.
    if !blocked_fwd { return ( dx, dy, !blocked_xy ); }
    if !blocked_left { return ( dy, -dx, !blocked_xy ); }
    if !blocked_back { return ( -dx, -dy, !blocked_xy ); }
    if !blocked_right { return ( -dy, dx, !blocked_xy ); }
    panic!("should be one of the above");
  }

  if blocked_count == 2 {
    // println!("  miner in corner or corridor");

    // This is either a corner or a corridor. But which is it and what's its orientation?
    if blocked_fwd {
      // Corridor or corner?
      if blocked_back {
        // println!("  fwd-back corridor");
        // Corridor. Ignore. Must go left or right.
        return ( 0, 0, false ); // the 0,0 will invoke normal turning rules
      } else if blocked_left {
        // println!("  fwd-left corner");
        // Corner fwd-left. Fill only if the back-right corner is available.
        // Moving to the right regardless.
        return ( -dy, dx, !blocked_xy && !blocked_br );
      } else {
        // println!("  fwd-right corner");
        assert!(blocked_right, "right and fwd must be blocked since back and left were not");
        // Corner fwd-right. Fill only if the back-left corner is available.
        // Moving to the left regardless.
        return ( dy, -dx, !blocked_xy && !blocked_bl );
      }
    } else if blocked_left {
      // Corridor or corner?
      if blocked_right {
        // Corridor. Ignore. Keep going forward
        // println!("  left-right corridor");
        return ( dx, dy, false );
      } else {
        // println!("  left-back corner");
        assert!(blocked_back, "left and back must be blocked since fwd and right were not");
        // Corner back-left. Fill only if the fwd-right corner is available.
        // Moving fwd regardless.
        return ( dx, dy, !blocked_xy && !blocked_fr );
      }
    } else {
      // println!("  fwd-left corner");
      assert!(blocked_back && blocked_right, "must be back and right since fwd and left were not");
      // Must be right-back corner. Fill only if fwd-left corner is available.
      // Moving fwd regardless
      return ( dx, dy, !blocked_xy && !blocked_fl );
    }

    // unreachable
  }

  // Either the miner is not next to a push/impassable block or just one. Do not fill.
  if blocked_fwd {
    // Force turn
    // println!("  free, force turn");
    return ( 0, 0, false );
  } else {
    // println!("  free, go forth");
    return ( dx, dy, false );
  }

  // println!("  xy taken");
  if !blocked_fwd { return ( dx, dy, false ); }
  return ( 0, 0, false ); // invokes normal turning rules

  //
  // let mut turnx = 0;
  // let mut turny = 0;
  // if blocked_left && blocked_right {
  //   println!("  Gotta turn around...");
  //   if blocked_back {
  //     // Somehow got stuck in the walls. This could happen when teleporting the miner to 0,0
  //     // or maybe due to a race condition (but hopefully not). Let it move under normal rules.
  //     turnx = 0;
  //     turny = 0;
  //   } else {
  //     // Must move backward (u-turn)
  //     turnx = -dx;
  //     turny = -dy;
  //
  //     // Don't seal off the origin (it'll mess up the hydrone, if nothing else)
  //     if mx != 0 || my != 0 {
  //       println!("  Sealing off this dead end at {}x{}...", mx, my);
  //       set_cell_tile_at(options, world, mx, my, Tile::Impassible);
  //       //options.visual = true;
  //     }
  //   }
  // } else if blocked_left {
  //   // Must move rightward
  //   turnx = -dy;
  //   turny = dx;
  //
  //   // This is a corner. You are on (a) facing up to (Q). The forward and left of you are push
  //   // blocks (P/Q). We want to know if the current tile (a) can be sealed off. Now we must
  //   // check to confirm that the back tile (c) and back-right tile (d) are not push blocks.
  //   // We must check this otherwise we risk locking ourselves into an unbreakable box.
  //   // We already know (b) is not a push block and by verifying (c) and (d) we know that
  //   // there's still always a path from (b) to (c) over (d).
  //   //  state:   PQP     seal    PQP
  //   //           Pab     corner: PRb
  //   //           Pcd             Pcd
  //   // This situation can happen due to blocking pockets of non-empty cells, like expandos.
  //
  //   if
  //     // Current tile can not already be blocked by a push or impassable tile
  //     !blocked_xy &&
  //     // This is (b) in the diagram
  //     !blocked_back &&
  //     // This is (d) in the diagram
  //     !blocked_br
  //   {
  //     // Should be okay to seal this cell off and turn to the right
  //     println!("  Sealing off this pocket corner at {}x{}...", mx, my);
  //     set_cell_tile_at(options, world, mx, my, Tile::Impassible);
  //   }
  // } else if blocked_right {
  //   // Must move leftward
  //   turnx = dy;
  //   turny = -dx;
  //
  //   // (See description in blocked_left branch)
  //   if !blocked_xy && !blocked_back && !blocked_bl {
  //     println!("  Sealing off this pocket corner at {}x{}...", mx, my);
  //     set_cell_tile_at(options, world, mx, my, Tile::Impassible);
  //   }
  //
  // } else {
  //   // free to move under normal rules
  //   turnx = 0;
  //   turny = 0;
  // }
}

fn move_it_xy(movable: &mut Movable, mslots_maybe: &MinerSlots, meta: &mut MinerMeta, world: &mut World, options: &mut Options, nextx: i32, nexty: i32, deltax: i32, deltay: i32) {
  let mut was_boring = false; // Did we just move forward? No blocks, no pickups?
  if movable.what == WHAT_MINER {
    meta.points_last_move = 0;
  }

  // If this move would go OOB, expand the world to make sure that does not happen

  // println!("");
  // println!("world A: {:?}", world);
  ensure_cell_in_world(world, options, nextx, nexty);
  // println!("world B: {:?}", world);

  let unextx = (world.min_x.abs() + nextx) as usize;
  let unexty = (world.min_y.abs() + nexty) as usize;

  // println!("Stepping to: {}x{} ({}x{}) world is {}x{} - {}x{}", nextx, nexty, unextx, unexty, world.min_x, world.min_y, world.max_x, world.max_y);
  // println!("Actual world has {} lines and the first row has {} cols", world.tiles.len(), world.tiles[0].len());
  // println!("Wot? {} + {} = {} -> {}", world.min_y, nexty, world.min_y + nexty, unexty);

  if world.tiles.len() <= unexty { assert_eq!((unexty, "unexty"), (world.tiles.len(), "len"), "OOB: world is not high enough"); }
  if world.tiles[unexty].len() <= unextx { assert_eq!((unextx, "unextx"), (world.tiles[unexty].len(), "len"), "OOB: world is not wide enough"); }
  assert!(world.tiles.len() > unexty);
  // assert!(unexty >= 0);
  assert!(world.tiles[unexty].len() > unextx);
  // assert!(unextx >= 0);

  let drills = meta.kind_counts[SlotKind::Drill as usize];
  let hammers = meta.kind_counts[SlotKind::Hammer as usize];
  match world.tiles[unexty][unextx] {
    Cell { tile: Tile::Wall4, pickup, tile_value, pickup_value, .. } => {
      world.tiles[unexty][unextx] = match if movable.what == WHAT_MINER { hammers } else { 1 } {
        0 => create_cell(Tile::Wall3, pickup, tile_value, pickup_value),
        1 => create_cell(Tile::Wall2, pickup, tile_value, pickup_value),
        2 => create_cell(Tile::Wall1, pickup, tile_value, pickup_value),
        _ => create_cell(Tile::Empty, pickup, tile_value, pickup_value),
      };
      // TODO: should drones use same "prefer visited tiles" heuristic as miner?
      movable.dir = get_most_visited_dir_from_xydir(options, world, nextx, nexty, movable.dir);
      movable.now_energy = movable.now_energy - meta.block_bump_cost;
      if movable.what == WHAT_MINER {
        if drills > 0 {
          drill_deeper(drills, hammers, nextx, nexty, deltax, deltay, world, options);
        }
        meta.prev_move_bumped = true;
      }
    },
    Cell { tile: Tile::Wall3, pickup, tile_value, pickup_value, .. } => {
      world.tiles[unexty][unextx] = match if movable.what == WHAT_MINER { hammers } else { 1 } {
        0 => create_cell(Tile::Wall2, pickup, tile_value, pickup_value),
        1 => create_cell(Tile::Wall1, pickup, tile_value, pickup_value),
        _ => create_cell(Tile::Empty, pickup, tile_value, pickup_value),
      };
      movable.dir = get_most_visited_dir_from_xydir(options, world, nextx, nexty, movable.dir);
      movable.now_energy = movable.now_energy - meta.block_bump_cost;
      if movable.what == WHAT_MINER {
        if drills > 0 {
          drill_deeper(drills, hammers, nextx, nexty, deltax, deltay, world, options);
        }
        meta.prev_move_bumped = true;
      }
    },
    Cell { tile: Tile::Wall2, pickup, tile_value, pickup_value, .. } => {
      world.tiles[unexty][unextx] = match if movable.what == WHAT_MINER { hammers } else { 1 } {
        0 => create_cell(Tile::Wall1, pickup, tile_value, pickup_value),
        _ => create_cell(Tile::Empty, pickup, tile_value, pickup_value),
      };
      movable.dir = get_most_visited_dir_from_xydir(options, world, nextx, nexty, movable.dir);
      movable.now_energy = movable.now_energy - meta.block_bump_cost;
      if movable.what == WHAT_MINER {
        if drills > 0 {
          drill_deeper(drills, hammers, nextx, nexty, deltax, deltay, world, options);
        }
        meta.prev_move_bumped = true;
      }
    },
    Cell { tile: Tile::Wall1, pickup, tile_value, pickup_value, .. } => {
      world.tiles[unexty][unextx] = create_cell(Tile::Empty, pickup, tile_value, pickup_value); // Or a different powerup?
      movable.dir = get_most_visited_dir_from_xydir(options, world, nextx, nexty, movable.dir);
      movable.now_energy = movable.now_energy - meta.block_bump_cost;
      if movable.what == WHAT_MINER {
        if drills > 0 {
          drill_deeper(drills, hammers, nextx, nexty, deltax, deltay, world, options);
        }
        meta.prev_move_bumped = true;
      }
    },
    Cell { tile: Tile::Push | Tile::Impassible, .. } => {
      // Moving to a push tile or an impassible (dead end) tile. Must turn and try to make sure
      // not to send the movable into an infinite loop.

      if movable.what == WHAT_HYDRONE {
        // ignore
      } else if movable.what == WHAT_WINDRONE {
        // ignore
      } else if movable.what == WHAT_DRONE {
        // ignore (otherwise you risk the miner getting boxed in)
      } else {
        // Note: this is a miner or a regular drone

        // println!("Miner just bumped into push. pos={},{} and dir={:?}", movable.x, movable.y, movable.dir);
        let ( tx, ty, fill ): ( i32, i32, bool ) = push_corner_move(options, world, movable.x, movable.y, deltax, deltay, false) ;
        // println!("  Result: tx={} ty={} fill={}", tx, ty, fill);
        if fill { set_cell_tile_at(options, world, movable.x, movable.y, Tile::Impassible); }

        // We have the new delta xy for the turn. Act accordingly. If they're 0, do the normal rule
        movable.dir = match (tx, ty) {
          (-1, 0) => Direction::Left,
          (1, 0) => Direction::Right,
          (0, 1) => Direction::Down,
          (0, -1) => Direction::Up,
          (0, 0) => get_most_visited_dir_from_xydir(options, world, nextx, nexty, movable.dir),
          _ => panic!("This delta should not be possible {},{}", tx, ty),
        };

        //movable.now_energy = movable.now_energy - meta.block_bump_cost;
      }
    }

    // The rest is considered an empty tile
    |  Cell {
      tile:
          Tile::Fountain
        | Tile::ZeroZero
        | Tile::TenLine
        | Tile::HideWorld
        | Tile::Test2
        | Tile::Test3
        | Tile::Empty
        | Tile::ExpandoWater,
      pickup,
      tile_value,
      pickup_value,
      ..
    } => {

      // TODO: do we really want to check for this every cell? Does it matter?
      let blocked_back = match get_cell_tile_at(options, world, movable.x + -deltax, movable.y + -deltay) { Tile::Push | Tile::Impassible => true, _ => false };
      if blocked_back {
        // println!("Miner just bumped into push. pos={},{} and dir={:?}", movable.x, movable.y, movable.dir);
        let ( tx, ty, fill ): ( i32, i32, bool ) = push_corner_move(options, world, movable.x, movable.y, deltax, deltay, true) ;
        // println!("  Result: tx={} ty={} fill={}", tx, ty, fill);
        if fill { set_cell_tile_at(options, world, movable.x, movable.y, Tile::Impassible); }
      }

      match pickup {
      Pickup::Diamond => {
        // Do we have any purity scanners primed? Bump the value by that many.
        // Note: purity scanner only works for the miner itself. For drones, slots is empty
        let mut primed = 0;
        for n in mslots_maybe {
          match n.kind {
            SlotKind::PurityScanner => if n.cur_cooldown >= n.max_cooldown {
              primed += 1;
            },
            _ => ()
          }
        }

        // Different gems with different points.
        // Miners could have properties or powerups to affect this, too.
        let gem_value = match (pickup_value + primed).min(3) {
          0 => {
            meta.inventory.diamond_white += 1;
            1
          },
          1 => {
            meta.inventory.diamond_green += 1;
            2
          },
          2 => {
            meta.inventory.diamond_blue += 1;
            3
          },
          3 => {
            meta.inventory.diamond_yellow += 1;
            4
          },
          _ => panic!("what value did this diamond have: {:?}", world.tiles[unexty][unextx]),
        };
        if movable.what == WHAT_MINER {
          meta.points_last_move = gem_value;
          if world.tiles[unexty][unextx].visited == 0 {
            movable.unique.push((nextx, nexty));
          }
          world.tiles[unexty][unextx].visited += 1;
        }
        world.tiles[unexty][unextx] = create_visited_cell(Tile::Empty, Pickup::Nothing, 0, 0, world.tiles[unexty][unextx].visited);
        movable.x = nextx;
        movable.y = nexty;
        movable.history.push((nextx, nexty));
      },
      Pickup::Energy => {
        movable.now_energy = movable.now_energy + (E_VALUE as f64 * ((100.0 + meta.multiplier_energy_pickup as f64) / 100.0)) as f32;
        if movable.now_energy > meta.max_energy {
          movable.now_energy = meta.max_energy;
          if world.tiles[unexty][unextx].visited == 0 {
            movable.unique.push((nextx, nexty));
          }
          world.tiles[unexty][unextx].visited += 1;
        }
        meta.inventory.energy += 1;
        world.tiles[unexty][unextx] = create_visited_cell(Tile::Empty, Pickup::Nothing, tile_value, pickup_value, world.tiles[unexty][unextx].visited);
        movable.x = nextx;
        movable.y = nexty;
        movable.history.push((nextx, nexty));
      },
      Pickup::Stone => {
        // Do we have any purity scanners primed? Bump the value by that many.
        // Note: purity scanner only works for the miner itself. For drones, slots is empty
        let mut primed = 0;
        for n in mslots_maybe {
          match n.kind {
            SlotKind::PurityScanner => if n.cur_cooldown >= n.max_cooldown {
              primed += 1;
            },
            _ => ()
          }
        }

        // println!("picking up a stone, value: {}, primed: {}", value, primed);

        match (pickup_value + primed).min(3) {
          0 => meta.inventory.stone_white += 1,
          1 => meta.inventory.stone_green += 1,
          2 => meta.inventory.stone_blue += 1,
          3 => meta.inventory.stone_yellow += 1,
          _ => panic!("what value did this stone have: {:?} {} {} {}", world.tiles[unexty][unextx], pickup_value, primed, (pickup_value + primed).min(3)),
        }
        if movable.what == WHAT_MINER {
          meta.points_last_move = pickup_value as i32;
          if world.tiles[unexty][unextx].visited == 0 {
            movable.unique.push((nextx, nexty));
          }
          world.tiles[unexty][unextx].visited += 1;
        }
        world.tiles[unexty][unextx] = create_visited_cell(Tile::Empty, Pickup::Nothing, tile_value, pickup_value, world.tiles[unexty][unextx].visited);
        movable.x = nextx;
        movable.y = nexty;
        movable.history.push((nextx, nexty));
      },
      Pickup::Wind => {
        meta.inventory.wind += 1;
        world.tiles[unexty][unextx] = create_visited_cell(Tile::Empty, Pickup::Nothing, tile_value, pickup_value, world.tiles[unexty][unextx].visited);
        movable.x = nextx;
        movable.y = nexty;
        movable.history.push((nextx, nexty));
        if movable.what == WHAT_MINER {
          if world.tiles[unexty][unextx].visited == 0 {
            movable.unique.push((nextx, nexty));
          }
        }
        world.tiles[unexty][unextx].visited += 1;
      },
      Pickup::Water => {
        meta.inventory.water += 1;
        world.tiles[unexty][unextx] = create_visited_cell(Tile::ExpandoWater, Pickup::Nothing, tile_value, pickup_value, world.tiles[unexty][unextx].visited);
        movable.x = nextx;
        movable.y = nexty;
        movable.history.push((nextx, nexty));
        if movable.what == WHAT_MINER {
          if world.tiles[unexty][unextx].visited == 0 {
            movable.unique.push((nextx, nexty));
          }
        }
        world.tiles[unexty][unextx].visited += 1;
      },
      Pickup::Wood => {
        meta.inventory.wood += 1;
        world.tiles[unexty][unextx] = create_visited_cell(Tile::Empty, Pickup::Nothing, tile_value, pickup_value, world.tiles[unexty][unextx].visited);
        movable.x = nextx;
        movable.y = nexty;
        movable.history.push((nextx, nexty));
        if movable.what == WHAT_MINER {
          if world.tiles[unexty][unextx].visited == 0 {
            movable.unique.push((nextx, nexty));
          }
          world.tiles[unexty][unextx].visited += 1;
        }
      },
      | Pickup::Nothing
      | Pickup::Expando // Ignore, fake pickup
      | Pickup::Fountain // Ignore, fake pickup... TODO: probably some special behavior?
      => {
        movable.x = nextx;
        movable.y = nexty;
        was_boring = true;
        movable.history.push((nextx, nexty));
        if movable.what == WHAT_MINER {
          if world.tiles[unexty][unextx].visited == 0 {
            movable.unique.push((nextx, nexty));
          }
          world.tiles[unexty][unextx].visited += 1;
        }
      },
    }
  },
  }

  // Do not remove an expando when moving over it.
  if !matches!(world.tiles[unexty][unextx].tile, Tile::ExpandoWater) {
    match world.tiles[unexty][unextx] {
      Cell {tile: Tile::Empty, pickup: Pickup::Expando, pickup_value, ..} => {
        // This must have been an expando that was just revealed.
        // Set the cell to water tile and add the expando to the world so it can flow.
        world.tiles[unexty][unextx].tile = Tile::ExpandoWater;
        world.expandos.push(create_expando(nextx, nexty, pickup_value));
      },
      _ => {},
    }
  }

  if movable.what == WHAT_MINER {
    if was_boring {
      // Prevent endless loops by making it increasingly more difficult to make consecutive moves that where nothing happens
      movable.now_energy = movable.now_energy - meta.boredom_level as f32;
      // The cost grows the longer nothing keeps happening ("You're getting antsy, thirsty for an event")
      meta.boredom_level = meta.boredom_level + 1;
    } else {
      meta.boredom_level = 0;
    }
  } else {
    movable.now_energy -= 1.0;
  }

  if movable.now_energy < 0.0 {
    movable.now_energy = 0.0;
  }
}

pub fn move_movable(movable: &mut Movable, mslots_maybe: &MinerSlots, meta: &mut MinerMeta, world: &mut World, options: &mut Options) {
  if movable.disabled { return; }

  // println!("moving from {}x{}", movable.x, movable.y);
  match movable.dir {
    Direction::Up => {
      let nexty = movable.y - 1;
      move_it_xy(movable, mslots_maybe, meta, world, options, movable.x, nexty, 0, -1);
    },
    Direction::Left => {
      let nextx = movable.x - 1;
      move_it_xy(movable, mslots_maybe, meta, world, options, nextx, movable.y, -1, 0);
    },
    Direction::Down => {
      let nexty = movable.y + 1;
      move_it_xy(movable, mslots_maybe, meta, world, options, movable.x, nexty, 0, 1);
    },
    Direction::Right => {
      let nextx = movable.x + 1;
      move_it_xy(movable, mslots_maybe, meta, world, options, nextx, movable.y, 1, 0);
    },
  }
}
