use super::icons::*;

#[derive(Debug, Clone, Copy)]
pub enum Tile {
  Diamond,
  DroneDown,
  DroneLeft,
  DroneRight,
  DroneUp,
  Empty,
  Energy,
  Stone,
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

pub fn tile_to_string(tile: Tile, wx: i32, wy: i32) -> String {
  return match tile {
    Tile::Diamond => ICON_DIAMOND.to_string(),
    Tile::DroneDown => ICON_DRONE_DOWN.to_string(),
    Tile::DroneLeft => ICON_DRONE_LEFT.to_string(),
    Tile::DroneRight => ICON_DRONE_RIGHT.to_string(),
    Tile::DroneUp => ICON_DRONE_UP.to_string(),
    Tile::Empty => "  ".to_string(),
    Tile::Energy => ICON_ENERGY.to_string(),
    Tile::Stone => ICON_ROCK.to_string(),
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
