use super::world::*;
use super::cell::*;
use super::options::*;
use super::tile::*;

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

  // Do not move while disabled.
  pub disabled: bool,
}

pub fn drill_deeper(drills: i32, hammers: i32, x: i32, y: i32, dx: i32, dy: i32, world: &mut World, options: &Options) {
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
          1 => create_unvisited_cell(Tile::Wall3, pickup, tile_value, pickup_value),
          2 => create_unvisited_cell(Tile::Wall2, pickup, tile_value, pickup_value),
          3 => create_unvisited_cell(Tile::Wall1, pickup, tile_value, pickup_value),
          _ => {
            remaining = 1;

            create_unvisited_cell(Tile::Empty, pickup, tile_value, pickup_value)
          },
        };
      },
      Cell { tile: Tile::Wall3, pickup, tile_value, pickup_value, .. } => {
        world.tiles[unext_y][unext_x] = match strength {
          1 => create_unvisited_cell(Tile::Wall2, pickup, tile_value, pickup_value),
          2 => create_unvisited_cell(Tile::Wall1, pickup, tile_value, pickup_value),
          _ => {
            remaining = 1;
            create_unvisited_cell(Tile::Empty, pickup, tile_value, pickup_value)
          },
        };
      },
      Cell { tile: Tile::Wall2, pickup, tile_value, pickup_value, .. } => {
        world.tiles[unext_y][unext_x] = match strength {
          1 => create_unvisited_cell(Tile::Wall1, pickup, tile_value, pickup_value),
          _ => {
            remaining = 1;
            create_unvisited_cell(Tile::Empty, pickup, tile_value, pickup_value)
          },
        };
      },
      Cell { tile: Tile::Wall1, pickup, tile_value, pickup_value, .. } => {
        world.tiles[unext_y][unext_x] = create_unvisited_cell(Tile::Empty, pickup, tile_value, pickup_value); // Or a different powerup?
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

pub fn get_most_visited_dir_from_xydir(options: &Options, world: &World, wx: i32, wy: i32, dir: Direction) -> Direction {
  // Given a direction and a coord, assuming a 90 degree turn must be made, which direction
  // turns us toward the cell that is visited more? In case of a tie turn clockwise (TBD).

  let lxy = coord_left(wx, wy, dir);
  let (_, _, _, visited_left) = get_cell_stuff_at(options, world, lxy.0, lxy.1);
  let rxy = coord_right(wx, wy, dir);
  let (_, _, _, visited_right) = get_cell_stuff_at(options, world, rxy.0, rxy.1);

  return turn_lr(dir, visited_left > visited_right);
}

pub fn push_corner_move(options: &Options, world: &World, mx: i32, my: i32, dx: i32, dy: i32, back_case: bool, bug: bool, dir: Direction) -> (i32, i32, bool ) {
  let (tx, ty, fill) = _push_corner_move(options, world, mx, my, dx, dy, back_case, bug, dir);
  if bug {
    println!("  -> new tx: {} ty: {}, fill? {}", tx, ty, fill);
  }
  return (tx, ty, fill);
}
pub fn _push_corner_move(options: &Options, world: &World, mx: i32, my: i32, dx: i32, dy: i32, _back_case: bool, bug: bool, dir: Direction) -> (i32, i32, bool ) {
  // Return value: delta x, delta y, fill

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
  let blocked_xy = matches!(get_cell_tile_at(options, world, mx, my), Tile::Push | Tile::Impassible);
  let blocked_fl = matches!(get_cell_tile_at(options, world, mx + dy + dx, my + dy + -dx), Tile::Push | Tile::Impassible);
  let blocked_fwd = matches!(get_cell_tile_at(options, world, mx + dx, my + dy), Tile::Push | Tile::Impassible);
  let blocked_fr = matches!(get_cell_tile_at(options, world, mx + dx + -dy, my + dx + dy), Tile::Push | Tile::Impassible);
  let blocked_right = matches!(get_cell_tile_at(options, world, mx + -dy, my + dx), Tile::Push | Tile::Impassible);
  let blocked_br = matches!(get_cell_tile_at(options, world, mx + -dy + -dx, my + dx + -dy), Tile::Push | Tile::Impassible);
  let blocked_back = matches!(get_cell_tile_at(options, world, mx + -dx, my + -dy), Tile::Push | Tile::Impassible);
  let blocked_bl = matches!(get_cell_tile_at(options, world, mx + dy + -dx, my + -dx + -dy), Tile::Push | Tile::Impassible);
  let blocked_left = matches!(get_cell_tile_at(options, world, mx + dy, my + -dx), Tile::Push | Tile::Impassible);

  if bug {
    println!("/---\\                         \n|{}{}{}|                       \n|{} {}| {:?}                 \n|{}{}{}|                         \n\\---/     ",
      if blocked_fl { 'F' } else { '.' },
      if blocked_fwd { 'F' } else { '.' },
      if blocked_fr { 'F' } else { '.' },

      if blocked_left { 'F' } else { '.' },
      if blocked_right { 'F' } else { ',' },
      dir,

      if blocked_bl { 'F' } else { '.' },
      if blocked_back { 'F' } else { '.' },
      if blocked_br { 'F' } else { '.' },
    );
    println!(" xy: {},{} dxy: {},{}  blocked; fl: {}, fwd: {}, fr: {}, right: {}, br: {}, back: {}, bl: {}, left: {}, n={}",
      mx,my, dx,dy, blocked_fl, blocked_fwd, blocked_fr, blocked_right, blocked_br, blocked_back, blocked_bl, blocked_left, (blocked_fwd as u8) + (blocked_left as u8) + (blocked_back as u8) + (blocked_right as u8));
  }

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
}

// pub fn bump_wall(strength: i32, world: &mut World, options: &Options, movable: &mut Movable, hammers: i32, drills: i32, pickup: Pickup, tile_value: u32, pickup_value: u32, nextx: i32, nexty: i32, deltax: i32, deltay: i32, unextx: usize, unexty: usize, meta: &mut MinerMeta, _building_sandcastle: bool, _magic_min_x: i32, _magic_min_y: i32, _magic_max_x: i32, _magic_max_y: i32) {
//   let n = strength - (1 + if movable.what == WHAT_MINER { hammers } else { 0 });
//
//   biome.world.tiles[unexty][unextx] = match n.max(0) {
//     3 => create_unvisited_cell(Tile::Wall3, pickup, tile_value, pickup_value),
//     2 => create_unvisited_cell(Tile::Wall2, pickup, tile_value, pickup_value),
//     1 => create_unvisited_cell(Tile::Wall1, pickup, tile_value, pickup_value),
//     0 => create_unvisited_cell(Tile::Empty, pickup, tile_value, pickup_value),
//     // always at least -1
//     _ => panic!("A bump should always at least decrease the wall by one so it can never stay 4: {}", n),
//   };
//   if n <= 0 {
//     // Broke a wall. Add sand.
//     // TODO: what about the drill? What about bonuses? Should it be u32 or f32?
//     biome.miner.meta.inventory.sand += 1;
//   }
//   if movable.what == WHAT_MINER {
//     if drills > 0 {
//       drill_deeper(drills, hammers, nextx, nexty, deltax, deltay, world, options);
//     }
//     meta.prev_move_bumped = true;
//   }
//
//   movable.now_energy = movable.now_energy - meta.block_bump_cost;
//   // TODO: should drones use same "prefer visited tiles" heuristic as miner?
//   movable.dir = get_most_visited_dir_from_xydir(options, world, nextx, nexty, movable.dir);
// }

pub fn dir_to_move_delta(dir: Direction) -> (i32, i32) {
  // println!("moving from {}x{}", movable.x, movable.y);
  match dir {
    Direction::Up => (0, -1),
    Direction::Left => (-1, 0),
    Direction::Down => (0, 1),
    Direction::Right => (1, 0),
  }
}

pub fn turn_left(dir: Direction) -> Direction {
  return turn_lr(dir, true);
}

pub fn turn_right(dir: Direction) -> Direction {
  return turn_lr(dir, false);
}

pub fn turn_lr(dir: Direction, left: bool) -> Direction {
  return match dir {
    Direction::Up => if left { Direction::Left } else { Direction::Right },
    Direction::Right => if left { Direction::Up } else { Direction::Down },
    Direction::Down => if left { Direction::Right } else { Direction::Left },
    Direction::Left => if left { Direction::Down } else { Direction::Up },
  };
}

pub fn turn_back(dir: Direction) -> Direction {
  return match dir {
    Direction::Up => Direction::Down,
    Direction::Right => Direction::Left,
    Direction::Down => Direction::Up,
    Direction::Left => Direction::Right,
  };
}

pub fn coord_forward(x: i32, y: i32, dir: Direction) -> (i32, i32) {
    match dir {
      Direction::Up =>    (x,    y-1),
      Direction::Right => (x+1,    y),
      Direction::Down =>  (x,    y+1),
      Direction::Left =>  (x-1,    y),
    }
}

pub fn coord_back(x: i32, y: i32, dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (x,    y+1),
    Direction::Right => (x-1,    y),
    Direction::Down =>  (x,    y-1),
    Direction::Left =>  (x+1,    y),
  }
}

pub fn coord_left(x: i32, y: i32, dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (x-1,    y),
    Direction::Right => (x,    y-1),
    Direction::Down =>  (x+1,    y),
    Direction::Left =>  (x,    y+1),
  }
}

pub fn coord_right(x: i32, y: i32, dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (x+1,    y),
    Direction::Right => (x,    y+1),
    Direction::Down =>  (x-1,    y),
    Direction::Left =>  (x,    y-1),
  }
}

pub fn coord_lr(x: i32, y: i32, dir: Direction, left: bool) -> (i32, i32) {
  // Left or right coordinate from given x/y/dir
  return if left {
    coord_left(x, y, dir)
  } else {
    coord_right(x, y, dir)
  };
}

pub fn coord_fl(x: i32, y: i32, dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (x-1,    y-1),
    Direction::Right => (x+1,    y-1),
    Direction::Down =>  (x+1,    y+1),
    Direction::Left =>  (x-1,    y+1),
  }
}

pub fn coord_fr(x: i32, y: i32, dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (x+1,    y-1),
    Direction::Right => (x+1,    y+1),
    Direction::Down =>  (x-1,    y+1),
    Direction::Left =>  (x-1,    y-1),
  }
}

pub fn coord_bl(x: i32, y: i32, dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (x-1,    y+1),
    Direction::Right => (x+1,    y-1),
    Direction::Down =>  (x+1,    y-1),
    Direction::Left =>  (x-1,    y+1),
  }
}

pub fn coord_br(x: i32, y: i32, dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (x+1,    y+1),
    Direction::Right => (x-1,    y+1),
    Direction::Down =>  (x-1,    y-1),
    Direction::Left =>  (x+1,    y-1),
  }
}

pub fn delta_forward(dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (0,    0-1),
    Direction::Right => (0+1,    0),
    Direction::Down =>  (0,    0+1),
    Direction::Left =>  (0-1,    0),
  }
}

pub fn delta_back(dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (0,    0+1),
    Direction::Right => (0-1,    0),
    Direction::Down =>  (0,    0-1),
    Direction::Left =>  (0+1,    0),
  }
}

pub fn delta_left(dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (0-1,    0),
    Direction::Right => (0,    0-1),
    Direction::Down =>  (0+1,    0),
    Direction::Left =>  (0,    0+1),
  }
}

pub fn delta_right(dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (0+1,    0),
    Direction::Right => (0,    0+1),
    Direction::Down =>  (0-1,    0),
    Direction::Left =>  (0,    0-1),
  }
}

pub fn delta_lr(dir: Direction, left: bool) -> (i32, i32) {
  // Left or right coordinate delta from given x/y/dir
  return if left {
    delta_left(dir)
  } else {
    delta_right(dir)
  };
}

pub fn delta_fl(dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (0-1,    0-1),
    Direction::Right => (0+1,    0-1),
    Direction::Down =>  (0+1,    0+1),
    Direction::Left =>  (0-1,    0+1),
  }
}

pub fn delta_fr(dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (0+1,    0-1),
    Direction::Right => (0+1,    0+1),
    Direction::Down =>  (0-1,    0+1),
    Direction::Left =>  (0-1,    0-1),
  }
}

pub fn delta_bl(dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (0-1,    0+1),
    Direction::Right => (0-1,    0-1),
    Direction::Down =>  (0+1,    0-1),
    Direction::Left =>  (0+1,    0+1),
  }
}

pub fn delta_br(dir: Direction) -> (i32, i32) {
  match dir {
    Direction::Up =>    (0+1,    0+1),
    Direction::Right => (0-1,    0+1),
    Direction::Down =>  (0-1,    0-1),
    Direction::Left =>  (0+1,    0-1),
  }
}
