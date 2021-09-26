use std::fmt;

use super::slottable::*;
use super::miner::*;
use super::movable::*;
use super::world::*;

pub const TITLE_EMPTINESS: &str = "Empty";

pub struct Emptiness {}

impl Slottable for Emptiness {
    fn before_paint(&mut self, _miner_movable: &mut Movable, _miner_meta: &mut MinerMeta, _world: &mut World) {
        // Do nothing
    }

    fn paint(&self, painting: &mut Grid, world: &World) {}

    fn after_paint(&mut self, _miner_movable: &mut Movable, _miner_meta: &mut MinerMeta, _world: &mut World) {}

    fn title(&self) -> &str { return TITLE_EMPTINESS; }

    fn to_symbol(&self) -> &str { return "e"; }

    fn get_cooldown(&self) -> f32 {
        return 0.0;
    }

    fn set_cooldown(&mut self, v: f32) -> f32 {
        return 0.0;
    }

    fn get_max_cooldown(&self) -> f32 {
        return 0.0;
    }

    fn set_max_cooldown(&mut self, v: f32) -> f32 {
        return 0.0;
    }
}

impl fmt::Display for Emptiness {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: >100}", ' ')
    }
}
