use std::fmt;
use rand_pcg::{Lcg128Xsl64};
use rand::distributions::{Distribution, Uniform};

use super::miner::*;
use super::world::*;
use super::movable::*;
use super::options::*;
use super::cell_contents::*;

pub struct Slottable {
    pub kind: SlotKind,
    pub title: String,
    pub max_cooldown: f32,
    pub cur_cooldown: f32,
    // Offset zero. This is the nth slottable of this kind
    pub nth: i32,
    // Generic scratch value / sum for this slottable
    pub val: i32,
    pub sum: i32,
}

/*
pub trait AsSlottable: fmt::Display {
    fn tick(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World, options: &Options);

    // This is the callback to paint this entity on the map. The callback should assume to be
    // painting itself onto one or two characters. It should return the cell to be printed and
    // the coordinate (x,y as i32) where it exists in the world model.
    // (Result may be discarded if out of view. We may optimize for this case in the future.)
    fn paint_entity(&self, world: &World, options: &Options) -> (Cell, i32, i32);

    // Print some UI info for this slottable. Should return the string as a vec, new new lines
    // and an arbitrary fixed max width that will be trunced if too long. Empty vec is ignored.
    fn paint_ui(&self, world: &World, options: &Options) -> Vec<char>;

    // Print something for debugging? Empty vec is ignored.
    fn paint_log(&self, world: &World, options: &Options) -> Vec<char>;

    fn title(&self) -> &str;
    fn to_symbol(&self) -> &str;

    fn get_cooldown(&self) -> f32;
    fn set_cooldown(&mut self, v: f32) -> f32;
    fn get_max_cooldown(&self) -> f32;
    fn set_max_cooldown(&mut self, v: f32) -> f32;
}
*/

#[derive(Clone, Copy)]
pub enum SlotKind {
    Drill = 0,
    DroneLauncher = 1,
    Hammer = 2,
    Emptiness = 3,
    EnergyCell = 4,
    PurityScanner = 5,
    BrokenGps = 6,
}
/*
match slot.kind {
  slottable::SlotKind::Emptiness => (), // noop
  slottable::SlotKind::EnergyCell => (), // noop
  slottable::SlotKind::DroneLauncher => (), // noop
  slottable::SlotKind::Hammer => (), // noop
  slottable::SlotKind::Drill => (), // noop
  slottable::SlotKind::PurityScanner => (), // noop
  slottable::SlotKind::BrokenGps => (), // noop
  _ => {
    panic!("Fix slot range generator in helix")
  },
}
*/

pub const SLOT_COUNT: i32 = 7; // Must manually keep up to date with the enum ;(
// This function serves as a sanity check for the size constant
fn assert_size(x: &SlotKind) -> i32 {
    match x {
        | SlotKind::Drill
        | SlotKind::DroneLauncher
        | SlotKind::Hammer
        | SlotKind::Emptiness
        | SlotKind::EnergyCell
        | SlotKind::PurityScanner
        | SlotKind::BrokenGps
        => SLOT_COUNT, // Update SLOT_COUNT when this function updates
    }
}


pub fn get_random_slot(rng: &mut Lcg128Xsl64) -> SlotKind {
    let slot_roller: Uniform<i32> = Uniform::from(0..SLOT_COUNT);
    match slot_roller.sample(rng) {
        0 => SlotKind::Drill,
        1 => SlotKind::DroneLauncher,
        2 => SlotKind::Hammer,
        3 => SlotKind::Emptiness,
        4 => SlotKind::EnergyCell,
        5 => SlotKind::PurityScanner,
        6 => SlotKind::BrokenGps,
        _ => panic!("wat?"),
    }
}

pub fn slot_type_to_symbol(slot: &SlotKind) -> String {
    match slot {
        SlotKind::Drill => "d".to_string(),
        SlotKind::DroneLauncher => "D".to_string(),
        SlotKind::Hammer => "h".to_string(),
        SlotKind::Emptiness => "e".to_string(),
        SlotKind::EnergyCell => "E".to_string(),
        SlotKind::PurityScanner => "P".to_string(),
        SlotKind::BrokenGps => "G".to_string(),
    }
}

pub fn create_slot_kind_counter() -> Vec<i32> {
    assert_eq!(SLOT_COUNT, 7);
    return vec![0; 7]; // One for every slot type value
}
