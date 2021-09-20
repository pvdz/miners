use super::slottable::*;
use super::values::*;
use super::emptiness::*;
use super::helix::*;
use super::drone_launcher::*;
use super::drone::*;
use super::energy_cell::*;
use super::movable::*;
use super::world::*;
use super::hammer::*;
use super::drill::*;
use super::purity_scanner::*;

pub type MinerSlots = [Box<Slottable>; 32];

pub struct Miner {
    // The genes that generated this miner
    pub helix: Helix,

    // The move details for this miner. Basically "inherits from movable".
    pub movable: Movable,

    // Miner specific properties
    pub meta: MinerMeta,

    // The items the miner is carrying
    pub slots: MinerSlots,
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
    pub points_last_move: i32, // How many points has the miner gathered last time it moved? Does not include points from drones (or whatever else).

    // Number of hammer slots (determines bump strength)
    pub hammers: i32,
    // Number of drill slots (determines how far back you bump)
    pub drills: i32,

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

    let mut slots: MinerSlots = [
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
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
        Box::new(Emptiness { }),
    ];

    let mut energy_cells = 0;
    let mut hammers = 0;
    let mut drills = 0;
    let mut scanners = 0;
    for i in 0..32 {
        match helix.slots[i] {
            SlotType::Emptiness => {
                slots[i] = Box::new(Emptiness {})
            },
            SlotType::EnergyCell => {
                slots[i] = Box::new(EnergyCell { energy_bonus: 100, max_cooldown: 100 * 2.0_f32.powf(energy_cells as f32) as i32, cooldown: 0, nth: energy_cells, generated: 0 });
                energy_cells = energy_cells + 1;
            },
            SlotType::DroneLauncher => {
                slots[i] = Box::new(DroneLauncher { drone: Drone { movable: Movable { what: WHAT_DRONE, x: 0, y: 0, dir: DIR_DOWN, energy: 0 } } });
            },
            SlotType::Hammer => {
                slots[i] = Box::new(Hammer {});
                hammers = hammers + 1;
            },
            SlotType::Drill => {
                slots[i] = Box::new(Drill {});
                drills = drills + 1;
            },
            SlotType::PurityScanner => {
                slots[i] = Box::new(PurityScanner { nth: scanners, max_cooldown: 100 * 2.0_f32.powf(scanners as f32) as i32, cooldown: 0, generated: 0 });
                scanners = scanners + 1;
            },
            _ => {
                panic!("Fix slot range generator in helix")
            },
        }
    }

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
            points_last_move: 0,
            max_energy,

            hammers,
            drills,

            boredom_level: 0,
            boredom_rate: (max_energy as f32).log(2.0),

            drone_gen_cooldown: helix.drone_gen_cooldown as i32,
            block_bump_cost: helix.block_bump_cost as i32,
            multiplier_energy_pickup: 1, // TODO
        },

        slots,
    };

}

pub fn paint(miner: &Miner, painting: &mut Grid, symbol: char) {
    painting[miner.movable.x][miner.movable.y] =
        if symbol != ' ' {
            symbol
        } else {
            match miner.movable.dir {
                DIR_UP => ICON_MINER_UP,
                DIR_DOWN => ICON_MINER_DOWN,
                DIR_LEFT => ICON_MINER_LEFT,
                DIR_RIGHT => ICON_MINER_RIGHT,
                _ => {
                    println!("unexpected dir: {:?}", miner.movable.dir);
                    panic!("dir is enum");
                },
            }
        }
}
