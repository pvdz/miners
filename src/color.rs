// Here we go ...

// These colors should be the part following `\x1b[38;` (fg color) or `\x1b[48;` (bg color)
// They can be 5;x (https://i.stack.imgur.com/KTSQa.png) or 2;r;g;b
pub const COLOR_LEVEL_1: &str = "5;7";
pub const COLOR_LEVEL_2: &str = "5;64";
pub const COLOR_LEVEL_3: &str = "5;124";
pub const COLOR_LEVEL_4: &str = "5;11";
pub const COLOR_DRONE: &str = "5;14";
pub const COLOR_ENERGY: &str = "5;11";
pub const COLOR_EXPANDO_WATER: &str = "5;18";
pub const COLOR_FOUNTAIN: &str = "5;105";
pub const COLOR_MINER: &str = "5;13";
pub const COLOR_GHOST: &str = "5;254";
pub const COLOR_PUSH: &str = "2;168;137;102"; // sand
pub const COLOR_WATER: &str = "5;105";
pub const COLOR_FOOD: &str = "5;2";
pub const COLOR_WOOD: &str = "2;139;69;19"; // "saddlebrown"
pub const COLOR_WIND: &str = "5;3";
pub const COLOR_SAND: &str = "5;174";

pub const COLOR_BLACK: &str = "5;0";
pub const COLOR_DARK_RED: &str = "5;1";
pub const COLOR_DARK_GREEN: &str = "5;2";
pub const COLOR_DARK_YELLOW: &str = "5;3";
pub const COLOR_DARK_BLUE: &str = "5;4";
pub const COLOR_PURPLE: &str = "5;5";
pub const COLOR_DARK_CYAN: &str = "5;6";
pub const COLOR_LIGHT_GREY: &str = "5;7";
pub const COLOR_GREY: &str = "5;8";
pub const COLOR_RED: &str = "5;9";
pub const COLOR_LIGHT_GREEN: &str = "5;10";
pub const COLOR_YELLOW: &str = "5;11";
pub const COLOR_BLUE: &str = "5;12";
pub const COLOR_PINK: &str = "5;13";
pub const COLOR_CYAN: &str = "5;14";
pub const COLOR_WHITE: &str = "5;15";

pub fn add_fg_color_with_reset(str: &String, color: &str) -> String {
  return format!("\x1b[38;{}m{}\x1b[0m", color, str);
}

pub fn add_bg_color_with_reset(str: &String, color: &str) -> String {
  return format!("\x1b[48;{}m{}\x1b[0m", color, str);
}

pub fn add_fg_color_with_reset_double(str: &String, color: &str) -> String {
  return format!("\x1b[38;{}m{}{}\x1b[0m", color, str, str);
}

pub fn add_bg_color_with_reset_double(str: &String, color: &str) -> String {
  return format!("\x1b[48;{}m{}{}\x1b[0m", color, str, str);
}
