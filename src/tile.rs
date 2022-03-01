use super::icons::*;
use super::color::*;
use super::pickup::*;
use super::options::*;

#[derive(Debug, Clone, Copy)]
pub enum Tile {
  ExpandoWater,
  Empty,
  Fountain,
  Impassible,
  Push,
  Soil,
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

pub fn cell_to_uncolored_string(tile: Tile, pickup: Pickup, _tile_value: u32, wx: i32, wy: i32) -> String {
  return match tile {
    Tile::ExpandoWater => ICON_EXPANDO_WATER.to_string(),
    Tile::Fountain => ICON_FOUNTAIN.to_string(),
    Tile::Impassible => format!("{} ", ICON_IMPASSIBLE.to_string()),
    Tile::Push => format!("{} ", ICON_PUSH.to_string()),
    // Tile::Soil => format!("{: >2}", tile_value.min(99)),
    Tile::Soil => format!("  "),
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

pub fn cell_add_color(str: &String, tile: Tile, tile_value: u32, pickup: Pickup, options: &Options) -> String {
  // Given a string, supposedly being the serialized pickup (pickup_to_string)
  // add a color to it according to its type and/or its value.
  // Each cell is assumed to start as reset. Only add foreground colors to the string.
  return match tile {
    Tile::Push => add_bg_color_with_reset(str, COLOR_PUSH, options),
    | Tile::Wall1
    | Tile::Wall2
    | Tile::Wall3
    =>
      match tile_value {
        0 => add_fg_color_with_reset(str, COLOR_LEVEL_1, options),
        1 => add_fg_color_with_reset(str, COLOR_LEVEL_2, options),
        2 => add_fg_color_with_reset(str, COLOR_LEVEL_3, options),
        _ => panic!("unexpected cell value for a wall tile"),
      },
    | Tile::Empty => pickup_add_color(&str, pickup, tile_value, options),
    | Tile::ExpandoWater => add_bg_color_with_reset(&pickup_add_color(&str, pickup, tile_value, options), COLOR_EXPANDO_WATER, options),
    Tile::Fountain => add_bg_color_with_reset(&pickup_add_color(&str, pickup, tile_value, options), COLOR_FOUNTAIN, options),
    Tile::Impassible => add_bg_color_with_reset(str, COLOR_IMPOSSIBLE, options),
    Tile::Wall4 => str.to_string(),
    Tile::Soil => {
      match tile_value.min(10) {
        0 => add_bg_color_with_reset(str, COLOR_SOIL0, options),
        1 => add_bg_color_with_reset(str, COLOR_SOIL1, options),
        2 => add_bg_color_with_reset(str, COLOR_SOIL2, options),
        3 => add_bg_color_with_reset(str, COLOR_SOIL3, options),
        4 => add_bg_color_with_reset(str, COLOR_SOIL4, options),
        5 => add_bg_color_with_reset(str, COLOR_SOIL5, options),
        6 => add_bg_color_with_reset(str, COLOR_SOIL6, options),
        7 => add_bg_color_with_reset(str, COLOR_SOIL7, options),
        8 => add_bg_color_with_reset(str, COLOR_SOIL8, options),
        9 => add_bg_color_with_reset(str, COLOR_SOIL9, options),
        10 => add_bg_color_with_reset(str, COLOR_SOIL10, options),
        _ => { panic!("impossible"); },
      }
      // str.to_string()
    },

    Tile::ZeroZero => str.to_string(),
    Tile::TenLine => str.to_string(),
    Tile::HideWorld => str.to_string(),
    Tile::Test2 => str.to_string(),
    Tile::Test3 => str.to_string(),
  };
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
