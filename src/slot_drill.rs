use super::utils::*;
use super::slottable::*;

pub const TITLE_DRILL: &str = "Drill";

pub fn create_drill(slot_index: usize, nth: i32) -> Slottable {
  return Slottable {
    kind: SlotKind::Drill,
    slot: slot_index,
    title: TITLE_DRILL.to_owned(),
    max_cooldown: 0.0,
    cur_cooldown: 0.0,
    nth,
    val: 0.0,
    sum: 0.0,
  };
}

pub fn ui_slot_drill(slot: &Slottable) -> (String, String, String) {
  return (
    TITLE_DRILL.to_string(),
    progress_bar(20, slot.cur_cooldown, slot.max_cooldown, false),
    "".to_string()
  );
}
