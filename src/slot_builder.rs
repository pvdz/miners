use crate::icons::ICON_TORNADO;
use super::utils::*;
use super::slottable::*;

pub const TITLE_BUILDER: &str = "Builder Drone";

pub fn create_slot_builder(slot_index: usize, nth: i32) -> Slottable {
  return Slottable {
    kind: SlotKind::Builder,
    slot: slot_index,
    title: TITLE_BUILDER.to_owned(),
    max_cooldown: 0.0,
    cur_cooldown: 0.0,
    nth,
    val: 0.0,
    sum: 0.0,
  };
}

pub fn ui_slot_builder(slot: &Slottable, wind: u32) -> (String, String, String) {
  //
  // let progress = ((cur_cooldown / max_cooldown) * bar_max_width as f32).min(bar_max_width as f32) as usize;
  // let remaining = bar_max_width - progress;
  // return format!(
  //     "[{}{}] ({: >3}%)",
  //     std::iter::repeat('|').take(progress).collect::<String>(),
  //     std::iter::repeat('-').take(remaining).collect::<String>(),
  //     ((cur_cooldown / max_cooldown) * 100.0) as i32,
  // )

  return (
    TITLE_BUILDER.to_string(),
    format!(
      " {}{}  ({: >2}/{})",
      std::iter::repeat(format!("{} ", ICON_TORNADO)).take(wind.min(10) as usize).collect::<String>(),
      std::iter::repeat(format!(" -")).take((10i32-wind as i32).max(0) as usize).collect::<String>(),
      wind.min(10), 10
    ),
    // progress_bar(20, slot.cur_cooldown, slot.max_cooldown, false),
    "Waiting for liftoff...".to_string()
  );
}

pub fn tick_builder(builder: &Slottable) {

}
