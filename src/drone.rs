use crate::miner::MinerMeta;
use crate::options::Options;
use crate::world::World;
use super::movable::*;

pub struct Drone {
  // Each drone has its own x, y, direction, and energy
  pub movable: Movable,
}

pub fn tick_drone(drone: &mut Drone, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World, options: &Options) {
  move_movable(&mut drone.movable, miner_meta, world, &options);
}

