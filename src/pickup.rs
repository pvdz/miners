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

pub fn pickup_add_color(str: &String, pickup: Pickup, value: u32) -> String {
  // Given a string, supposdly being the serialized pickup (pickup_to_string)
  // add a color to it according to its type and/or its value.
  // Each cell is assumed to start as reset. Only add foreground colors to the string.
  return match pickup {
    | Pickup::Nothing => str.to_string(),
    | Pickup::Energy => str.to_string(),
    | Pickup::Stone
    | Pickup::Diamond => format!(
      "{}{}\x1b[39;0m",
      match value {
        0 => "\x1b[30;0m",
        1 => "\x1b[32;1m",
        2 => "\x1b[34;1m",
        _ => panic!("wat"),
      },
      str
    ),
  };
}
