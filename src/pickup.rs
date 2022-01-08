use super::icons::*;

#[derive(Debug, Clone, Copy)]
pub enum Pickup {
  Diamond,
  Nothing,
  Energy,
  Stone,
}

pub fn pickup_to_string(tile: Pickup, _wx: i32, _wy: i32) -> String {
  return match tile {
    Pickup::Diamond => ICON_DIAMOND.to_string(),
    Pickup::Nothing => "  ".to_string(),
    Pickup::Energy => format!("\x1b[93;40m{}\x1b[0m", ICON_ENERGY),
    Pickup::Stone => format!("{}", ICON_STONE),
  };
}
