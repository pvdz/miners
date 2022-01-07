// Should drones pick up anything at all? Or be specialized? (Drones that mine, drones that collect)


use std::fmt::Write;

use std::fmt;
use rand_pcg::{Lcg128Xsl64};
use rand::distributions::{Distribution, Uniform};

use super::miner::*;
use super::options::*;
use super::slottable::*;

#[derive(Debug)]
pub struct Inventory {
    stone_white: u32,
    stone_blue: u32,
    stone_green: u32,

    diamonds_white: u32,
    diamonds_blue: u32,
    diamonds_green: u32,
}

// impl fmt::Display for Inventory {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Helix [ drone gen: {}, energy start: {}, points: {}, bump cost: {}, energy pickups: {} ]", self.drone_gen_cooldown, self.multiplier_energy_start, self.multiplier_points, self.block_bump_cost, self.multiplier_energy_pickup)
//     }
// }


pub fn create_inventory() -> Inventory {
    return Inventory {
        stone_white: 0,
        stone_blue: 0,
        stone_green: 0,

        diamonds_white: 0,
        diamonds_blue: 0,
        diamonds_green: 0,
    };
}
