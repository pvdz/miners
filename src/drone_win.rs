use super::movable::*;
use super::fountain::*;
use super::tile::*;
use super::options::*;
use super::pickup::*;
use super::world::*;
use super::biome::*;
use super::expando::*;
use super::values::*;
use super::color::*;
use super::icons::*;

#[derive(Debug)]
pub enum WindroneState {
  // Not yet built
  Unconstructed,
  // Waiting for enough wind pickups
  WaitingForWind,
  // Have wind but waiting for available expando
  WaitingForGoal,
  // Temporary state so the miner knows the windrone is about to take off
  ReadyForTakeOff,
  // Flying towards nearest expando
  FlyingToGoal,
  // Finished at expando, flying back to miner
  FlyingHome,
  // Temporary state so the miner knows the windrone returned from a task
  ReturnedHome,
}

#[derive(Debug)]
pub struct Windrone {
  pub state: WindroneState,
  // TBD but right now a sort of freeform desc of what the windrone is doing
  pub status_desc: String,
  // Each drone has its own x, y, direction, and energy
  pub movable: Movable,
}

pub fn create_windrone() -> Windrone {
  return Windrone {
    state: WindroneState::Unconstructed,
    status_desc: "Idle. Waiting for enough wind...".to_string(),
    movable: Movable {
      what: WHAT_WINDRONE,
      x: 0,
      y: 0,
      dir: Direction::Up,
      now_energy: 0.0,
      init_energy: 0.0,
      disabled: false,
    }
  }
}

pub fn set_windrone_state(biome: &mut Biome, state: WindroneState) {
  biome.miner.windrone.status_desc = match state {
    WindroneState::Unconstructed => panic!("A windrone does not deconstruct"),
    WindroneState::WaitingForWind => format!("Waiting for enough wind to take off..."),
    WindroneState::WaitingForGoal => format!("Waiting for expando to move to..."),
    WindroneState::ReadyForTakeOff => format!("Have wind and at least one destination; ready!"),
    WindroneState::FlyingToGoal => format!("Found an expando! Flying to nearest expando..."),
    WindroneState::FlyingHome => format!("Finished at expando. Flying back to miner..."),
    WindroneState::ReturnedHome => format!("Returned to miner"),
  };
  biome.miner.windrone.state = state;
}

fn find_closest_expando(expandos: &Vec<Expando>, bx: f64, by: f64) -> (bool, usize, i32, i32, f64) {
  let mut closest_d = usize::MAX as f64;
  let mut closest_i = 0;
  let mut closest_x = 0;
  let mut closest_y = 0;
  let len = expandos.len();

  let mut found = false;
  for i in 0..len {
    if expandos[i].disabled { continue; }

    found = true;
    let ex = expandos[i].x;
    let ey = expandos[i].y;
    let fx = ex as f64;
    let fy = ey as f64;

    // TODO: I don't think we actually need to sqrt since the absolute value is not relevant (?)
    let d = ((fx - bx).powf(2.0) + (fy - by).powf(2.0)).sqrt();

    // Pick the closest expando. When breaking ties, take the left-most and then the top-most expando. Expandos won't overlap so that should result in one unambiguous winner.
    if
    d == 0.0 ||
      closest_d > d ||
      (
        closest_d == d &&
          (
            (closest_x as f64) < fx ||
              (
                (closest_x as f64) == fx &&
                  (closest_y as f64) == fy
              )
          )
      )
    {
      closest_d = d;
      closest_i = i;
      closest_x = ex;
      closest_y = ey;
    }
  }
  return (found, closest_i, closest_x, closest_y, closest_d);
}

fn find_closest_fountain(fountains: &Vec<Fountain>, bx: f64, by: f64, mut closest_x: i32, mut closest_y: i32, mut closest_d: f64) -> (bool, usize, i32, i32, f64) {
  let len = fountains.len();
  let mut found = false;
  let mut closest_i = 0;

  for i in 0..len {
    if fountains[i].disabled { continue; }

    found = true;
    let ex = fountains[i].x;
    let ey = fountains[i].y;
    let fx = ex as f64;
    let fy = ey as f64;

    // TODO: I don't think we actually need to sqrt since the absolute value is not relevant (?)
    let d = ((fx - bx).powf(2.0) + (fy - by).powf(2.0)).sqrt();

    // Pick the closest expando. When breaking ties, take the left-most and then the top-most expando. Expandos won't overlap so that should result in one unambiguous winner.
    if
    d == 0.0 ||
      closest_d > d ||
      (
        closest_d == d &&
          (
            (closest_x as f64) < fx ||
              (
                (closest_x as f64) == fx &&
                  (closest_y as f64) == fy
              )
          )
      )
    {
      closest_d = d;
      closest_i = i;
      closest_x = ex;
      closest_y = ey;
    }
  }
  return (found, closest_i, closest_x, closest_y, closest_d);
}

pub fn tick_windrone(options: &mut Options, biome: &mut Biome, slot_index: usize) {
  match biome.miner.windrone.state {
    WindroneState::Unconstructed => {

    }
    WindroneState::WaitingForWind => {
      if biome.miner.meta.inventory.wind >= 10 {
        set_windrone_state(biome, WindroneState::WaitingForGoal);
      }
    }
    WindroneState::WaitingForGoal => {
      if biome.world.expandos.len() > 0 {
        // Get it up in the air!
        set_windrone_state(biome, WindroneState::ReadyForTakeOff);
        biome.miner.slots[slot_index].val = 1.0;
        biome.miner.windrone.movable.x = biome.miner.movable.x;
        biome.miner.windrone.movable.y = biome.miner.movable.y;
      }
    }
    WindroneState::ReadyForTakeOff => {
      // Miner tick should act on this
    }
    WindroneState::FlyingToGoal => {
      // Find the nearest expando (birds eye), at every tick
      // Move one step closer to it
      // If on top of it, do stuff.

      // let mut found = false;
      // let mut closest_d = usize::MAX as f64;
      // let mut closest_e = &expandos[0];
      // let mut closest_i = 0;
      // let mut closest_x = expandos[0].x;
      // let mut closest_y = expandos[0].y;

      // Make sure all expandos are still there
      for i in 0..biome.world.expandos.len() {
        if biome.world.expandos[i].disabled { continue; }
        if !matches!(get_cell_tile_at(options, &biome.world, biome.world.expandos[i].x, biome.world.expandos[i].y), Tile::ExpandoWater) {
          biome.world.expandos[i].disabled = true;
        }
      }

      // Make sure all fountains are still there
      for i in 0..biome.world.fountains.len() {
        if biome.world.fountains[i].disabled { continue; }
        if !matches!(get_cell_pickup_at(options, &biome.world, biome.world.fountains[i].x, biome.world.fountains[i].y), Pickup::Fountain) {
          biome.world.fountains[i].disabled = true;
        }
      }

      let bx = biome.miner.windrone.movable.x;
      let by = biome.miner.windrone.movable.y;
      let bfx = bx as f64;
      let bfy = by as f64;
      let (found_a, closest_i_a, closest_x_a, closest_y_a, closest_d_a) = find_closest_expando(&biome.world.expandos, bfx, bfy);
      let (found_b, closest_i, closest_x, closest_y, closest_d) = find_closest_fountain(&biome.world.fountains, bfx, bfy, closest_x_a, closest_y_a, closest_d_a);

      if (found_a || found_b) && move_windrone_towards(&mut biome.miner.windrone, closest_x, closest_y, ) {
        // Windrone reached an expando or fountain. Replace it.

        // Disable the windrone. No longer flying.
        set_windrone_state(biome, WindroneState::FlyingHome);

        // Check whether the windrone is on a targeted tile right now
        if bx == closest_x && by == closest_y {
          // Check the kind of cell
          let cell = get_cell_stuff_at(options, &biome.world, closest_x, closest_y);
          match cell {
            (_, Pickup::Fountain, pickup_value, ..) => {
              // let mut found = 0;
              // The cell pickup value should reflect the index of the fountain.
              // Convert all water pickups, return that as a number
              for (wx, wy) in biome.world.fountains[pickup_value as usize].water_tiles.to_owned().iter() {
                let stuff = get_cell_stuff_at(options, &biome.world, *wx, *wy);
                if matches!(stuff.1, Pickup::Water) {
                  set_cell_pickup_at(options, &mut biome.world, *wx, *wy, Pickup::Nothing);
                  // found += 1;
                  // break;
                }
              }
            },
            (Tile::ExpandoWater, ..) => {
              // Pop the element and swap it with the closest (if not already last). This will drop closest.
              let last = biome.world.expandos.pop();
              if closest_i_a != biome.world.expandos.len() {
                // We know there must be at least two expandos since the closest one wasn't last on the list.
                biome.world.expandos[closest_i_a] = match last {
                  Some(expando) => expando,
                  None => panic!("{:?}", assert!(false, "cannot happen")),
                };
              }

              let new_fountain_index = biome.world.fountains.len();

              // Add the replacement to the world. Working title is "fountain". Set its value to the index. Clear the tile.
              set_cell_pickup_at(options, &mut biome.world, biome.miner.windrone.movable.x, biome.miner.windrone.movable.y, Pickup::Fountain);
              set_cell_pickup_value_at(options, &mut biome.world, biome.miner.windrone.movable.x, biome.miner.windrone.movable.y, new_fountain_index as u32); // This will break if we ever start popping/shuffling the fountains vec
              set_cell_tile_at(options, &mut biome.world, biome.miner.windrone.movable.x, biome.miner.windrone.movable.y, Tile::Empty);
              set_cell_tile_value_at(options, &mut biome.world, biome.miner.windrone.movable.x, biome.miner.windrone.movable.y, 0);

              // Add a fountain for this coord into the vec for this world
              // And it will start doing stuff...
              let fountain = create_fountain(options, biome);
              biome.world.fountains.push(fountain);
            },
            _ => panic!("Expected to be at a particular cell of interest ... {:?} {} {} {} {} {} {} {} {} {} {} {} {}", cell, found_a, closest_i_a, closest_x_a, closest_y_a, closest_d_a, found_b, closest_i, closest_x, closest_y, closest_d, biome.world.expandos.len(), biome.world.fountains.len()),
          }
        }
      }
    }
    WindroneState::FlyingHome => {
      // println!("homing back from {},{} to {},{}", windrone.movable.x, windrone.movable.y, mx, my);
      // Fly back to the miner. Stop flying as soon as you hit the same coord (or next to it?).
      if move_windrone_towards(&mut biome.miner.windrone, biome.miner.movable.x, biome.miner.movable.y) {
        biome.miner.windrone.state = WindroneState::ReturnedHome;
        biome.miner.windrone.status_desc = format!("Idle. Waiting for enough wind...");
      }
    }
    WindroneState::ReturnedHome => {
      // Miner tick should pick up on this one
    }
  }
}

pub fn ui_windrone(_sandrone: &Windrone, options: &Options) -> String {
  return add_fg_color_with_reset(&format!("{}", ICON_WINDRONE), COLOR_WIND, options);
}

fn move_windrone_towards(windrone: &mut Windrone, to_x: i32, to_y: i32) -> bool {
  let bx = windrone.movable.x;
  let by = windrone.movable.y;

  if bx == to_x && by == to_y {
    return true;
  } else {
    // Now move closer to the closest expando
    let x1 = to_x as f64;
    let x2 = bx as f64;
    let y1 = to_y as f64;
    let y2 = by as f64;

    // Windrone still on its way to the expando...
    let dx = x1 - x2;
    let dy = y1 - y2;
    if dx.abs() > dy.abs() {
      windrone.movable.x += if dx < 0.0 { -1 } else { 1 };
    } else {
      windrone.movable.y += if dy < 0.0 { -1 } else { 1 };
    }
    return false;
  }
}
