use std::fmt;

use super::slottable::*;
use super::miner::*;
use super::movable::*;
use super::world::*;

pub const TITLE_DRILL: &str = "Drill";

pub struct Drill {}

impl Slottable for Drill {
    fn before_paint(&mut self, _miner_movable: &mut Movable, _miner_meta: &mut MinerMeta, _world: &mut World) {
        // Do nothing
    }

    fn paint(&self, painting: &mut Grid, world: &World) {}

    fn after_paint(&mut self, _miner_movable: &mut Movable, _miner_meta: &mut MinerMeta, _world: &mut World) {}

    fn title(&self) -> &str { return TITLE_DRILL; }

    fn to_symbol(&self) -> &str { return "d"; }
}

impl fmt::Display for Drill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: >100}", ' ')
    }
}
