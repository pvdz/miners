use std::fmt;
use rand_pcg::{Lcg128Xsl64};
use rand::distributions::{Distribution, Uniform};

use super::miner::*;
use super::world::*;
use super::movable::*;

pub trait Slottable: fmt::Display {
    fn before_paint(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World);
    fn paint(&self, painting: &mut Grid, world: &World);
    fn after_paint(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World);
    fn title(&self) -> &str;
    fn to_symbol(&self) -> &str;

    fn get_cooldown(&self) -> f32;
    fn set_cooldown(&mut self, v: f32) -> f32;
    fn get_max_cooldown(&self) -> f32;
    fn set_max_cooldown(&mut self, v: f32) -> f32;
}

#[derive(Clone, Copy)]
pub enum SlotType {
    Drill = 1,
    DroneLauncher = 2,
    Hammer = 3,
    Emptiness = 4,
    EnergyCell = 5,
    PurityScanner = 6,
    BrokenGps = 7,
}

pub const SLOT_COUNT: i32 = 7; // Must manually keep up to date with the enum ;(
// This function serves as a sanity check for the size constant
fn assert_size(x: &SlotType) -> i32 {
    match x {
        | SlotType::Drill
        | SlotType::DroneLauncher
        | SlotType::Hammer
        | SlotType::Emptiness
        | SlotType::EnergyCell
        | SlotType::PurityScanner
        | SlotType::BrokenGps
        => SLOT_COUNT, // Update SLOT_COUNT when this function updates
    }
}


pub fn get_random_slot(rng: &mut Lcg128Xsl64) -> SlotType {
    let slot_roller: Uniform<i32> = Uniform::from(0..SLOT_COUNT);
    match slot_roller.sample(rng) {
        0 => SlotType::Drill,
        1 => SlotType::DroneLauncher,
        2 => SlotType::Hammer,
        3 => SlotType::Emptiness,
        4 => SlotType::EnergyCell,
        5 => SlotType::PurityScanner,
        6 => SlotType::BrokenGps,
        _ => panic!("wat?"),
    }
}

pub fn slot_type_to_symbol(slot: &SlotType) -> String {
    match slot {
        SlotType::Drill => "d".to_string(),
        SlotType::DroneLauncher => "D".to_string(),
        SlotType::Hammer => "h".to_string(),
        SlotType::Emptiness => "e".to_string(),
        SlotType::EnergyCell => "E".to_string(),
        SlotType::PurityScanner => "P".to_string(),
        SlotType::BrokenGps => "G".to_string(),
    }
}
