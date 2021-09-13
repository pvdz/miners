use crate::slottable::*;
use crate::*;
use crate::values::*;
use crate::emptiness::*;
use crate::helix::*;
use crate::drone_launcher::*;
use crate::drone::*;
use crate::energy_cell::*;
use crate::movable::*;

pub struct Miner {
    // The genes that generated this miner
    pub helix: Helix,

    // The move details for this miner. Basically "inherits from movable".
    pub movable: Movable,

    // Miner specific properties
    pub meta: MinerMeta,

    // The items the miner is carrying
    pub slots: [Box<Slottable>; 32],
}

/**
 * This structure exists to work around the Rust rule that each object may have either
 * one write reference or any read references at any time, for any object recursively.
 *
 * Since we want to pass around drones and the miner to a function generically, but
 * always the miner too to update points, we have to to separate it into its own object.
 */
pub struct MinerMeta {
    // A miner may not exceed its initial energy
    pub max_energy: i32,
    // How many points has the miner accrued so far?
    pub points: i32,

    // Increase energy cost per step per boredom level
    // The miner finds plain moves boring and the price for keep doing these will grow until something happens.
    // The rate depends on your max energy. The more energy you have, the more bored you get if nothing happens.
    pub boredom_level: i32, // Current level of boredom, or "number of steps without further action"
    pub boredom_rate: f32, // Cost of making a actionless move, which will be multiplied to the boredom level.

    // Gene: Generate a new drone at this interval
    pub drone_gen_cooldown: i32,

    // TODO: find a meaningful use for this cost
    pub block_bump_cost: i32,

    // Gene: How effective are pickups?
    pub multiplier_energy_pickup: i32,

    // Gene: How effective are items (slottables)?
    //  multiplier_cooldown: i32,
}

pub fn create_miner_from_helix(helix: Helix) -> Miner {
    let max_energy = ((INIT_ENERGY as f32) * ((100.0 + helix.multiplier_energy_start) as f32) / 100.0) as i32;

    return Miner {
        helix,
        movable: Movable {
            what: WHAT_MINER,
            x: WIDTH >> 1,
            y: HEIGHT >> 1,
            dir: DIR_UP,
            energy: max_energy,
        },
        meta: MinerMeta {
            points: 0,
            max_energy,

            boredom_level: 0,
            boredom_rate: (max_energy as f32).log(2.0),

            drone_gen_cooldown: helix.drone_gen_cooldown as i32,
            block_bump_cost: helix.block_bump_cost as i32,
            multiplier_energy_pickup: 1, // TODO
        },

        slots: [
            Box::new(DroneLauncher { drone: Drone { movable: Movable { what: values::WHAT_DRONE, x: 0, y: 0, dir: values::DIR_DOWN, energy: 0 } } }),
            Box::new(DroneLauncher { drone: Drone { movable: Movable { what: values::WHAT_DRONE, x: 0, y: 0, dir: values::DIR_DOWN, energy: 0 } } }),
            Box::new(DroneLauncher { drone: Drone { movable: Movable { what: values::WHAT_DRONE, x: 0, y: 0, dir: values::DIR_DOWN, energy: 0 } } }),
            Box::new(DroneLauncher { drone: Drone { movable: Movable { what: values::WHAT_DRONE, x: 0, y: 0, dir: values::DIR_DOWN, energy: 0 } } }),
            Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
            Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
            Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
            Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
            Box::new(Emptiness { }),
        ],
    };

}



// let start_energy = (best_miner.meta.max_energy as f64 * (1.0 + multiplier_energy_start as f64 / 100.0)) as i32;
// let mut miner: miner::Miner = miner::Miner {
// movable: Movable {
// what: values::WHAT_MINER,
// x: values::WIDTH >> 1,
// y: values::HEIGHT >> 1,
// dir: values::DIR_UP,
// energy: start_energy,
// },
// meta: miner::MinerMeta {
// max_energy: start_energy,
// points: 0,
// boredom_level: 0,
// boredom_rate: 0,
// boredom_steps: 0,
// drone_gen_cooldown: 50,
// multiplier_energy_start,
// multiplier_points,
// multiplier_energy_pickup,
// block_bump_cost: 5,
// },
//
// slots: [
// Box::new(DroneLauncher { drone: Drone { movable: Movable { what: values::WHAT_DRONE, x: 0, y: 0, dir: values::DIR_DOWN, energy: 0 } } }),
// Box::new(DroneLauncher { drone: Drone { movable: Movable { what: values::WHAT_DRONE, x: 0, y: 0, dir: values::DIR_DOWN, energy: 0 } } }),
// Box::new(DroneLauncher { drone: Drone { movable: Movable { what: values::WHAT_DRONE, x: 0, y: 0, dir: values::DIR_DOWN, energy: 0 } } }),
// Box::new(DroneLauncher { drone: Drone { movable: Movable { what: values::WHAT_DRONE, x: 0, y: 0, dir: values::DIR_DOWN, energy: 0 } } }),
// Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
// Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
// Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
// Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100, cooldown: 0 }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// Box::new(emptiness::Emptiness { }),
// ],
// };
