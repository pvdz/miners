use super::utils::*;
use super::movable::*;
use super::slottable::*;
use super::biome::*;
use super::options::*;

pub const TITLE_BROKEN_GPS: &str = "Broken GPS";

/**
 * Turns you left or right after a certain cooldown. Cooldown increases the more you get.
 * This may allow you go to in places that you otherwise would not go into.
 */
pub fn create_slot_broken_gps(slot_index: usize, nth: i32, max_cooldown: f32) -> Slottable {
    assert!(max_cooldown > 0.0, "slot max cooldown should be non-zero: {}", max_cooldown);
    return Slottable {
        kind: SlotKind::BrokenGps,
        slot: slot_index,
        title: TITLE_BROKEN_GPS.to_owned(),
        max_cooldown,
        cur_cooldown: 0.0,
        nth,
        val: 1.0, // last degree; 1 or -1
        sum: 0.0,
    };
}

pub fn tick_slot_broken_gps(_options: &Options, biome: &mut Biome, slot_index: usize) {

    let slot = &mut biome.miner.slots[slot_index];

    slot.cur_cooldown += 1.0;
    if slot.cur_cooldown >= slot.max_cooldown {
        biome.miner.movable.dir = match biome.miner.movable.dir {
            Direction::Up => if slot.val < 0.0 { Direction::Right } else { Direction::Left },
            Direction::Right => if slot.val < 0.0 { Direction::Down } else { Direction::Up },
            Direction::Down => if slot.val < 0.0 { Direction::Left } else { Direction::Right },
            Direction::Left => if slot.val < 0.0 { Direction::Up } else { Direction::Down },
        };
        slot.cur_cooldown = 0.0;
        slot.val *= -1.0;
        slot.sum += 1.0;
    }
}

pub fn ui_slot_broken_gps(slot: &Slottable) -> (String, String, String) {
    return (
        TITLE_BROKEN_GPS.to_string(),
        progress_bar(20, slot.cur_cooldown, slot.max_cooldown, false),
        format!("Activated: {} times. Last dir: {}", slot.sum, if slot.val == 1.0 { "Clockwise" } else { "Counter Clockwise" })
    );
}
