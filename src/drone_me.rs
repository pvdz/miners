use crate::miner::*;
use crate::options::*;
use crate::world::*;
use super::movable::*;
use super::tile::*;

pub struct MeDrone {
  // Each drone has its own x, y, direction, and energy
  pub movable: Movable,
}

pub fn tick_me_drone(ticks: u32, drone: &mut MeDrone, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World, options: &mut Options) {
  let v = vec!();
  move_movable(ticks, &mut drone.movable, &v, miner_meta, world, options, None, false, false, 0, 0, 0, 0);
  if matches!(get_cell_tile_at(options, world, drone.movable.x, drone.movable.y), Tile::Soil) {
    // Collect the current value as food / grass / plants / whatever
    // Reset the value.
    let soil_value = get_cell_tile_value_at(options, world, drone.movable.x, drone.movable.y);
    set_cell_tile_value_at(options, world, drone.movable.x, drone.movable.y, 0);
    if soil_value > 2 {
      miner_meta.inventory.food += 1; // 1? Depends on state of soil and items, I guess.
      miner_movable.now_energy += 100.0; // ? TBD
    }
  }

}

