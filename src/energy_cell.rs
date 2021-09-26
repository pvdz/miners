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
    pub max_cooldown: f32,
    pub cooldown: f32,
    // Offset zero. The how manieth energy cell is this? Every extra cell is half as efficient as the previous.
    pub nth: i32,
}

impl Slottable for EnergyCell {
    fn before_paint(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, _world: &mut World) {
        self.set_cooldown(self.get_cooldown() + 1.0);
        if self.get_cooldown() >= self.get_max_cooldown() {
            miner_movable.energy = miner_movable.energy + self.energy_bonus;
            self.generated = self.generated + self.energy_bonus;
            if miner_movable.energy > miner_meta.max_energy {
                miner_movable.energy = miner_meta.max_energy;
            }
            self.set_cooldown(0.0);
        }
    }

    fn paint(&self, painting: &mut Grid, world: &World) {}

    fn after_paint(&mut self, _miner_movable: &mut Movable, _miner_meta: &mut MinerMeta, _world: &mut World) {}

    fn title(&self) -> &str { return TITLE_ENERGY_CELL; }

    fn to_symbol(&self) -> &str { return "E"; }

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

impl fmt::Display for EnergyCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        write!(
            f,
            "{}{} {: >3}% (generated: {}) {: >100}",
            std::iter::repeat('|').take(((self.get_cooldown() / self.get_max_cooldown()) * 10.0) as usize).collect::<String>(),
            std::iter::repeat('-').take(10 - ((self.get_cooldown() as f64 / self.get_max_cooldown() as f64) * 10.0) as usize).collect::<String>(),
            ((self.get_cooldown() / self.get_max_cooldown()) * 100.0) as i32,
            self.generated,
            ' ',
        )
    }
}
