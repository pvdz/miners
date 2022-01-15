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

    let mut unext_x = (world.min_x.abs() + next_x) as usize;
    let mut unext_y = (world.min_y.abs() + next_y) as usize;

    // Apply the drill power
    match world.tiles[unext_y][unext_x] {
      Cell { tile: Tile::Wall4, pickup, value, visited: _ } => {
        // let multiplier_percent: Uniform<f32> = Uniform::from(0.0..100.0);
        // let r = multiplier_percent.sample(.rng).round();

        world.tiles[unext_y][unext_x] = match strength {
          1 => create_cell(Tile::Wall3, pickup, value),
          2 => create_cell(Tile::Wall2, pickup, value),
          3 => create_cell(Tile::Wall1, pickup, value),
          _ => {
            remaining = 1;

            create_cell(Tile::Empty, pickup, value)
          },
        };
      },
      Cell { tile: Tile::Wall3, pickup, value, visited: _ } => {
        world.tiles[unext_y][unext_x] = match strength {
          1 => create_cell(Tile::Wall2, pickup, value),
          2 => create_cell(Tile::Wall1, pickup, value),
          _ => {
            remaining = 1;
            create_cell(Tile::Empty, pickup, value)
          },
        };
      },
      Cell { tile: Tile::Wall2, pickup, value, visited: _ } => {
        world.tiles[unext_y][unext_x] = match strength {
          1 => create_cell(Tile::Wall1, pickup, value),
          _ => {
            remaining = 1;
            create_cell(Tile::Empty, pickup, value)
          },
        };
      },
      Cell { tile: Tile::Wall1, pickup, value, visited: _ } => {
        world.tiles[unext_y][unext_x] = create_cell(Tile::Empty, pickup, value); // Or a different powerup?
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

fn move_it_xy(movable: &mut Movable, mslots_maybe: &MinerSlots, meta: &mut MinerMeta, world: &mut World, options: &Options, nextx: i32, nexty: i32, deltax: i32, deltay: i32) {
  let mut was_boring = false; // Did we just move forward? No blocks, no pickups?
  if movable.what == WHAT_MINER {
    meta.points_last_move = 0;
  }

  // If this move would go OOB, expand the world to make sure that does not happen

  // println!("");
  // println!("world A: {:?}", world);
  ensure_cell_in_world(world, options, nextx, nexty);
  // println!("world B: {:?}", world);

  let mut unextx = (world.min_x.abs() + nextx) as usize;
  let mut unexty = (world.min_y.abs() + nexty) as usize;

  // println!("Stepping to: {}x{} ({}x{}) world is {}x{} - {}x{}", nextx, nexty, unextx, unexty, world.min_x, world.min_y, world.max_x, world.max_y);
  // println!("Actual world has {} lines and the first row has {} cols", world.tiles.len(), world.tiles[0].len());
  // println!("Wot? {} + {} = {} -> {}", world.min_y, nexty, world.min_y + nexty, unexty);

  if world.tiles.len() <= unexty { assert_eq!((unexty, "unexty"), (world.tiles.len(), "len"), "OOB: world is not high enough"); }
  if world.tiles[unexty].len() <= unextx { assert_eq!((unextx, "unextx"), (world.tiles[unexty].len(), "len"), "OOB: world is not wide enough"); }
  assert!(world.tiles.len() > unexty);
  assert!(unexty >= 0);
  assert!(world.tiles[unexty].len() > unextx);
  assert!(unextx >= 0);

  let drills = meta.kind_counts[SlotKind::Drill as usize];
  let hammers = meta.kind_counts[SlotKind::Hammer as usize];
  match world.tiles[unexty][unextx] {
    Cell { tile: Tile::Wall4, pickup, value, visited: _ } => {
      world.tiles[unexty][unextx] = match if movable.what == WHAT_MINER { hammers } else { 1 } {
        0 => create_cell(Tile::Wall3, pickup, value),
        1 => create_cell(Tile::Wall2, pickup, value),
        2 => create_cell(Tile::Wall1, pickup, value),
        _ => create_cell(Tile::Empty, pickup, value),
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
    Cell { tile: Tile::Wall3, pickup, value, visited: _ } => {
      world.tiles[unexty][unextx] = match if movable.what == WHAT_MINER { hammers } else { 1 } {
        0 => create_cell(Tile::Wall2, pickup, value),
        1 => create_cell(Tile::Wall1, pickup, value),
        _ => create_cell(Tile::Empty, pickup, value),
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
    Cell { tile: Tile::Wall2, pickup, value, visited: _ } => {
      world.tiles[unexty][unextx] = match if movable.what == WHAT_MINER { hammers } else { 1 } {
        0 => create_cell(Tile::Wall1, pickup, value),
        _ => create_cell(Tile::Empty, pickup, value),
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
    Cell { tile: Tile::Wall1, pickup, value, visited: _ } => {
      world.tiles[unexty][unextx] = create_cell(Tile::Empty, pickup, value); // Or a different powerup?
      movable.dir = get_most_visited_dir_from_xydir(options, world, nextx, nexty, movable.dir);
      movable.now_energy = movable.now_energy - meta.block_bump_cost;
      if movable.what == WHAT_MINER {
        if drills > 0 {
          drill_deeper(drills, hammers, nextx, nexty, deltax, deltay, world, options);
        }
        meta.prev_move_bumped = true;
      }
    },
    Cell { tile: Tile::Empty | Tile::ExpandoWater, pickup, value: pickup_value, visited: _ } => {
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
          world.tiles[unexty][unextx] = create_visited_cell(Tile::Empty, Pickup::Nothing, 0, world.tiles[unexty][unextx].visited);
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
          world.tiles[unexty][unextx] = create_visited_cell(Tile::Empty, Pickup::Nothing, pickup_value, world.tiles[unexty][unextx].visited);
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
          world.tiles[unexty][unextx] = create_visited_cell(Tile::Empty, Pickup::Nothing, pickup_value, world.tiles[unexty][unextx].visited);
          movable.x = nextx;
          movable.y = nexty;
          movable.history.push((nextx, nexty));
        },
        Pickup::Wind => {
          meta.inventory.wind += 1;
          world.tiles[unexty][unextx] = create_visited_cell(Tile::Empty, Pickup::Nothing, pickup_value, world.tiles[unexty][unextx].visited);
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
          world.tiles[unexty][unextx] = create_visited_cell(Tile::Empty, Pickup::Nothing, pickup_value, world.tiles[unexty][unextx].visited);
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
    _ => {
      panic!("wat");
    },
  }

  // Do not remove an expando when moving over it.
  if !matches!(world.tiles[unexty][unextx].tile, Tile::ExpandoWater) {
    match world.tiles[unexty][unextx] {
      Cell {tile: Tile::Empty, pickup: Pickup::Expando, ..} => {
        // This must have been an expando that was just revealed.
        // Set the cell to water tile and add the expando to the world so it can flow.
        world.tiles[unexty][unextx].tile = Tile::ExpandoWater;
        world.expandos.push(create_expando(nextx, nexty));
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

pub fn move_movable(movable: &mut Movable, mslots_maybe: &MinerSlots, meta: &mut MinerMeta, world: &mut World, options: &Options) {
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

    _ => {
      println!("unexpected dir is: {:?}", movable.dir);
      panic!("dir is enum");
    },
  }
}
