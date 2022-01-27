use super::slottable::*;
use super::hydrone::*;
use super::slot_windrone::*;
use super::slot_hydrone::*;
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
use super::windrone::*;

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

  pub windrone: Windrone,
  pub hydrone: Hydrone,
}

/**
 * This structure exists to work around the Rust rule that each object may have either
 * one write reference or any read references at any time, for any object recursively.
 *
 * Since we want to pass around drones and the miner to a function generically, but
 * always the miner too to update points, we have to to separate it into its own object.
 */
#[derive(Debug)]
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
  pub boredom_level: i32,
  // Current level of boredom, or "number of steps without further action"
  pub boredom_rate: f32, // Cost of making a actionless move, which will be multiplied to the boredom level.

  // Gene: Generate a new drone at this interval
  pub drone_gen_cooldown: i32,

  // TODO: find a meaningful use for this cost
  pub block_bump_cost: f32,
  // (i32)
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
        slots[i] = create_drone_launcher(i, nth, nth, helix.drone_gen_cooldown * 2.0_f32.powf(((nth as f32 / 2.0) + 1.0) as f32));
        drones.push(Drone {
          movable: Movable {
            what: WHAT_DRONE,
            x: 0,
            y: 0,
            dir: Direction::Up,
            now_energy: 0.0,
            init_energy: 0.0,
            history: vec!((0, 0)),
            unique: vec!((0, 0)),
            disabled: false,
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
      },
      SlotKind::Windrone => {
        panic!("The windrone is not a valid starting slot");
      }
      SlotKind::Hydrone => {
        panic!("The hydrone is not a valid starting slot");
      }
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
      history: vec!((0, 0)),
      unique: vec!((0, 0)),
      disabled: false,
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
    windrone: create_windrone(),
    hydrone: create_hydrone(),
  };
}

fn can_craft_windrone(meta: &MinerMeta, slots: &Vec<Slottable>) -> bool {
  let mut has_empty = false;
  for i in 0..slots.len() {
    match slots[i].kind {
      SlotKind::Windrone => return false,
      SlotKind::Emptiness => has_empty = true,
      _ => {},
    }
  }

  // Must have an available slot for the windrone
  if !has_empty { return false; }

  // Must have enough materials to craft a windrone
  return meta.inventory.wood > 5 && (meta.inventory.stone_white + meta.inventory.stone_blue + meta.inventory.stone_green + meta.inventory.stone_yellow) > 5;
}
fn can_craft_hydrone(meta: &MinerMeta, slots: &Vec<Slottable>) -> bool {
  let mut has_empty = false;
  for i in 0..slots.len() {
    match slots[i].kind {
      SlotKind::Hydrone => return false,
      SlotKind::Emptiness => has_empty = true,
      _ => {},
    }
  }

  // Must have an available slot for the hydrone
  if !has_empty { return false; }

  // Must have enough materials to craft a hydrone
  return meta.inventory.wood > 5 && meta.inventory.water > 10 && (/*meta.inventory.stone_white + */meta.inventory.stone_blue + meta.inventory.stone_green + meta.inventory.stone_yellow) > 5;
}

pub fn tick_miner(movable: &mut Movable, meta: &mut MinerMeta, slots: &mut MinerSlots, windrone: &mut Windrone, hydrone: &mut Hydrone) {
  // If;
  // - There are slots available
  // - The build drone was built
  // - The build drone is currently not used
  // - There are enough resources
  // then send a drone to build something on the expando

  // The build drone needs to be crafted using wood and stone.

  // For now the drone can phase through walls but later we'll want to add some pathfinding.

  match windrone.state {
    WindroneState::Unconstructed => {
      if can_craft_windrone(meta, slots) {
        // Deduct materials
        meta.inventory.wood -= 5;
        let mut left = 5;
        let mut next = meta.inventory.stone_white.min(left);
        left -= next;
        meta.inventory.stone_white -= next;
        if left > 0 {
          next = meta.inventory.stone_blue.min(left);
          left -= next;
          meta.inventory.stone_blue -= next;
        }
        if left > 0 {
          next = meta.inventory.stone_green.min(left);
          left -= next;
          meta.inventory.stone_green -= next;
        }
        if left > 0 {
          next = meta.inventory.stone_yellow.min(left);
          left -= next;
          meta.inventory.stone_yellow -= next;
        }
        assert_eq!(left, 0, "we asserted that there were enough stones, so we should have consumed that many stones now");

        // Add a windrone to the first empty slot
        let len = slots.len();
        for i in 0..len {
          if matches!(slots[i].kind, SlotKind::Emptiness) {
            slots[i] = create_slot_windrone(i, 1);
            windrone.state = WindroneState::WaitingForWind;
            break;
          }
          assert!(i < len - 1, "should have asserted beforehand that the windrone would fit somewhere");
        }
      }
    }
    WindroneState::WaitingForWind => {}
    WindroneState::WaitingForGoal => {}
    WindroneState::ReadyForTakeOff => {
      // The windrone has enough wind and at least one target to go to. Deduct the wind and go.
      set_windrone_state(windrone, WindroneState::FlyingToGoal);
      meta.inventory.wind -= 10;
    }
    WindroneState::FlyingToGoal => {}
    WindroneState::FlyingHome => {}
    WindroneState::ReturnedHome => {
      // The windrone just returned home. The windrone resets, waiting for its next task.
      // It gives you an energy boost for completing a task (50% of your missing energy).
      set_windrone_state(windrone, WindroneState::WaitingForWind);
      movable.now_energy += (movable.init_energy - movable.now_energy) * 0.5;
    }
  }

  match hydrone.state {
    HydroneState::Unconstructed => {
      if can_craft_hydrone(meta, slots) {
        // Deduct materials
        meta.inventory.wood -= 5;
        meta.inventory.water -= 10;
        let mut left = 5;
        let mut next = meta.inventory.stone_white.min(left);
        left -= next;
        // Do not allow the white stones. Only use more expensive stones to build a hydrone.
        // meta.inventory.stone_white -= next;
        // if left > 0 {
          next = meta.inventory.stone_blue.min(left);
          left -= next;
          meta.inventory.stone_blue -= next;
        // }
        if left > 0 {
          next = meta.inventory.stone_green.min(left);
          left -= next;
          meta.inventory.stone_green -= next;
        }
        if left > 0 {
          next = meta.inventory.stone_yellow.min(left);
          left -= next;
          meta.inventory.stone_yellow -= next;
        }
        assert_eq!(left, 0, "we asserted that there were enough stones, so we should have consumed that many stones now");

        // Add a hydrone to the first empty slot
        let len = slots.len();
        for i in 0..len {
          if matches!(slots[i].kind, SlotKind::Emptiness) {
            slots[i] = create_slot_hydrone(i, 1);
            set_hydrone_state(hydrone, HydroneState::WaitingForWater);
            break;
          }
          assert!(i < len - 1, "should have asserted beforehand that the hydrone would fit somewhere");
        }
      }
    }
    HydroneState::WaitingForWater => {}
    HydroneState::MovingToOrigin => {}
    HydroneState::MovingToNeighborCell => {}
    HydroneState::BuildingArrowCell => {}
    HydroneState::PickingUpMiner => {}
    HydroneState::DeliveringMiner => {}
  }
}
