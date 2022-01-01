use super::slottable::*;
use super::values::*;
// use super::icons::*;
use super::slot_emptiness::*;
use super::helix::*;
use super::slot_drone_launcher::*;
use super::drone::*;
use super::slot_energy_cell::*;
use super::movable::*;
// use super::world::*;
use super::slot_hammer::*;
use super::slot_drill::*;
use super::slot_purity_scanner::*;
use super::slot_broken_gps::*;

pub type MinerSlots = Vec<Slottable>;

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
    pub max_energy: f32,
    // How many points has the miner accrued so far?
    pub points: i32,
    pub points_last_move: i32, // How many points has the miner gathered last time it moved? Does not include points from drones (or whatever else).

    // Tally of number of slots per kind
    pub kind_counts: Vec<i32>,

    // Increase energy cost per step per boredom level
    // The miner finds plain moves boring and the price for keep doing these will grow until something happens.
    // The rate depends on your max energy. The more energy you have, the more bored you get if nothing happens.
    pub boredom_level: i32, // Current level of boredom, or "number of steps without further action"
    pub boredom_rate: f32, // Cost of making a actionless move, which will be multiplied to the boredom level.

    // Gene: Generate a new drone at this interval
    pub drone_gen_cooldown: i32,

    // TODO: find a meaningful use for this cost
    pub block_bump_cost: f32, // (i32)
    pub prev_move_bumped: bool, // Hack until I figure out how to model this better. If we bumped during a move, all slots should cool down.

    // Gene: How effective are pickups?
    pub multiplier_energy_pickup: i32,

    // Gene: How effective are items (slottables)?
    //  multiplier_cooldown: i32,
}

pub fn create_miner_from_helix(helix: Helix) -> Miner {
    // Given a Helix ("footprint of a miner") return a Miner with those baseline properties
    // Note: this function receives a clone of the helix since the helix will be stored in this miner. TODO: what does the version without cloning look like?

    let max_energy: f32 = ((INIT_ENERGY as f32) * ((100.0 + helix.multiplier_energy_start) as f32) / 100.0);

    // Start with empty slots and populate them with the slots indicated by the helix
    let mut slots: MinerSlots = vec![
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
        create_empty_slot(),
    ];

    assert_eq!(slots.len(), helix.slots.len(), "miner should be initialized to the same number of slots as the helix carries...");

    let mut kind_counts: Vec<i32> = create_slot_kind_counter();

    for i in 0..32 {
        let kind: SlotKind = helix.slots[i];
        let kind_usize = kind as usize;
        let nth: i32 = kind_counts[kind_usize];
        match kind {
            SlotKind::Emptiness => {
                slots[i] = create_empty_slot();
            },
            SlotKind::EnergyCell => {
                slots[i] = create_slot_energy_cell(nth, 100, 100.0 * 2.0_f32.powf(nth as f32));
            },
            SlotKind::DroneLauncher => {
                slots[0] = create_drone_launcher(nth, nth);
                // slots[i] = Box::new(DroneLauncher { drone: Drone { movable: Movable { what: WHAT_DRONE, x: 0, y: 0, dir: DIR_DOWN, energy: 0 } } });
            },
            SlotKind::Hammer => {
                slots[i] = create_hammer(nth);
            },
            SlotKind::Drill => {
                slots[i] = create_drill(nth);
            },
            SlotKind::PurityScanner => {
                slots[i] = create_slot_purity_scanner(nth, 100.0 * 2.0_f32.powf(nth as f32));
            },
            SlotKind::BrokenGps => {
                slots[i] = create_slot_broken_gps(nth, 100.0 * 2.0_f32.powf(nth as f32));
            }
            _ => {
                panic!("Fix slot range generator in helix")
            },
        }

        kind_counts[kind_usize] = kind_counts[kind_usize] + 1;
    }

    return Miner {
        helix,
        movable: Movable {
            what: WHAT_MINER,
            x: 0,
            y: 0,
            dir: DIR_UP,
            energy: max_energy,
        },
        meta: MinerMeta {
            points: 0,
            points_last_move: 0,
            max_energy,

            kind_counts,

            boredom_level: 0,
            boredom_rate: (max_energy as f32).log(2.0),

            drone_gen_cooldown: helix.drone_gen_cooldown as i32,
            block_bump_cost: helix.block_bump_cost,
            prev_move_bumped: false,
            multiplier_energy_pickup: 1, // TODO
        },

        slots,
    };

}

// impl Miner {
//     pub fn paint(miner: &Miner, painting: &mut Grid, symbol: char) {
//         painting[miner.movable.x][miner.movable.y] =
//             if symbol != ' ' {
//                 symbol
//             } else {
//                 match miner.movable.dir {
//                     DIR_UP => ICON_MINER_UP,
//                     DIR_DOWN => ICON_MINER_DOWN,
//                     DIR_LEFT => ICON_MINER_LEFT,
//                     DIR_RIGHT => ICON_MINER_RIGHT,
//                     _ => {
//                         println!("unexpected dir: {:?}", miner.movable.dir);
//                         panic!("dir is enum");
//                     },
//                 }
//             }
//     }
//
//
// }
