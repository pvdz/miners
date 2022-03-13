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

pub const SLOT_COUNT: i32 = 11; // Must manually keep up to date with the enum ;(
#[derive(Debug, Clone, Copy)]
pub enum SlotKind {
    BrokenGps = 0,
    Windrone = 1,
    Drill = 2,
    DroneLauncher = 3,
    Emptiness = 4,
    EnergyCell = 5,
    Hammer = 6,
    JacksCompass = 7,
    Magnet = 8,
    PurityScanner = 9,
    RandomStart = 10,
    Sandrone = 11,
    // Make sure to update the SLOT_COUNT!
}

// This function serves as a sanity check for the size constant
fn assert_size(x: &SlotKind) -> i32 {
    match x {
        | SlotKind::BrokenGps
        | SlotKind::Drill
        | SlotKind::DroneLauncher
        | SlotKind::Emptiness
        | SlotKind::EnergyCell
        | SlotKind::Hammer
        | SlotKind::JacksCompass
        | SlotKind::Magnet
        | SlotKind::PurityScanner
        | SlotKind::RandomStart
        | SlotKind::Sandrone
        | SlotKind::Windrone
        => SLOT_COUNT, // Update SLOT_COUNT when this function updates
    }
}

pub fn get_random_slot(rng: &mut Lcg128Xsl64) -> SlotKind {
    assert_size(&SlotKind::Drill);
    let slot_roller: Uniform<i32> = Uniform::from(0..SLOT_COUNT);
    return match slot_roller.sample(rng) {
        0 => SlotKind::BrokenGps,
        1 => SlotKind::Drill,
        2 => SlotKind::DroneLauncher,
        3 => SlotKind::Emptiness,
        4 => SlotKind::EnergyCell,
        5 => SlotKind::Hammer,
        6 => SlotKind::Magnet,
        7 => SlotKind::JacksCompass,
        8 => SlotKind::PurityScanner,
        // Must add empty slots for slotkinds that we don't want to generate (TODO: no we dont)
        9 => SlotKind::Emptiness,  // SlotKind::Windrone
        10 => SlotKind::Emptiness, // SlotKind::Sandrone
        11 => SlotKind::Emptiness, // SlotKind::RandomStart // Preventloop
        _ => panic!("wat?"),
    }
}

pub fn slot_type_to_symbol(slot: &SlotKind) -> String {
    return match slot {
        SlotKind::BrokenGps => "G".to_string(),
        SlotKind::Drill => "d".to_string(),
        SlotKind::DroneLauncher => "D".to_string(),
        SlotKind::Emptiness => ".".to_string(),
        SlotKind::EnergyCell => "E".to_string(),
        SlotKind::Hammer => "h".to_string(),
        SlotKind::JacksCompass => "J".to_string(),
        SlotKind::Magnet => "m".to_string(),
        SlotKind::PurityScanner => "P".to_string(),
        SlotKind::RandomStart => "?".to_string(),
        SlotKind::Sandrone => "H".to_string(),
        SlotKind::Windrone => "B".to_string(),
    };
}

pub fn symbol_to_slot_type(sym: char) -> SlotKind {
    match sym {
        'G' => SlotKind::BrokenGps,
        'd' => SlotKind::Drill,
        'D' => SlotKind::DroneLauncher,
        '.' => SlotKind::Emptiness,
        'E' => SlotKind::EnergyCell,
        'h' => SlotKind::Hammer,
        'J' => SlotKind::JacksCompass,
        'm' => SlotKind::Magnet,
        'P' => SlotKind::PurityScanner,
        '?' => SlotKind::RandomStart,
        'H' => SlotKind::Sandrone,
        'B' => SlotKind::Windrone,
        // ?: random
        _ => panic!("add me, {}", sym),
    }
}

pub fn create_slot_kind_counter() -> Vec<i32> {
    assert_eq!(SLOT_COUNT, 11);
    return vec![0; SLOT_COUNT as usize]; // One for every slot type value
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
