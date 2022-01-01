use std::fmt;

use super::slottable::*;
// use super::miner::*;
// use super::movable::*;
// use super::world::*;
// use super::options::*;
// use super::cell_contents::*;

pub const TITLE_DRILL: &str = "Drill";

pub fn create_drill(nth: i32) -> Slottable {
    return Slottable {
        kind: SlotKind::BrokenGps,
        title: TITLE_DRILL.to_owned(),
        max_cooldown: 0.0,
        cur_cooldown: 0.0,
        nth,
        val: 0,
        sum: 0,
    };
}

/*
pub struct Drill {}

impl Slottable for Drill {
    fn tick(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World, options: &Options) {}

    fn paint_entity(&self, world: &World, options: &Options) -> (Cell, i32, i32) { return (Cell::Empty, 0, 0); }
    fn paint_ui(&self, world: &World, options: &Options) -> Vec<char> { vec!() }
    fn paint_log(&self, world: &World, options: &Options) -> Vec<char> { vec!() }

    fn title(&self) -> &str { return TITLE_DRILL; }

    fn to_symbol(&self) -> &str { return "d"; }

    fn get_cooldown(&self) -> f32 {
        return 0.0;
    }

    fn set_cooldown(&mut self, _v: f32) -> f32 {
        return 0.0;
    }

    fn get_max_cooldown(&self) -> f32 {
        return 0.0;
    }

    fn set_max_cooldown(&mut self, _v: f32) -> f32 {
        return 0.0;
    }
}

impl fmt::Display for Drill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: >100}", ' ')
    }
}
*/
