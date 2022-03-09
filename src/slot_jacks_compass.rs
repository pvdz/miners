use crate::pickup::*;
use crate::biome::*;
use super::utils::*;
use super::movable::*;
use super::slottable::*;
use super::world::*;
use super::options::*;

pub const TITLE_JACKS_COMPASS: &str = "Jack's Compass";

/**
 * Always points towards that which you want most.
 * When it activates it checks your current major milestone and turns you into the direction towards
 * that which is closest and most likely to finish this objective. TODO: or maybe it's a teleporter?
 * Reference to Pirates of the Caribbean.
 */
pub fn create_slot_jacks_compass(slot_index: usize, nth: i32, max_cooldown: f32) -> Slottable {
  assert!(max_cooldown > 0.0, "slot max cooldown should be non-zero: {}", max_cooldown);
  return Slottable {
    kind: SlotKind::JacksCompass,
    slot: slot_index,
    title: TITLE_JACKS_COMPASS.to_owned(),
    max_cooldown,
    cur_cooldown: 0.0,
    nth,
    val: 1.0, // last degree; 1 or -1
    sum: 0.0,
  };
}

pub fn tick_slot_jacks_compass(options: &mut Options, biome: &mut Biome, slot_index: usize) {
  let slot: &mut Slottable = &mut biome.miner.slots[slot_index];

  if slot.cur_cooldown < slot.max_cooldown {
    slot.cur_cooldown += 1.0;
  } else {
    slot.cur_cooldown = slot.max_cooldown;
  }
  if slot.cur_cooldown >= slot.max_cooldown {
    // Search in an increasing radius up to 4x4 for the most valuable resource and face that way
    // If nothing is found, nothing happens. Targets the nearest, most valuable resource.

    let mx = biome.miner.movable.x;
    let my = biome.miner.movable.y;

    let mut highest = 0;
    let mut tox = 0;
    let mut toy = 0;
    for y in my - 2..my + 2 {
      for x in mx - 2..mx + 2 {
        let pickup = get_cell_stuff_at(options, &biome.world, x, y).1;
        let prio = pickup_to_priority(pickup);
        if prio > highest {
          highest = prio;
          tox = x;
          toy = y;
        }
      }
    }

    slot.cur_cooldown = 0.0;

    if highest > 0 {
      slot.sum += 1.0;
      slot.val = highest as f32;

      biome.miner.movable.dir =
        if (mx - tox).abs() < (my - toy).abs() {
          if tox < mx {
            Direction::Left
          } else {
             Direction::Right
          }
        } else {
          if toy < my {
            Direction::Up
          } else {
            Direction::Down
          }
        };
    }
  }
}

pub fn ui_slot_jacks_compass(slot: &Slottable) -> (String, String, String) {
  return (
    TITLE_JACKS_COMPASS.to_string(),
    progress_bar(20, slot.cur_cooldown, slot.max_cooldown, false),
    format!("Activated: {} times. Last prio: {}", slot.sum, slot.val as u32)
  );
}
