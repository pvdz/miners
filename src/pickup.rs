use super::icons::*;
use super::color::*;

#[derive(Debug, Clone, Copy)]
pub enum Pickup {
  Diamond,
  Nothing,
  Energy,
  Stone,
  Expando,
  Fountain,
  Wind,
  Water,
  Wood,
}

pub fn pickup_to_string(tile: Pickup, _wx: i32, _wy: i32) -> String {
  return match tile {
    Pickup::Diamond => ICON_DIAMOND.to_string(),
    Pickup::Energy => ICON_ENERGY.to_string(),
    Pickup::Expando => format!("{}", ICON_EXPANDO_WATER),
    Pickup::Fountain => format!("{}", ICON_FOUNTAIN),
    Pickup::Nothing => "  ".to_string(),
    Pickup::Stone => format!("{}", ICON_STONE),
    Pickup::Water => format!("{}", ICON_WATER),
    Pickup::Wind => format!("{} ", ICON_TORNADO),
    Pickup::Wood => format!("{}", ICON_WOOD),
  };
}

pub fn pickup_add_color(str: &String, pickup: Pickup, value: u32) -> String {
  // Given a string, supposedly being the serialized pickup (pickup_to_string)
  // add a color to it according to its type and/or its value.
  // Each cell is assumed to start as reset. Only add foreground colors to the string.
  return match pickup {
    | Pickup::Nothing => str.to_string(),
    | Pickup::Energy => add_fg_color_with_reset(str, COLOR_ENERGY),
    | Pickup::Expando => add_fg_color_with_reset(str, COLOR_BLUE),
    | Pickup::Fountain => add_fg_color_with_reset(str, COLOR_FOUNTAIN),
    | Pickup::Water => add_fg_color_with_reset(str, COLOR_FOUNTAIN),
    | Pickup::Wind => add_fg_color_with_reset(str, COLOR_WIND),
    | Pickup::Wood => add_fg_color_with_reset(str, COLOR_WOOD),
    | Pickup::Stone
    | Pickup::Diamond =>
      match value {
        0 => add_fg_color_with_reset(str, COLOR_LEVEL_1),
        1 => add_fg_color_with_reset(str, COLOR_LEVEL_2),
        2 => add_fg_color_with_reset(str, COLOR_LEVEL_3),
        _ => panic!("wat"),
      },
  };
}
