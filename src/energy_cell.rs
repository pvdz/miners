use std::fmt;

use super::slottable::*;
use super::miner::*;
use super::world::*;
use super::movable::*;

pub const TITLE_ENERGY_CELL: &str = "Energy Cell";

/**
 * An energy cell gives you an energy boost at a certain interval. It takes up n slots.
 */
pub struct EnergyCell {
    pub energy_bonus: i32,
    pub generated: i32,
    pub max_cooldown: i32,
    pub cooldown: i32,
    // Offset zero. The how manieth energy cell is this? Every extra cell is half as efficient as the previous.
    pub nth: i32,
}

impl Slottable for EnergyCell {
    fn before_paint(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, _world: &mut World) {
        self.cooldown = self.cooldown + 1;
        if self.cooldown >= self.max_cooldown {
            miner_movable.energy = miner_movable.energy + self.energy_bonus;
            self.generated = self.generated + self.energy_bonus;
            if miner_movable.energy > miner_meta.max_energy {
                miner_movable.energy = miner_meta.max_energy;
            }
            self.cooldown = 0;
        }
    }

    fn paint(&self, painting: &mut Grid, world: &World) {}

    fn after_paint(&mut self, _miner_movable: &mut Movable, _miner_meta: &mut MinerMeta, _world: &mut World) {}

    fn title(&self) -> &str { return TITLE_ENERGY_CELL; }

    fn to_symbol(&self) -> &str { return "E"; }
}

impl fmt::Display for EnergyCell {
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
