use crate::slottable::*;
use crate::movable::*;

pub struct Miner {
    pub movable: Movable,
    pub meta: MinerMeta,
    // Whenever a drone is generated it will take a chunk of energy from the miner
    pub slots: [Box<Slottable>; 32],
}

pub struct MinerMeta {
    pub max_energy: i32,
    pub points: i32,
    //  item:
    //  cooldown: i32, // Iterations before item can be used again
    pub drone_gen_cooldown: i32, // Generate a new drone every this many ticks

    // Increase energy cost per step per boredom level
    // The miner gets bored if it hasn't seen anything in a while. Prevents endless loops
    pub boredom_level: i32, // Current level of boredom
    pub boredom_steps: i32, // Current number of boring steps
    pub boredom_rate: i32, // Number of boring steps after which the boredom level goes up

    // These multipliers are in whole percentages
    pub multiplier_energy_start: i32,
    pub multiplier_points: i32,
    pub block_bump_cost: i32,
    pub multiplier_energy_pickup: i32,
    //  multiplier_cooldown: i32,
}
