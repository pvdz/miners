use std::fmt;

use super::slottable::*;
use super::miner::*;
use super::world::*;
use super::movable::*;

pub const TITLE_PURITY_SCANNER: &str = "Purity Scanner";

/**
 * A purity scanner gives your next gem double points. Has a cooldown that doubles with
 * each additional scanner you get.
 */
pub struct PurityScanner {
    // pub point_bonus: i32, // Do we want to make this somehow scaling rather than absolute double?
    pub max_cooldown: i32,
    pub cooldown: i32,
    pub generated: i32,
    // Offset zero. The how manieth purity scanner is this? Every extra scanner is half as efficient as the previous.
    pub nth: i32,
}

impl Slottable for PurityScanner {
    fn before_paint(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, _world: &mut World) {
        if self.cooldown < self.max_cooldown {
            self.cooldown = self.cooldown + 1;
        }
        if self.cooldown >= self.max_cooldown && miner_meta.points_last_move > 0 {
            miner_meta.points = miner_meta.points + miner_meta.points_last_move;
            self.generated = self.generated + miner_meta.points_last_move;
            self.cooldown = 0;
        }
    }

    fn paint(&self, painting: &mut Grid, world: &World) {}

    fn after_paint(&mut self, _miner_movable: &mut Movable, _miner_meta: &mut MinerMeta, _world: &mut World) {}

    fn title(&self) -> &str { return TITLE_PURITY_SCANNER; }

    fn to_symbol(&self) -> &str { return "P"; }
}

impl fmt::Display for PurityScanner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        write!(
            f,
            "{}{} {: >3}% (generated: {}) {: >100}",
            std::iter::repeat('|').take(((self.cooldown as f32 / self.max_cooldown as f32) * 10.0) as usize).collect::<String>(),
            std::iter::repeat('-').take(10 - ((self.cooldown as f64 / self.max_cooldown as f64) * 10.0) as usize).collect::<String>(),
            ((self.cooldown as f64 / self.max_cooldown as f64) * 100.0) as i32,
            self.generated,
            ' ',
        )
    }
}
