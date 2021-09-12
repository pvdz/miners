use std::fmt;

use crate::miner::*;
use crate::world::*;
use crate::movable::*;

pub trait Slottable: fmt::Display {
    fn before_paint(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World);
    fn paint(&self, world: &mut World);
    fn after_paint(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World);
    fn title(&self) -> &str;
}
