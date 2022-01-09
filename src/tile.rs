use super::icons::*;
use super::pickup::*;

#[derive(Debug, Clone, Copy)]
pub enum Tile {
  DroneDown,
  DroneLeft,
  DroneRight,
  DroneUp,
  Empty,
  Wall1,
  Wall2,
  Wall3,
  Wall4,
  ZeroZero,

  // Debug stuff
  TenLine,
  HideWorld,
  Test2,
  Test3,
}

pub fn cell_to_uncolored_string(tile: Tile, pickup: Pickup, wx: i32, wy: i32) -> String {
  return match tile {
    Tile::DroneDown => ICON_DRONE_DOWN.to_string(),
    Tile::DroneLeft => ICON_DRONE_LEFT.to_string(),
    Tile::DroneRight => ICON_DRONE_RIGHT.to_string(),
    Tile::DroneUp => ICON_DRONE_UP.to_string(),
    Tile::Wall1 => format!("{}{}", ICON_BLOCK_25, ICON_BLOCK_25),
    Tile::Wall2 => format!("{}{}", ICON_BLOCK_50, ICON_BLOCK_50),
    Tile::Wall3 => format!("{}{}", ICON_BLOCK_75, ICON_BLOCK_75),
    Tile::Wall4 => format!("{}{}", ICON_BLOCK_100, ICON_BLOCK_100),

    // Debugging
    Tile::TenLine => ten_line_cell(wx, wy),
    Tile::ZeroZero => format!("{}", ICON_DEBUG_ORIGIN),
    Tile::HideWorld => {
      if wx % 10 == 0 || wy % 10 == 0 { ten_line_cell(wx, wy) }
      else { format!("{}{}", ICON_BLOCK_25, ICON_BLOCK_25) }
    }, // ⚽ :ball:
    Tile::Test2 => panic!("Enable me 2"),
    Tile::Test3 => panic!("Enable me 3"),

    // Pickups (only when cell is empty)
    Tile::Empty => pickup_to_string(pickup, wx, wy),
  }
}

pub fn ten_line_cell(wx: i32, wy: i32) -> String {
  if wy % 10 == 0 {
    if wx % 10 == 0 {
      if wx == 0 && wy == 0 {
        return "##".to_string();
      }
      return "  ".to_string();
    }

    // Horizontal line
    return format!(" {}", wx.abs()%10);
  }

  // Vertical line
  return format!(" {}", wy.abs()%10);
}

pub fn cell_add_color(str: &String, tile: Tile, value: u32, pickup: Pickup) -> String {
  // Given a string, supposedly being the serialized pickup (pickup_to_string)
  // add a color to it according to its type and/or its value.
  // Each cell is assumed to start as reset. Only add foreground colors to the string.
  return match tile {
    | Tile::Wall1
    | Tile::Wall2
    | Tile::Wall3
    =>
      format!(
        "{}{}\x1b[0m",
        match value {
          0 => "\x1b[39m", // default fg color, not necessarily black. no attributes.
          1 => "\x1b[39;48;5;17m", // 39=default fg, 48;5 = color 17 (dark green). 48;2;r;g;b for rgb mode.
          2 => "\x1b[39;48;5;22m", // 39=default fg, 48;5 = color 22 (dark blue). 48;2;r;g;b for rgb mode.
          _ => panic!("wat"),
        },
        str
      ),
    | Tile::Empty => pickup_add_color(&str, pickup, value),
    _ => str.to_string(),
  };
}
