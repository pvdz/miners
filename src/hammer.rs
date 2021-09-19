use std::fmt;

use crate::slottable::*;
use crate::miner::*;
use crate::movable::*;
use crate::world::*;

pub const TITLE_HAMMER: &str = "Hammer";

pub struct Hammer {}

impl Slottable for Hammer {
    fn before_paint(&mut self, _miner_movable: &mut Movable, _miner_meta: &mut MinerMeta, _world: &mut World) {
        // Do nothing
    }

    fn paint(&self, painting: &mut Grid, world: &World) {}

    fn after_paint(&mut self, _miner_movable: &mut Movable, _miner_meta: &mut MinerMeta, _world: &mut World) {}

    fn title(&self) -> &str { return TITLE_HAMMER; }
}

impl fmt::Display for Hammer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: >100}", ' ')
    }
}
