use crate::icons::*;
use super::slottable::*;
use super::drone_san::*;

pub const TITLE_SANDRONE: &str = "SanDrone";

pub fn create_slot_sandrone(slot_index: usize, nth: i32) -> Slottable {
  return Slottable {
    kind: SlotKind::Sandrone,
    slot: slot_index,
    title: TITLE_SANDRONE.to_owned(),
    max_cooldown: 0.0,
    cur_cooldown: 0.0,
    nth,
    val: 0.0,
    sum: 0.0,
  };
}

pub fn ui_slot_sandrone(_sandrone_slot: &Slottable, sandrone: &Sandrone, sand: u32) -> (String, String, String) {
  return (
    TITLE_SANDRONE.to_string(),
    format!(
      " {}{}  ({: >2}/{})",
      std::iter::repeat(format!("{}", ICON_SAND)).take(sand.min(10) as usize).collect::<String>(),
      std::iter::repeat(format!("-")).take((10i32- sand as i32).max(0) as usize).collect::<String>(),
      sand.min(10),
      10,
    ),
    // This text is offset which is caused by the character printing as two but rust {: <30} counting it as one.
    format!("Area: {}. w: {}. h: {}. Walls: {}. Imps: {}. Open: {}. Path: {}. Pos: {},{}.",// Seeking: {}. back tracking: {}. air lifted: {}. {}",
      (sandrone.expansion_max_x - sandrone.expansion_min_x + 1) * (sandrone.expansion_max_y - sandrone.expansion_min_y + 1),
      sandrone.expansion_max_x - sandrone.expansion_min_x + 1,
      sandrone.expansion_max_y - sandrone.expansion_min_y + 1,
      sandrone.push_tiles.len(),
      sandrone.impassable_tiles.len(),
      ((sandrone.expansion_max_x - sandrone.expansion_min_x + 1) * (sandrone.expansion_max_y - sandrone.expansion_min_y + 1)) - ((sandrone.push_tiles.len() + sandrone.impassable_tiles.len()) as i32),
      ((sandrone.expansion_max_x - sandrone.expansion_min_x + 1) + (sandrone.expansion_max_y - sandrone.expansion_min_y + 1)) * 2 - 4,
      sandrone.movable.x,
      sandrone.movable.y,
      // sandrone.seeking,
      // sandrone.backtracking,
      // sandrone.air_lifted,
      // sandrone.status_desc.to_owned(),
    ),
    // // This text is offset which is caused by the character printing as two but rust {: <30} counting it as one.
    // format!("Area: {}. w: {}. h: {}. Walls: {}. Imps: {}. Open: {}. Path: {}. Pos: {},{}. Seeking: {}. back tracking: {}. air lifted: {}. {}",
    //   (sandrone.expansion_max_x - sandrone.expansion_min_x + 1) * (sandrone.expansion_max_y - sandrone.expansion_min_y + 1),
    //   sandrone.expansion_max_x - sandrone.expansion_min_x + 1,
    //   sandrone.expansion_max_y - sandrone.expansion_min_y + 1,
    //   sandrone.push_tiles.len(),
    //   sandrone.impassable_tiles.len(),
    //   ((sandrone.expansion_max_x - sandrone.expansion_min_x + 1) * (sandrone.expansion_max_y - sandrone.expansion_min_y + 1)) - ((sandrone.push_tiles.len() + sandrone.impassable_tiles.len()) as i32),
    //   ((sandrone.expansion_max_x - sandrone.expansion_min_x + 1) + (sandrone.expansion_max_y - sandrone.expansion_min_y + 1)) * 2 - 4,
    //   sandrone.movable.x,
    //   sandrone.movable.y,
    //   sandrone.seeking,
    //   sandrone.backtracking,
    //   sandrone.air_lifted,
    //   sandrone.status_desc.to_owned(),
    // ),
  );
}
