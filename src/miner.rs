use super::slottable::*;
use super::values::*;
use super::slot_emptiness::*;
use super::helix::*;
use super::slot_drone_launcher::*;
use super::slot_energy_cell::*;
use super::movable::*;
use super::slot_hammer::*;
use super::slot_drill::*;
use super::slot_purity_scanner::*;
use super::slot_broken_gps::*;
use super::inventory::*;
use super::drone::*;

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

    pub drones: Vec<Drone>,
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

    // Inventory of this miner
    pub inventory: Inventory,

    // How many points has the miner accrued so far?
    // pub points: i32,
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

    let max_energy: f32 = (INIT_ENERGY as f32) * ((100.0 + helix.multiplier_energy_start) as f32) / 100.0;

    // Start with empty slots and populate them with the slots indicated by the helix
    let mut slots: MinerSlots = vec![
        create_empty_slot(0),
        create_empty_slot(1),
        create_empty_slot(2),
        create_empty_slot(3),
        create_empty_slot(4),
        create_empty_slot(5),
        create_empty_slot(6),
        create_empty_slot(7),
        create_empty_slot(8),
        create_empty_slot(9),
        create_empty_slot(10),
        create_empty_slot(11),
        create_empty_slot(12),
        create_empty_slot(13),
        create_empty_slot(14),
        create_empty_slot(15),
        create_empty_slot(16),
        create_empty_slot(17),
        create_empty_slot(18),
        create_empty_slot(19),
        create_empty_slot(20),
        create_empty_slot(21),
        create_empty_slot(22),
        create_empty_slot(23),
        create_empty_slot(24),
        create_empty_slot(25),
        create_empty_slot(26),
        create_empty_slot(27),
        create_empty_slot(28),
        create_empty_slot(29),
        create_empty_slot(30),
        create_empty_slot(31),
    ];

    assert_eq!(slots.len(), helix.slots.len(), "miner should be initialized to the same number of slots as the helix carries...");

    let mut kind_counts: Vec<i32> = create_slot_kind_counter();

    let mut drones: Vec<Drone> = vec!();

    for i in 0..32 {
        let kind: SlotKind = helix.slots[i];
        let kind_usize = kind as usize;
        let nth: i32 = kind_counts[kind_usize];
        match kind {
            SlotKind::Emptiness => {
                slots[i] = create_empty_slot(i);
            },
            SlotKind::EnergyCell => {
                slots[i] = create_slot_energy_cell(i, nth, 100, 100.0 * 2.0_f32.powf((nth + 1) as f32));
            },
            SlotKind::DroneLauncher => {
                slots[i] = create_drone_launcher(i, nth, nth, helix.drone_gen_cooldown * 2.0_f32.powf(((nth as f32/2.0) + 1.0) as f32));
                drones.push(Drone {
                    movable: Movable {
                        what: WHAT_DRONE,
                        x: 0,
                        y: 0,
                        dir: Direction::Up,
                        now_energy: 0.0,
                        init_energy: 0.0,
                        history: vec!((0,0)),
                        unique: vec!((0,0)),
                    },
                });
                assert_eq!(drones.len() - 1, nth as usize, "there should be as many drones as there are drone launchers");
            },
            SlotKind::Hammer => {
                slots[i] = create_hammer(i, nth);
            },
            SlotKind::Drill => {
                slots[i] = create_drill(i, nth);
            },
            SlotKind::PurityScanner => {
                slots[i] = create_slot_purity_scanner(i, nth, 100.0 * 2.0_f32.powf((nth + 1) as f32));
            },
            SlotKind::BrokenGps => {
                slots[i] = create_slot_broken_gps(i, nth, 100.0 * 2.0_f32.powf((nth + 1) as f32));
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
            dir: Direction::Up,
            now_energy: max_energy,
            init_energy: max_energy,
            history: vec!((0,0)),
            unique: vec!((0,0)),
        },
        meta: MinerMeta {
            points_last_move: 0,
            max_energy,

            inventory: create_inventory(),

            kind_counts,

            boredom_level: 0,
            boredom_rate: (max_energy as f32).log(2.0),

            drone_gen_cooldown: helix.drone_gen_cooldown as i32,
            block_bump_cost: helix.block_bump_cost,
            prev_move_bumped: false,
            multiplier_energy_pickup: 1, // TODO
        },

        slots,
        drones,
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
