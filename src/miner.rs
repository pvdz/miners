use super::slottable::*;
use super::{bridge};
use super::app_state::*;
use super::expando::*;
use super::cell::*;
use super::drone_san::*;
use super::options::*;
use super::biome::*;
use super::slot_windrone::*;
use super::slot_sandrone::*;
use super::slot_emptiness::*;
use super::world::*;
use super::slot_drone_launcher::*;
use super::slot_magnet::*;
use super::slot_energy_cell::*;
use super::values::*;
use super::helix::*;
use super::movable::*;
use super::slot_hammer::*;
use super::slot_drill::*;
use super::slot_purity_scanner::*;
use super::slot_broken_gps::*;
use super::slot_jacks_compass::*;
use super::inventory::*;
use super::drone_me::*;
use super::drone_win::*;
use super::tile::*;
use super::pickup::*;

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

  pub drones: Vec<MeDrone>,

  pub windrone: Windrone,
  pub sandrone: Sandrone,
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


fn create_slot(kind: SlotKind, i: usize, nth: i32, helix: &Helix, state: &mut AppState) -> Slottable {
  match kind {
    SlotKind::BrokenGps => {
      return create_slot_broken_gps(i, nth, 100.0 * 2.0_f32.powf((nth + 1) as f32));
    },
    SlotKind::Drill => {
      return create_drill(i, nth);
    },
    SlotKind::DroneLauncher => {
      return create_drone_launcher(i, nth, i as i32, helix.drone_gen_cooldown * 2.0_f32.powf(((nth as f32 / 2.0) + 1.0) as f32));
    },
    SlotKind::Emptiness => {
      return create_empty_slot(i);
    },
    SlotKind::EnergyCell => {
      return create_slot_energy_cell(i, nth, 100, 100.0 * 2.0_f32.powf((nth + 1) as f32));
    },
    SlotKind::Hammer => {
      return create_hammer(i, nth);
    },
    SlotKind::JacksCompass => {
      return create_slot_jacks_compass(i, nth, 40.0 * 2.0_f32.powf((nth + 1) as f32));
    }
    SlotKind::Magnet => {
      return create_slot_magnet(i, nth);
    }
    SlotKind::PurityScanner => {
      return create_slot_purity_scanner(i, nth, 100.0 * 2.0_f32.powf((nth + 1) as f32));
    },
    SlotKind::Sandrone => {
      panic!("The sandrone is not a valid starting slot");
    }
    SlotKind::RandomStart => {
      let slot = get_random_slot(&mut state.instance_rng_unseeded);
      return create_slot(slot, i, nth, helix, state);
    }
    SlotKind::Windrone => {
      panic!("The windrone is not a valid starting slot");
    }
  }
}

pub fn create_miner_from_helix(state: &mut AppState, helix: &Helix) -> Miner {
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

  // Prematurely create 32 drones. Otherwise we'd have to either,
  // - juggle the drones array into create_slot
  // - put the drone onto the slot (which is impossible or equally expensive when slottable is generic)
  // - make the drone optional, which is awkward
  // - initialize them afterwards, conditionally
  // The 32 objects should not be a big deal regardless so this is just easier
  let mut drones: Vec<MeDrone> = vec!();
  for _ in 0..32 {
    drones.push(MeDrone {
      movable: Movable {
        what: WHAT_DRONE,
        x: 0,
        y: 0,
        dir: Direction::Up,
        now_energy: 0.0,
        init_energy: 0.0,
        disabled: false,
      },
    });
  }

  // Initialize the slots from their helix config
  for i in 0..32 {
    let kind: SlotKind = helix.slots[i];
    let kind_usize = kind as usize;
    let nth: i32 = kind_counts[kind_usize];
    slots[i] = create_slot(kind, i, nth, helix, state);
    kind_counts[kind_usize] = kind_counts[kind_usize] + 1;
  }

  return Miner {
    helix: *helix,
    movable: Movable {
      what: WHAT_MINER,
      x: 0,
      y: 0,
      dir: Direction::Up,
      now_energy: max_energy,
      init_energy: max_energy,
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
    sandrone: create_sandrone(),
  };
}

fn can_craft_windrone(miner: &Miner) -> bool {
  let meta = &miner.meta;
  let slots = &miner.slots;
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
fn can_craft_sandrone(meta: &MinerMeta, slots: &Vec<Slottable>) -> bool {
  let mut has_empty = false;
  for i in 0..slots.len() {
    match slots[i].kind {
      SlotKind::Sandrone => return false,
      SlotKind::Emptiness => has_empty = true,
      _ => {},
    }
  }

  // Must have an available slot for the sandrone
  if !has_empty { return false; }

  // Must have enough materials to craft a sandrone
  return meta.inventory.wood > 5 && meta.inventory.water > 10 && (/*meta.inventory.stone_white + */meta.inventory.stone_blue + meta.inventory.stone_green + meta.inventory.stone_yellow) > 5;
}

pub fn tick_miner(options: &mut Options, biome: &mut Biome) {
  // If;
  // - There are slots available
  // - The build drone was built
  // - The build drone is currently not used
  // - There are enough resources
  // then send a drone to build something on the expando

  // The build drone needs to be crafted using wood and stone.

  // For now the drone can phase through walls but later we'll want to add some pathfinding.

  match biome.miner.windrone.state {
    WindroneState::Unconstructed => {
      if can_craft_windrone(&biome.miner) {
        // Deduct materials
        biome.miner.meta.inventory.wood -= 5;
        let mut left = 5;
        let mut next = biome.miner.meta.inventory.stone_white.min(left);
        left -= next;
        biome.miner.meta.inventory.stone_white -= next;
        if left > 0 {
          next = biome.miner.meta.inventory.stone_blue.min(left);
          left -= next;
          biome.miner.meta.inventory.stone_blue -= next;
        }
        if left > 0 {
          next = biome.miner.meta.inventory.stone_green.min(left);
          left -= next;
          biome.miner.meta.inventory.stone_green -= next;
        }
        if left > 0 {
          next = biome.miner.meta.inventory.stone_yellow.min(left);
          left -= next;
          biome.miner.meta.inventory.stone_yellow -= next;
        }
        assert_eq!(left, 0, "we asserted that there were enough stones, so we should have consumed that many stones now");

        // Add a windrone to the first empty slot
        let len = biome.miner.slots.len();
        for i in 0..len {
          if matches!(biome.miner.slots[i].kind, SlotKind::Emptiness) {
            biome.miner.slots[i] = create_slot_windrone(i, 1);
            biome.miner.windrone.state = WindroneState::WaitingForWind;
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
      set_windrone_state(biome, WindroneState::FlyingToGoal);
      biome.miner.meta.inventory.wind -= 10;
    }
    WindroneState::FlyingToGoal => {}
    WindroneState::FlyingHome => {}
    WindroneState::ReturnedHome => {
      // The windrone just returned home. The windrone resets, waiting for its next task.
      // It gives you an energy boost for completing a task (50% of your missing energy).
      set_windrone_state(biome, WindroneState::WaitingForWind);
      biome.miner.movable.now_energy += (biome.miner.movable.init_energy - biome.miner.movable.now_energy) * 0.5;
    }
  }

  match biome.miner.sandrone.state {
    SandroneState::Unconstructed => {
      if can_craft_sandrone(&mut biome.miner.meta, &mut biome.miner.slots) {
        // Deduct materials
        biome.miner.meta.inventory.wood -= 5;
        biome.miner.meta.inventory.water -= 10;
        let mut left = 5;
        let mut next = biome.miner.meta.inventory.stone_white.min(left);
        left -= next;
        // Do not allow the white stones. Only use more expensive stones to build a sandrone.
        // biome.miner.meta.inventory.stone_white -= next;
        // if left > 0 {
          next = biome.miner.meta.inventory.stone_blue.min(left);
          left -= next;
          biome.miner.meta.inventory.stone_blue -= next;
        // }
        if left > 0 {
          next = biome.miner.meta.inventory.stone_green.min(left);
          left -= next;
          biome.miner.meta.inventory.stone_green -= next;
        }
        if left > 0 {
          next = biome.miner.meta.inventory.stone_yellow.min(left);
          left -= next;
          biome.miner.meta.inventory.stone_yellow -= next;
        }
        assert_eq!(left, 0, "we asserted that there were enough stones, so we should have consumed that many stones now");

        // Add a sandrone to the first empty slot
        let len = biome.miner.slots.len();
        for i in 0..len {
          if matches!(biome.miner.slots[i].kind, SlotKind::Emptiness) {
            biome.miner.slots[i] = create_slot_sandrone(i, 1);
            set_sandrone_state(&mut biome.miner.sandrone, SandroneState::WaitingForWater);
            break;
          }
          assert!(i < len - 1, "should have asserted beforehand that the sandrone would fit somewhere");
        }
      }
    }
    SandroneState::WaitingForWater => {}
    SandroneState::MovingToOrigin => {}
    SandroneState::MovingToNeighborCell => {}
    SandroneState::BuildingArrowCell => {}
    SandroneState::PickingUpMiner => {}
    SandroneState::DeliveringMiner => {}
    SandroneState::Redecorating => {}
  }

  if !biome.miner.sandrone.air_lifting {
    move_miner(options, biome);
  }
}

pub fn move_miner(options: &mut Options, biome: &mut Biome) {
  biome.miner.meta.points_last_move = 0;

  let cx = biome.miner.movable.x;
  let cy = biome.miner.movable.y;
  let (deltax, deltay) = delta_forward(biome.miner.movable.dir);
  let nextx = cx + deltax;
  let nexty = cy + deltay;

  // If this move would go OOB, expand the world to make sure that does not happen
  ensure_cell_in_world(&mut biome.world, options, nextx, nexty);

  let unextx = (biome.world.min_x.abs() + nextx) as usize;
  let unexty = (biome.world.min_y.abs() + nexty) as usize;

  // Do not remove an expando when moving over it.
  match biome.world.tiles[unexty][unextx] {
    Cell {tile: Tile::Empty, pickup: Pickup::Expando, pickup_value, ..} => {
      // This must have been an expando that was just revealed. TODO: prevent this case..? :)
      // Set the cell to water tile and add the expando to the world so it can flow.
      biome.world.tiles[unexty][unextx].tile = Tile::ExpandoWater;
      biome.world.expandos.push(create_expando(nextx, nexty, pickup_value));
    },
    _ => {},
  }

  if biome.miner.sandrone.air_lifted { // TODO: rename to "filling castle" or "magic wall up" or whatever
    // If the miner would move OOB then apply special move logic
    if oob(nextx, nexty, biome.miner.sandrone.expansion_min_x, biome.miner.sandrone.expansion_min_y, biome.miner.sandrone.expansion_max_x, biome.miner.sandrone.expansion_max_y) {
      // The miner is about to step OOB. Force it to turn.

      // The current cell can be filled in some cases, except when it's already the last seen exit tile.
      if biome.miner.movable.x == biome.miner.sandrone.last_empty_castle_exit_x && biome.miner.movable.y == biome.miner.sandrone.last_empty_castle_exit_y {
        // Do nothing. Must keep at least one exit tile.
      } else if can_magic_wall_bordering_empty_cell_be_push_cell(options, &mut biome.world, biome.miner.movable.x, biome.miner.movable.y, biome.miner.sandrone.expansion_min_x, biome.miner.sandrone.expansion_min_y, biome.miner.sandrone.expansion_max_x, biome.miner.sandrone.expansion_max_y) {
        set_cell_tile_at(options, &mut biome.world, biome.miner.movable.x, biome.miner.movable.y, Tile::Impassible);
      } else {
        // This is the new "last seen unfilled exit tile"
        biome.miner.sandrone.last_empty_castle_exit_x = biome.miner.movable.x;
        biome.miner.sandrone.last_empty_castle_exit_y = biome.miner.movable.y;
      }

      // So forward is blocked because it's OOB of the magic castle wall. Check which of
      // the other three directions are available. Prefer left or right and otherwise turn
      // around. Only when the miner is completely stuck do we destroy the magic wall.

      let avail_left = !is_push_impossible_cell(options, &mut biome.world, biome.miner.movable.x + deltay, biome.miner.movable.y - deltax) && !oob(biome.miner.movable.x + deltay, biome.miner.movable.y - deltax, biome.miner.sandrone.expansion_min_x, biome.miner.sandrone.expansion_min_y, biome.miner.sandrone.expansion_max_x, biome.miner.sandrone.expansion_max_y);
      let avail_right = !is_push_impossible_cell(options, &mut biome.world, biome.miner.movable.x - deltay, biome.miner.movable.y + deltax) && !oob(biome.miner.movable.x - deltay, biome.miner.movable.y + deltax, biome.miner.sandrone.expansion_min_x, biome.miner.sandrone.expansion_min_y, biome.miner.sandrone.expansion_max_x, biome.miner.sandrone.expansion_max_y);

      if avail_left && avail_right {
        // flip-flop (each time you visit this tile take left then right then left in repeat)

        let v = get_cell_tile_value_at(options, &mut biome.world, biome.miner.movable.x, biome.miner.movable.y, );
        set_cell_tile_value_at(options, &mut biome.world, biome.miner.movable.x, biome.miner.movable.y, if v == 1 { 0 } else { 1 });

        biome.miner.movable.dir = turn_lr(biome.miner.movable.dir, v == 1);
      } else if avail_left {
        biome.miner.movable.dir = turn_lr(biome.miner.movable.dir, true);
      } else if avail_right {
        biome.miner.movable.dir = turn_lr(biome.miner.movable.dir, false);
      } else {
        // Can't go left or right. Check back.
        // In practice the back should not be oob but in theory that's possible :shrug:
        let avail_back = !is_push_impossible_cell(options, &mut biome.world, biome.miner.movable.x - deltax, biome.miner.movable.y - deltay) && !oob(biome.miner.movable.x - deltax, biome.miner.movable.y - deltay, biome.miner.sandrone.expansion_min_x, biome.miner.sandrone.expansion_min_y, biome.miner.sandrone.expansion_max_x, biome.miner.sandrone.expansion_max_y);
        if avail_back {
          // Turn around
          biome.miner.movable.dir = turn_back(biome.miner.movable.dir);
        } else {
          // Last cell? Assume we are finished. Fill it and destroy the magic wall.
          if !options.visual {
            bridge::log("Setting visual because miner finished filling up castle");
            options.visual = true;
            options.visible_index = biome.index;
          }
          set_cell_tile_at(options, &mut biome.world, biome.miner.movable.x, biome.miner.movable.y, Tile::Impassible);
          biome.miner.sandrone.post_castle = biome.ticks;
          biome.miner.sandrone.air_lifted = false;

          // Change tiles for the entire castle grid. Reset values and everything.
          // Special case the stuck drones?
          for y in biome.miner.sandrone.expansion_min_y..biome.miner.sandrone.expansion_max_y+1 {
            for x in biome.miner.sandrone.expansion_min_x..biome.miner.sandrone.expansion_max_x+1 {
              if matches!(get_cell_tile_at(options, &mut biome.world, x, y), Tile::Impassible) {
                // Change the tile so we can enable special drawing mode for it
                // TODO: if a drone was stuck in it perhaps it enables a special super soil?
                set_cell_tile_at(options, &mut biome.world, x, y, Tile::Soil);
              }
              set_cell_tile_value_at(options, &mut biome.world, x, y, 0);
              set_cell_pickup_at(options, &mut biome.world, x, y, Pickup::Nothing);
              set_cell_pickup_value_at(options, &mut biome.world, x, y, 0);
            }
          }
        }
      }

      // Do not move the miner, just turn it. This should prevent it from going OOB.
      return;
    }
  }

  let mut fill_current_cell = false;
  let mut fill_current_x = 0;
  let mut fill_current_y = 0;

  let mut was_boring = false; // Did we just move forward? No blocks, no pickups?

  // let drills = biome.miner.meta.kind_counts[SlotKind::Drill as usize];
  // let hammers = biome.miner.meta.kind_counts[SlotKind::Hammer as usize];
  let tile = biome.world.tiles[unexty][unextx].tile;
  match tile {
    Tile::Wall4 => bump_wall_miner(options, biome, 4, nextx, nexty, deltax, deltay, unextx, unexty),
    Tile::Wall3 => bump_wall_miner(options, biome, 3, nextx, nexty, deltax, deltay, unextx, unexty),
    Tile::Wall2 => bump_wall_miner(options, biome, 2, nextx, nexty, deltax, deltay, unextx, unexty),
    Tile::Wall1 => bump_wall_miner(options, biome, 1, nextx, nexty, deltax, deltay, unextx, unexty),

    | Tile::Push
    | Tile::Impassible
    => {
      // Moving to a push tile or an impassible (dead end) tile. Must turn and try to make sure
      // not to send the movable into an infinite loop.
      let ( tx, ty, fill ): ( i32, i32, bool ) = push_corner_move(options, &mut biome.world, biome.miner.movable.x, biome.miner.movable.y, deltax, deltay, false) ;

      if biome.miner.sandrone.air_lifted && fill {
        fill_current_cell = true;
        fill_current_x = biome.miner.movable.x;
        fill_current_y = biome.miner.movable.y;
      }

      // We have the new delta xy for the turn. Act accordingly. If they're 0 flip-flop. The normal rule has a reasonable chance to loop so flip-flopping is more efficient.
      biome.miner.movable.dir = match (tx, ty) {
        (-1, 0) => Direction::Left,
        (1, 0) => Direction::Right,
        (0, 1) => Direction::Down,
        (0, -1) => Direction::Up,
        (0, 0) => {
          // Must check whether left or right is oob. If so, force the other way.
          // Check for oobs. Prevents annoying flip-flop patterns for one-way-streets
          if biome.miner.sandrone.air_lifted && oob(biome.miner.movable.x + deltay, biome.miner.movable.y - deltax, biome.miner.sandrone.expansion_min_x, biome.miner.sandrone.expansion_min_y, biome.miner.sandrone.expansion_max_x, biome.miner.sandrone.expansion_max_y) {
            // Do not turn this way. Turn the other way.
            // Turn clockwise
            match biome.miner.movable.dir {
              Direction::Up => Direction::Right,
              Direction::Right => Direction::Down,
              Direction::Down => Direction::Left,
              Direction::Left => Direction::Up,
            }
          } else if biome.miner.sandrone.air_lifted && oob(biome.miner.movable.x - deltay, biome.miner.movable.y + deltax, biome.miner.sandrone.expansion_min_x, biome.miner.sandrone.expansion_min_y, biome.miner.sandrone.expansion_max_x, biome.miner.sandrone.expansion_max_y) {
            // Do not turn this way, turn the other way
            // Turn counter-clockwise
            match biome.miner.movable.dir {
              Direction::Up => Direction::Left,
              Direction::Right => Direction::Up,
              Direction::Down => Direction::Right,
              Direction::Left => Direction::Down,
            }
          } else {
            let v = get_cell_tile_value_at(options, &biome.world, biome.miner.movable.x, biome.miner.movable.y, );
            set_cell_tile_value_at(options, &mut biome.world, biome.miner.movable.x, biome.miner.movable.y, if v == 1 { 0 } else { 1 });

            match biome.miner.movable.dir {
              Direction::Up => if v == 1 { Direction::Left } else { Direction::Right },
              Direction::Right => if v == 1 { Direction::Up } else { Direction::Down },
              Direction::Down => if v == 1 { Direction::Right } else { Direction::Left },
              Direction::Left => if v == 1 { Direction::Down } else { Direction::Up },
            }
          }
        },
        _ => panic!("This delta should not be possible {},{}", tx, ty),
      };

      //biome.miner.movable.now_energy = biome.miner.movable.now_energy - biome.miner.meta.block_bump_cost;
    }

    // The rest is considered an empty or at least passable tile
    | Tile::Fountain
    | Tile::Soil
    | Tile::ZeroZero
    | Tile::TenLine
    | Tile::HideWorld
    | Tile::Test2
    | Tile::Test3
    | Tile::Empty
    | Tile::ExpandoWater
    => {
      if biome.miner.sandrone.air_lifted {
        let blocked_back = matches!(get_cell_tile_at(options, &biome.world, biome.miner.movable.x + -deltax, biome.miner.movable.y + -deltay), Tile::Push | Tile::Impassible);
        if blocked_back {
          let ( _tx, _ty, fill ): ( i32, i32, bool ) = push_corner_move(options, &mut biome.world, biome.miner.movable.x, biome.miner.movable.y, deltax, deltay, true) ;
          if fill {
            fill_current_cell = true;
            fill_current_x = biome.miner.movable.x;
            fill_current_y = biome.miner.movable.y;
          }
        }
      }

      was_boring = true;

      // Always pick up the forward tile (the tile we moved into)
      if move_miner_pickup_from_empty_tile(options, biome, nextx, nexty) {
        was_boring = false;
      }

      // Do we have any magnets primed? Bump the value by that many.
      // Note: purity scanner only works for the miner itself
      for slot_index in 0..biome.miner.slots.len() {
        match biome.miner.slots[slot_index].kind {
          SlotKind::Magnet => {
            // Magnets allow to pick up neighboring tiles from the tile you started at
            // In order of magnet count; back, left, right, forward-left, forward-right, back-left, back-right
            // There's no advantage to having more than 7 magnets.
            // TBD whether there's any other drawback than taking up a slot
            match biome.miner.slots[slot_index].nth {
              0 => {
                let (nx, ny) = coord_back(cx, cy, biome.miner.movable.dir);
                if move_miner_pickup_from_empty_tile(options, biome, nx, ny) {
                  was_boring = false;
                } else {
                  biome.miner.slots[slot_index].sum += 1.0;
                }
              },
              1 => {
                let (nx, ny) = coord_left(cx, cy, biome.miner.movable.dir);
                if move_miner_pickup_from_empty_tile(options, biome, nx, ny) {
                  was_boring = false;
                } else {
                  biome.miner.slots[slot_index].sum += 1.0;
                }
              },
              2 => {
                let (nx, ny) = coord_right(cx, cy, biome.miner.movable.dir);
                if move_miner_pickup_from_empty_tile(options, biome, nx, ny) {
                  was_boring = false;
                } else {
                  biome.miner.slots[slot_index].sum += 1.0;
                }
              },
              3 => {
                let (nx, ny) = coord_fl(cx, cy, biome.miner.movable.dir);
                if move_miner_pickup_from_empty_tile(options, biome, nx, ny) {
                  was_boring = false;
                } else {
                  biome.miner.slots[slot_index].sum += 1.0;
                }
              },
              4 => {
                let (nx, ny) = coord_fr(cx, cy, biome.miner.movable.dir);
                if move_miner_pickup_from_empty_tile(options, biome, nx, ny) {
                  was_boring = false;
                } else {
                  biome.miner.slots[slot_index].sum += 1.0;
                }
              },
              5 => {
                let (nx, ny) = coord_bl(cx, cy, biome.miner.movable.dir);
                if move_miner_pickup_from_empty_tile(options, biome, nx, ny) {
                  was_boring = false;
                } else {
                  biome.miner.slots[slot_index].sum += 1.0;
                }
              },
              6 => {
                let (nx, ny) = coord_br(cx, cy, biome.miner.movable.dir);
                if move_miner_pickup_from_empty_tile(options, biome, nx, ny) {
                  was_boring = false;
                } else {
                  biome.miner.slots[slot_index].sum += 1.0;
                }
              },
              _ => {
                // Unused magnet
              }
            }
          },
          _ => {

          }
        }
      }

      biome.world.tiles[unexty][unextx].visited += 1;
      biome.miner.movable.x = nextx;
      biome.miner.movable.y = nexty;
    },
  }

  if biome.miner.movable.now_energy < 0.0 {
    biome.miner.movable.now_energy = 0.0;
  }

  // Allow to fill if it's not the last seen exit tile
  if fill_current_cell && (biome.miner.movable.x != biome.miner.sandrone.last_empty_castle_exit_x || biome.miner.movable.y != biome.miner.sandrone.last_empty_castle_exit_y) {
    set_cell_tile_at(options, &mut biome.world, fill_current_x, fill_current_y, Tile::Impassible);
    biome.miner.sandrone.impassable_tiles.push((fill_current_x, fill_current_y));
  }

  // Cannot be in an infinite loop while building the sand castle
  if was_boring && !biome.miner.sandrone.air_lifted /*&& !post_castle*/ {
    // Prevent endless loops by making it increasingly more difficult to make consecutive moves that where nothing happens
    biome.miner.movable.now_energy = (biome.miner.movable.now_energy - biome.miner.meta.boredom_level as f32).max(0.0);
    // The cost grows the longer nothing keeps happening ("You're getting antsy, thirsty for an event")
    biome.miner.meta.boredom_level = biome.miner.meta.boredom_level + 1;
  } else {
    biome.miner.meta.boredom_level = 0;
  }
}

pub fn bump_wall_miner(options: &mut Options, biome: &mut Biome, strength: i32, nextx: i32, nexty: i32, deltax: i32, deltay: i32, unextx: usize, unexty: usize) {
  let cell = &mut biome.world.tiles[unexty][unextx];

  let hammers = biome.miner.meta.kind_counts[SlotKind::Hammer as usize];
  let drills = biome.miner.meta.kind_counts[SlotKind::Drill as usize];

  let n = strength - (1 + hammers);

  biome.world.tiles[unexty][unextx] = match n.max(0) {
    3 => create_unvisited_cell(Tile::Wall3, cell.pickup, cell.tile_value, cell.pickup_value),
    2 => create_unvisited_cell(Tile::Wall2, cell.pickup, cell.tile_value, cell.pickup_value),
    1 => create_unvisited_cell(Tile::Wall1, cell.pickup, cell.tile_value, cell.pickup_value),
    0 => create_unvisited_cell(Tile::Empty, cell.pickup, cell.tile_value, cell.pickup_value),
    // always at least -1
    _ => panic!("A bump should always at least decrease the wall by one so it can never stay 4: {}", n),
  };
  if n <= 0 {
    // Broke a wall. Add sand.
    // TODO: what about the drill? What about bonuses? Should it be u32 or f32?
    biome.miner.meta.inventory.sand += 1;
  }

  if drills > 0 {
    drill_deeper(drills, hammers, nextx, nexty, deltax, deltay, &mut biome.world, options);
  }
  biome.miner.meta.prev_move_bumped = true;

  biome.miner.movable.now_energy = (biome.miner.movable.now_energy - biome.miner.meta.block_bump_cost).max(0.0);
  // TODO: should drones use same "prefer visited tiles" heuristic as miner?
  biome.miner.movable.dir = get_most_visited_dir_from_xydir(options, &mut biome.world, nextx, nexty, biome.miner.movable.dir);
}

pub fn move_miner_pickup_from_empty_tile(options: &mut Options, biome: &mut Biome, x: i32, y: i32) -> bool {
  // Return true if anything was picked up. False if nothing. Used for the boring stat.
  ensure_cell_in_world(&mut biome.world, options, x, y);

  let unextx = (biome.world.min_x.abs() + x) as usize;
  let unexty = (biome.world.min_y.abs() + y) as usize;

  let tile = &mut biome.world.tiles[unexty][unextx];
  let meta = &mut biome.miner.meta;

  match tile.pickup {
    Pickup::Diamond => {
      let mut primed = 0;
      // Do we have any purity scanners primed? Bump the value by that many.
      // Note: purity scanner only works for the miner itself
      for n in &mut biome.miner.slots {
        if matches!(n.kind, SlotKind::PurityScanner) && n.cur_cooldown >= n.max_cooldown {
          primed += 1;
        }
      }

      // Different gems with different points.
      // Miners could have properties or powerups to affect this, too.
      let gv: i32 = (tile.pickup_value + primed).min(3) as i32;
      match gv {
        0 => meta.inventory.diamond_white += 1,
        1 => meta.inventory.diamond_green += 1,
        2 => meta.inventory.diamond_blue += 1,
        3 => meta.inventory.diamond_yellow += 1,
        _ => panic!("what value did this diamond have: {:?}", tile),
      };
      let gem_value: i32 = gv + 1;

      meta.points_last_move = gem_value;
      tile.pickup = Pickup::Nothing;
      tile.pickup_value = 0;
    },
    Pickup::Energy => {
      biome.miner.movable.now_energy = (biome.miner.movable.now_energy + (E_VALUE as f64 * ((100.0 + meta.multiplier_energy_pickup as f64) / 100.0)) as f32).min(meta.max_energy);
      meta.inventory.energy += 1;
      tile.pickup = Pickup::Nothing;
      tile.pickup_value = 0;
    },
    Pickup::Stone => {
      // Do we have any purity scanners primed? Bump the value by that many.
      // Note: purity scanner only works for the miner itself
      let mut primed = 0;
      for n in &mut biome.miner.slots {
        match n.kind {
          SlotKind::PurityScanner => if n.cur_cooldown >= n.max_cooldown {
            primed += 1;
          },
          _ => ()
        }
      }

      match (tile.pickup_value + primed).min(3) {
        0 => meta.inventory.stone_white += 1,
        1 => meta.inventory.stone_green += 1,
        2 => meta.inventory.stone_blue += 1,
        3 => meta.inventory.stone_yellow += 1,
        _ => panic!("what value did this stone have: {:?}", tile),
      }
      meta.points_last_move = tile.pickup_value as i32;
      tile.pickup = Pickup::Nothing;
      tile.pickup_value = 0;
    },
    Pickup::Wind => {
      meta.inventory.wind += 1;
      tile.pickup = Pickup::Nothing;
      tile.pickup_value = 0;
    },
    Pickup::Water => {
      meta.inventory.water += 1;
      tile.pickup = Pickup::Nothing;
      tile.pickup_value = 0;
    },
    Pickup::Wood => {
      meta.inventory.wood += 1;
      tile.pickup = Pickup::Nothing;
      tile.pickup_value = 0;
    },
    | Pickup::Nothing
    | Pickup::Expando // Ignore, fake pickup
    | Pickup::Fountain // Ignore, fake pickup... TODO: probably some special behavior?
    => {
      // Ignore this "pickup"
      return true; // "boring", nothing happened
    },
  }

  // Was not boring
  return false; // "not boring"
}
