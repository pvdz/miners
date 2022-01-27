use crate::icons::*;
use super::slottable::*;
use super::windrone::*;

pub const TITLE_WINDRONE: &str = "WinDrone";

pub fn create_slot_windrone(slot_index: usize, nth: i32) -> Slottable {
  return Slottable {
    kind: SlotKind::Windrone,
    slot: slot_index,
    title: TITLE_WINDRONE.to_owned(),
    max_cooldown: 0.0,
    cur_cooldown: 0.0,
    nth,
    val: 0.0,
    sum: 0.0,
  };
}

pub fn ui_slot_windrone(_windrone_slot: &Slottable, windrone: &Windrone, wind: u32) -> (String, String, String) {
  return (
    TITLE_WINDRONE.to_string(),
    format!(
      " {}{}  ({: >2}/{})",
      std::iter::repeat(format!("{} ", ICON_TORNADO)).take(wind.min(10) as usize).collect::<String>(),
      std::iter::repeat(format!(" -")).take((10i32-wind as i32).max(0) as usize).collect::<String>(),
      wind.min(10), 10
    ),
    windrone.status_desc.to_owned(),
  );
}
