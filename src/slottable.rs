use rand_pcg::{Lcg128Xsl64};
use rand::distributions::{Distribution, Uniform};

// use super::tile::*;

#[derive(Debug)]
pub struct Slottable {
    pub kind: SlotKind,
    pub slot: usize,
    pub title: String,
    pub max_cooldown: f32,
    pub cur_cooldown: f32,
    // Offset zero. This is the nth slottable of this kind
    pub nth: i32,
    // Generic scratch value / sum for this slottable
    pub val: f32,
    pub sum: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum SlotKind {
    Drill = 0,
    DroneLauncher = 1,
    Hammer = 2,
    Emptiness = 3,
    EnergyCell = 4,
    PurityScanner = 5,
    BrokenGps = 6,
}

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
    return match slot_roller.sample(rng) {
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
    return match slot {
        SlotKind::Drill => "d".to_string(),
        SlotKind::DroneLauncher => "D".to_string(),
        SlotKind::Hammer => "h".to_string(),
        SlotKind::Emptiness => ".".to_string(),
        SlotKind::EnergyCell => "E".to_string(),
        SlotKind::PurityScanner => "P".to_string(),
        SlotKind::BrokenGps => "G".to_string(),
    };
}

pub fn symbol_to_slot_type(sym: char) -> SlotKind {
    match sym {
        'd' => SlotKind::Drill,
        'D' => SlotKind::DroneLauncher,
        'h' => SlotKind::Hammer,
        '.' => SlotKind::Emptiness,
        'E' => SlotKind::EnergyCell,
        'P' => SlotKind::PurityScanner,
        'G' => SlotKind::BrokenGps,
        _ => panic!("add me, {}", sym),
    }
}

pub fn create_slot_kind_counter() -> Vec<i32> {
    assert_eq!(SLOT_COUNT, 7);
    return vec![0; 7]; // One for every slot type value
}

pub fn slots_to_short_string(slots: [SlotKind; 32]) -> String {
    slots.iter().map(|slot| slot_type_to_symbol(&slot)).collect()
}

pub fn short_string_to_slots(short_string: String) -> [SlotKind; 32] {
    assert_eq!(short_string.len(), 32, "the short_string should be exactly 32 characters; one for each slot");
    // Note: can't do the map().collect() thing so we manually loop.
    let bytes = short_string.as_bytes();
    let mut arr = [SlotKind::Emptiness; 32];
    for i in 0..32 {
        arr[i] = symbol_to_slot_type(bytes[i] as char);
    }
    return arr;
}
