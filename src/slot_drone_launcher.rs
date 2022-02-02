use super::utils::*;
use super::slottable::*;
use super::movable::*;
use super::miner::*;
use super::world::*;
use super::drone::*;
use super::options::*;

pub const TITLE_DRONE_LAUNCHER: &str = "Drone Launcher";
pub const DRONE_INITIAL_ENERGY: f32 = 1000.0;

pub fn create_drone_launcher(slot_index: usize, nth: i32, drone_id: i32, max_cooldown: f32) -> Slottable {
  return Slottable {
    kind: SlotKind::DroneLauncher,
    slot: slot_index,
    title: TITLE_DRONE_LAUNCHER.to_owned(),
    max_cooldown,
    cur_cooldown: 0.0,
    nth,
    val: drone_id as f32,
    sum: 0.0,
  };
}

pub fn tick_slot_drone_launcher(slot: &mut Slottable, miner_movable: &mut Movable, drones: &mut Vec<Drone>, miner_meta: &mut MinerMeta, world: &mut World, options: &mut Options, _first_miner: bool) {
  // TODO: this function has access to all drones but it should really only have access to its own. :shrug:?

  let nth = slot.nth;
  assert!(drones.len() > nth as usize, "Each drone launcher should at least have one drone in the list");
  let drone: &mut Drone = &mut drones[nth as usize];

  // There always exists a drone for this launcher but it may not be operable
  // (not (yet) re/launched or out of energy). First check for that and launch if the miner has
  // enough energy and the launcher progress is 100%. When the drone is in flight, just tick() it.

  if drone.movable.now_energy > 0.0 {
    // if first_miner { println!("slot {} drone {} has energy: {}", slot.slot, nth, drone.movable.now_energy); }
    // Drone is alive so tick it. The launcher is irrelevant right now.
    tick_drone(drone, miner_movable, miner_meta, world, options);
  } else {
    // If the launcher is charged and the miner has enough energy, launch another drone
    if slot.cur_cooldown >= slot.max_cooldown {
      if miner_movable.now_energy > 2.0 * DRONE_INITIAL_ENERGY {
        drone.movable.now_energy = DRONE_INITIAL_ENERGY;
        miner_movable.now_energy -= DRONE_INITIAL_ENERGY / 2.0; // TODO: this ratio can be a tool to act as penalty for a helix property
        // Position the drone on your location, facing perpendicular from your current direction
        drone.movable.x = miner_movable.x;
        drone.movable.y = miner_movable.y;
        drone.movable.dir = match miner_movable.dir {
          Direction::Up => Direction::Right,
          Direction::Right => Direction::Down,
          Direction::Down => Direction::Left,
          Direction::Left => Direction::Up,
        };
        // Reset the cooldown. It will be ignored until the drone runs out of energy.
        slot.cur_cooldown = 0.0;
      } else {
        // Do nothing. Wait until the miner energy goes over the threshold again, which it
        // may not do anymore, but that's not relevant.
      }
    } else {
      slot.cur_cooldown += 1.0;
    }
  }
}

pub fn ui_slot_drone_launcher(slot: &Slottable, drone: &Drone) -> (String, String, String) {
  if drone.movable.now_energy > 0.0 {
    return (
      TITLE_DRONE_LAUNCHER.to_string(),
      progress_bar(20, drone.movable.now_energy, DRONE_INITIAL_ENERGY, false),
      format!("Drone is flying at {}x{}", drone.movable.x, drone.movable.y)
    );
  }

  return (
    TITLE_DRONE_LAUNCHER.to_string(),
    progress_bar(20, slot.cur_cooldown, slot.max_cooldown, false),
    format!("Waiting to launch new drone...")
  );
}
