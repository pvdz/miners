use std::fmt;
use rand_pcg::{Lcg128Xsl64};
use rand::distributions::{Distribution, Uniform};

use crate::miner::*;

/**
 * Describe the genes for a single Miner instantiation
 */
#[derive(Clone, Copy)]
pub struct Helix {
    // Arguably this should be a drone launcher property
    // Gene: Generate a new drone at this interval
    pub drone_gen_cooldown: f32,

    // Gene: How much energy does the miner start with
    pub multiplier_energy_start: f32,

    // Gene: How many fast does the miner receive points
    pub multiplier_points: f32,

    // Gene: How expensive is it to bump against a block?
    pub block_bump_cost: f32,

    // Gene: How effective are pickups?
    pub multiplier_energy_pickup: f32,

    // Gene: How effective are items (slottables)?
    //  multiplier_cooldown: i32,

    pub slots: [i32; 32],
}

impl fmt::Display for Helix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Helix [ drone gen: {}, energy start: {}, points: {}, bump cost: {}, energy pickups: {} ]", self.drone_gen_cooldown, self.multiplier_energy_start, self.multiplier_points, self.block_bump_cost, self.multiplier_energy_pickup)
    }
}

pub fn create_initial_helix(rng: &mut Lcg128Xsl64) -> Helix {
    let multiplier_percent: Uniform<f32> = Uniform::from(0.0..100.0);
    let multiplier_slot_type: Uniform<i32> = Uniform::from(0..5);

    return Helix {
        drone_gen_cooldown: multiplier_percent.sample(rng),
        multiplier_energy_start: multiplier_percent.sample(rng),
        multiplier_points: 0f32, // multiplier_percent.sample(rng),
        block_bump_cost: multiplier_percent.sample(rng),
        multiplier_energy_pickup: 0.0, // multiplier_percent.sample(rng),
        slots: [
            multiplier_slot_type.sample(rng),
            multiplier_slot_type.sample(rng),
            multiplier_slot_type.sample(rng),
            multiplier_slot_type.sample(rng),
            multiplier_slot_type.sample(rng),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    }
}

pub fn mutated_helix(rng: &mut Lcg128Xsl64, helix: Helix) -> Helix {
    // Modify each gene by up to 5%, up or down. Make sure the final value does not underflow or overflow.
    let multiplier_5_percent: Uniform<f32> = Uniform::from(0.0..10.0);
    let multiplier_slot_type: Uniform<i32> = Uniform::from(0..3);
    let slot_odds = 0.1;

    return Helix {
        drone_gen_cooldown: (helix.drone_gen_cooldown + (multiplier_5_percent.sample(rng) - 5.0)).max(0.0),
        multiplier_energy_start: (helix.multiplier_energy_start + (multiplier_5_percent.sample(rng) - 5.0)).max(0.0),
        multiplier_points: 0f32, // (helix.multiplier_points + (multiplier_5_percent.sample(rng) - 5.0)).max(0.0),
        block_bump_cost: (helix.block_bump_cost + (multiplier_5_percent.sample(rng) - 5.0)).max(0.0),
        multiplier_energy_pickup: 0.0, // (helix.multiplier_energy_pickup + (multiplier_5_percent.sample(rng) - 5.0)).max(0.0),
        slots: [
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[0] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[1] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[2] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[3] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[4] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[5] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[6] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[7] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[8] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[9] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[10] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[11] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[12] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[13] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[14] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[15] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[16] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[17] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[18] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[19] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[11] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[20] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[21] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[22] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[23] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[24] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[25] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[26] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[27] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[28] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[30] },
            if multiplier_5_percent.sample(rng) < slot_odds { multiplier_slot_type.sample(rng) } else { helix.slots[31] },
        ],
    }
}

pub fn fitness(miner: Miner) -> i32 {
  return miner.meta.points;
}
