use super::icons::*;
use super::color::*;
use super::options::*;

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

pub fn pickup_to_priority(pickup: Pickup) -> u32 {
  return match pickup {
    Pickup::Diamond => 100,
    Pickup::Nothing => 0,
    // This should be higher the less energy you have
    Pickup::Energy => 1000,
    Pickup::Stone => 1,
    Pickup::Expando => 0, // TBD
    Pickup::Fountain => 0, // TBD
    Pickup::Wind => 20,
    Pickup::Water => 20,
    Pickup::Wood => 20,
  }
}

pub fn pickup_to_string(pickup: Pickup, _wx: i32, _wy: i32) -> String {
  return match pickup {
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

pub fn pickup_add_color(str: &String, pickup: Pickup, value: u32, options: &Options) -> String {
  // Given a string, supposedly being the serialized pickup (pickup_to_string)
  // add a color to it according to its type and/or its value.
  // Each cell is assumed to start as reset. Only add foreground colors to the string.
  return match pickup {
    | Pickup::Nothing => str.to_string(),
    | Pickup::Energy => add_fg_color_with_reset(str, COLOR_ENERGY, options),
    | Pickup::Expando => add_fg_color_with_reset(str, COLOR_BLUE, options),
    | Pickup::Fountain => add_fg_color_with_reset(str, COLOR_FOUNTAIN, options),
    | Pickup::Water => add_fg_color_with_reset(str, COLOR_FOUNTAIN, options),
    | Pickup::Wind => add_fg_color_with_reset(str, COLOR_WIND, options),
    | Pickup::Wood => add_fg_color_with_reset(str, COLOR_WOOD, options),
    | Pickup::Stone
    | Pickup::Diamond =>
      match value {
        0 => add_fg_color_with_reset(str, COLOR_LEVEL_1, options),
        1 => add_fg_color_with_reset(str, COLOR_LEVEL_2, options),
        2 => add_fg_color_with_reset(str, COLOR_LEVEL_3, options),
        _ => panic!("wat"),
      },
  };
}
