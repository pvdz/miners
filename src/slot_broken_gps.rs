use std::fmt;

// use super::miner::*;
// use super::world::*;
use super::movable::*;
use super::slottable::*;
use super::values::*;
// use super::icons::*;
// use super::options::*;
// use super::drone::*;
// use super::cell_contents::*;

pub const TITLE_BROKEN_GPS: &str = "Broken GPS";

/**
 * Turns you left or right after a certain cooldown. Cooldown increases the more you get.
 * This may allow you go to in places that you otherwise would not go into.
 */
pub fn create_slot_broken_gps(nth: i32, max_cooldown: f32) -> Slottable {
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
            DIR_UP => if slot.val < 0 { DIR_RIGHT } else { DIR_LEFT },
            DIR_RIGHT => if slot.val < 0 { DIR_DOWN } else { DIR_UP },
            DIR_DOWN => if slot.val < 0 { DIR_LEFT } else { DIR_RIGHT },
            DIR_LEFT => if slot.val < 0 { DIR_UP } else { DIR_DOWN },
            _ => panic!("what enum"),
        };
        slot.cur_cooldown = 0.0;
        slot.val = slot.val * -1;
    }
}

// pub struct BrokenGps {
//     pub max_cooldown: f32,
//     pub cooldown: f32,
//     pub last_degrees: i32, // I guess 90 or -90 but it might as well be an enum
//     // Offset zero. The how manieth gps is this? Every gps is half as efficient as the previous.
//     pub nth: i32,
// }
/*
impl Slottable for BrokenGps {


    fn paint_entity(&self, world: &World, options: &Options) -> (Cell, i32, i32) { return ( Cell::Empty, 0, 0 ); }
    fn paint_ui(&self, world: &World, options: &Options) -> Vec<char> { vec!() }
    fn paint_log(&self, world: &World, options: &Options) -> Vec<char> { vec!() }

    fn title(&self) -> &str { return TITLE_BROKEN_GPS; }

    fn to_symbol(&self) -> &str { return "G"; }

    fn get_cooldown(&self) -> f32 {
        return self.cooldown;
    }

    fn set_cooldown(&mut self, v: f32) -> f32 {
        if v > self.get_max_cooldown() {
            self.cooldown = self.get_max_cooldown();
        } else if v < 0.0 {
            self.cooldown = 0.0;
        } else {
            self.cooldown = v;
        }
        return self.cooldown;
    }

    fn get_max_cooldown(&self) -> f32 {
        return self.max_cooldown;
    }

    fn set_max_cooldown(&mut self, v: f32) -> f32 {
        self.max_cooldown = v;
        return v;
    }
}

impl fmt::Display for BrokenGps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{} {: >3}% (last turn was {}) {: >100}",
            std::iter::repeat('|').take(((self.get_cooldown() / self.get_max_cooldown()) * 10.0) as usize).collect::<String>(),
            std::iter::repeat('-').take(10 - ((self.get_cooldown() as f64 / self.get_max_cooldown() as f64) * 10.0) as usize).collect::<String>(),
            ((self.get_cooldown() / self.get_max_cooldown()) * 100.0) as i32,
            if self.last_degrees < 0 { "left" } else { "right" },
            ' ',
        )
    }
}
*/
