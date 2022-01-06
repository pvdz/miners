use super::movable::*;
use super::slottable::*;

pub const TITLE_BROKEN_GPS: &str = "Broken GPS";

/**
 * Turns you left or right after a certain cooldown. Cooldown increases the more you get.
 * This may allow you go to in places that you otherwise would not go into.
 */
pub fn create_slot_broken_gps(nth: i32, max_cooldown: f32) -> Slottable {
    assert!(max_cooldown > 0.0, "slot max cooldown should be non-zero: {}", max_cooldown);
    return Slottable {
        kind: SlotKind::BrokenGps,
        title: TITLE_BROKEN_GPS.to_owned(),
        max_cooldown,
        cur_cooldown: 0.0,
        nth,
        val: 1, // last degree; 1 or -1
        sum: 0,
    };
}

pub fn tick_slot_broken_gps(slot: &mut Slottable, miner_movable: &mut Movable) {
    slot.cur_cooldown = slot.cur_cooldown + 1.0;
    if slot.cur_cooldown >= slot.max_cooldown {
        miner_movable.dir = match miner_movable.dir {
            Direction::Up => if slot.val < 0 { Direction::Right } else { Direction::Left },
            Direction::Right => if slot.val < 0 { Direction::Down } else { Direction::Up },
            Direction::Down => if slot.val < 0 { Direction::Left } else { Direction::Right },
            Direction::Left => if slot.val < 0 { Direction::Up } else { Direction::Down },
            _ => panic!("what enum"),
        };
        slot.cur_cooldown = 0.0;
        slot.val = slot.val * -1;
        slot.sum = slot.sum + 1;
    }
}
