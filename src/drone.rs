use crate::miner::MinerMeta;
use crate::options::Options;
use crate::world::World;
use super::movable::*;

pub struct Drone {
  // Each drone has its own x, y, direction, and energy
  pub movable: Movable,
}

pub fn tick_drone(drone: &mut Drone, _miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World, options: &mut Options) {
  let v = vec!();
  move_movable(&mut drone.movable, &v, miner_meta, world, options, None, false, 0, 0, 0, 0);
}

