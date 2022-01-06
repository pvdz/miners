use std::fmt::Write;

use std::fmt;
use rand_pcg::{Lcg128Xsl64};
use rand::distributions::{Distribution, Uniform};

use super::miner::*;
use super::options::*;
use super::slottable::*;

/**
 * Describe the genes for a single Miner instantiation
 */
#[derive(Clone, Copy)]
pub struct Helix {
    // Seed used to randomly generate this helix. Zero if it was derived from another helix.
    pub seed: u64,

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

    pub slots: [SlotKind; 32],
}

impl fmt::Display for Helix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Helix [ drone gen: {}, energy start: {}, points: {}, bump cost: {}, energy pickups: {} ]", self.drone_gen_cooldown, self.multiplier_energy_start, self.multiplier_points, self.block_bump_cost, self.multiplier_energy_pickup)
    }
}

fn slots_string(slots: [SlotKind; 32]) -> String  {
    slots.iter().map(|slot| slot_type_to_symbol(&slot)).collect()
}

pub fn create_initial_helix(rng: &mut Lcg128Xsl64, seed: u64) -> Helix {
    let multiplier_percent: Uniform<f32> = Uniform::from(0.0..100.0);

    let h = Helix {
        seed,
        drone_gen_cooldown: multiplier_percent.sample(rng).round(),
        multiplier_energy_start: multiplier_percent.sample(rng).round(),
        multiplier_points: 0f32, // multiplier_percent.sample(rng),
        block_bump_cost: multiplier_percent.sample(rng).max(1.0).round(),
        multiplier_energy_pickup: 0.0, // multiplier_percent.sample(rng),
        slots: [
            get_random_slot(rng),
            get_random_slot(rng),
            get_random_slot(rng),
            get_random_slot(rng),
            get_random_slot(rng),
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
            SlotKind::Emptiness,
        ],
    };

    return h;
}

fn mutate_gen_maybe(current: f32, roll: f32, options: &Options) -> f32 {
    // Roll is 0..100
    // Move the value up or down by 5%
    // Return a rounded value
    // Do not underflow
    let delta = (roll / 100.0) * (2.0 * options.mutation_rate_genes) - options.mutation_rate_genes;
    let mutated = current + delta;
    return mutated.round().max(0.0);
}

fn mutate_slot_maybe(current: SlotKind, roll: f32, rng: &mut Lcg128Xsl64, options: &Options) -> SlotKind {
    if roll < options.mutation_rate_slots {
        get_random_slot(rng)
    } else {
        current
    }
}

pub fn mutate_helix(rng: &mut Lcg128Xsl64, helix: Helix, options: &Options) -> Helix {
    // Modify each gene by up to x%, up or down. Make sure the final value does not underflow or overflow.
    let pct_roller: Uniform<f32> = Uniform::from(0.0..100.0);

    return Helix {
        seed: 0,
        drone_gen_cooldown: mutate_gen_maybe(helix.drone_gen_cooldown, pct_roller.sample(rng), options),
        multiplier_energy_start: mutate_gen_maybe(helix.multiplier_energy_start, pct_roller.sample(rng), options),
        multiplier_points: 0.0,
        block_bump_cost: mutate_gen_maybe(helix.block_bump_cost, pct_roller.sample(rng), options).max(1.0),
        multiplier_energy_pickup: 0.0,
        slots: [
            mutate_slot_maybe(helix.slots[0], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[1], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[2], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[3], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[4], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[5], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[6], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[7], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[8], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[9], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[10], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[11], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[12], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[13], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[14], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[15], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[16], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[17], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[19], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[20], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[21], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[22], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[23], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[24], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[25], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[26], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[27], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[28], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[29], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[30], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[31], pct_roller.sample(rng), rng, options),
            mutate_slot_maybe(helix.slots[18], pct_roller.sample(rng), rng, options),
        ],
    }
}

pub fn fitness(miner: Miner) -> i32 {
  return miner.meta.points;
}

pub fn helix_to_string(into: &mut String, helix: &Helix) {
    // let mut out: String = "".to_string();
    write!(into, "Helix {{ drone gen: {}, energy start: {}, points: {}, bump cost: {}, energy pickups: {}, slots: {} }}",
           helix.drone_gen_cooldown,
           helix.multiplier_energy_start,
           helix.multiplier_points,
           helix.block_bump_cost,
           helix.multiplier_energy_pickup,
           slots_string(helix.slots),
    ).unwrap();
}
