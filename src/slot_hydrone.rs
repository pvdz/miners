use crate::icons::*;
use super::slottable::*;
use super::hydrone::*;

pub const TITLE_HYDRONE: &str = "HyDrone";

pub fn create_slot_hydrone(slot_index: usize, nth: i32) -> Slottable {
  return Slottable {
    kind: SlotKind::Hydrone,
    slot: slot_index,
    title: TITLE_HYDRONE.to_owned(),
    max_cooldown: 0.0,
    cur_cooldown: 0.0,
    nth,
    val: 0.0,
    sum: 0.0,
  };
}

pub fn ui_slot_hydrone(_hydrone_slot: &Slottable, hydrone: &Hydrone, water: u32) -> (String, String, String) {
  return (
    TITLE_HYDRONE.to_string(),
    format!(
      " {}{}  ({: >2}/{})",
      std::iter::repeat(format!("{}", ICON_WATER)).take(water.min(10) as usize).collect::<String>(),
      std::iter::repeat(format!("-")).take((10i32- water as i32).max(0) as usize).collect::<String>(),
      water.min(10),
      10,
    ),
    // This text is offset which is caused by the character printing as two but rust {: <30} counting it as one.
    format!("Blocks: {}. Pos: {},{}. Seeking: {}. back tracking: {}. air lifted: {}. {}",
      hydrone.push_tiles.len(),
      hydrone.movable.x,
      hydrone.movable.y,
      hydrone.seeking,
      hydrone.backtracking,
      hydrone.air_lifted,
      hydrone.status_desc.to_owned(),
    ),
  );
}
