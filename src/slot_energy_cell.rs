use super::utils::*;
use super::slottable::*;
use super::options::*;
use super::biome::*;

pub const TITLE_ENERGY_CELL: &str = "Energy Cell";

/**
 * An energy cell gives you an energy boost at a certain interval. It takes up n slots.
 */
pub fn create_slot_energy_cell(slot_index: usize, nth: i32, energy_bonus: i32, max_cooldown: f32) -> Slottable {
    assert!(max_cooldown > 0.0, "slot max cooldown should be non-zero: {}", max_cooldown);
    return Slottable {
        kind: SlotKind::EnergyCell,
        slot: slot_index,
        title: TITLE_ENERGY_CELL.to_owned(),
        max_cooldown, // max_energy,
        cur_cooldown: 0.0,
        nth,
        val: energy_bonus as f32,
        sum: 0.0,
    };
}

pub fn tick_slot_energy_cell(options: &mut Options, biome: &mut Biome, slot_index: usize) {
    let slot: &mut Slottable = &mut biome.miner.slots[slot_index];

    slot.cur_cooldown += 1.0;
    if slot.cur_cooldown >= slot.max_cooldown {
        biome.miner.movable.now_energy = (biome.miner.movable.now_energy + slot.val).max(0.0);
        slot.sum += slot.val;
        if biome.miner.movable.now_energy > biome.miner.movable.init_energy {
            biome.miner.movable.now_energy = biome.miner.movable.init_energy;
        }
        slot.cur_cooldown = 0.0;
    }
}

pub fn ui_slot_energy_cell(slot: &Slottable) -> (String, String, String) {
    return (
        TITLE_ENERGY_CELL.to_string(),
        progress_bar(20, slot.cur_cooldown, slot.max_cooldown, false),
        format!("Generated energy: {}", slot.sum)
    );
}
