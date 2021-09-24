use std::fmt;

use super::slottable::*;
use super::miner::*;
use super::world::*;
use super::movable::*;
use super::slottable::*;
use super::movable::*;
use super::miner::*;
use super::world::*;
use super::values::*;
use super::drone::*;

pub const TITLE_BROKEN_GPS: &str = "Broken GPS";

/**
 * Turns you left or right after a certain cooldown. Cooldown increases the more you get.
 * This may allow you go to in places that you otherwise would not go into.
 */
pub struct BrokenGps {
    pub max_cooldown: i32,
    pub cooldown: i32,
    pub last_degrees: i32, // I guess 90 or -90 but it might as well be an enum
    // Offset zero. The how manieth gps is this? Every gps is half as efficient as the previous.
    pub nth: i32,
}

impl Slottable for BrokenGps {
    fn before_paint(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, _world: &mut World) {
        self.cooldown = self.cooldown + 1;
        if self.cooldown >= self.max_cooldown {
            miner_movable.dir = match miner_movable.dir {
                DIR_UP => if self.last_degrees < 0 { DIR_RIGHT } else { DIR_LEFT },
                DIR_RIGHT => if self.last_degrees < 0 { DIR_DOWN } else { DIR_UP },
                DIR_DOWN => if self.last_degrees < 0 { DIR_LEFT } else { DIR_RIGHT },
                DIR_LEFT => if self.last_degrees < 0 { DIR_UP } else { DIR_DOWN },
                _ => panic!("what enum"),
            };
            self.cooldown = 0;
            self.last_degrees = self.last_degrees * -1;
        }
    }

    fn paint(&self, painting: &mut Grid, world: &World) {}

    fn after_paint(&mut self, _miner_movable: &mut Movable, _miner_meta: &mut MinerMeta, _world: &mut World) {}

    fn title(&self) -> &str { return TITLE_BROKEN_GPS; }

    fn to_symbol(&self) -> &str { return "G"; }
}

impl fmt::Display for BrokenGps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        write!(
            f,
            "{}{} {: >3}% (last turn was {}) {: >100}",
            std::iter::repeat('|').take(((self.cooldown as f32 / self.max_cooldown as f32) * 10.0) as usize).collect::<String>(),
            std::iter::repeat('-').take(10 - ((self.cooldown as f64 / self.max_cooldown as f64) * 10.0) as usize).collect::<String>(),
            ((self.cooldown as f64 / self.max_cooldown as f64) * 100.0) as i32,
            if self.last_degrees < 0 { "left" } else { "right" },
            ' ',
        )
    }
}
