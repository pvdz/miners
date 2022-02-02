use super::miner::*;
use super::world::*;

pub struct Biome {
    pub ticks: u32,
    pub world: World,
    pub miner: Miner,

    // The real path this miner has taken in this world
    pub path: Vec<i32>,
}
