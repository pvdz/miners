use super::options::*;

// These colors should be the part following `\x1b[38;` (fg color) or `\x1b[48;` (bg color)
// They can be 5;x (https://i.stack.imgur.com/KTSQa.png) or 2;r;g;b
pub const COLOR_LEVEL_1: &str = "190,190,190";
pub const COLOR_LEVEL_2: &str = "85,125,0";
pub const COLOR_LEVEL_3: &str = "170,0,0";
pub const COLOR_LEVEL_4: &str = "255,255,0";
pub const COLOR_DRONE: &str = "0,255,255";
pub const COLOR_ENERGY: &str = "255,255,0";
pub const COLOR_EXPANDO_WATER: &str = "0,0,135";
pub const COLOR_FOUNTAIN: &str = "135,135,255";
pub const COLOR_MINER: &str = "255,0,255";
pub const COLOR_GHOST: &str = "228,228,228";
pub const COLOR_PUSH: &str = "168,137,102"; // sand
pub const COLOR_WATER: &str = "0,0,255";
pub const COLOR_FOOD: &str = "0,128,0";
pub const COLOR_WOOD: &str = "139,69,19"; // "saddlebrown"
pub const COLOR_WIND: &str = "128,128,0";
pub const COLOR_SAND: &str = "215,135,135";
pub const COLOR_IMPOSSIBLE: &str = "75,55,13";

pub const COLOR_SOIL0: &str = "75,55,13";
pub const COLOR_SOIL1: &str = "70,63,17";
pub const COLOR_SOIL2: &str = "64,73,22";
pub const COLOR_SOIL3: &str = "56,85,29";
pub const COLOR_SOIL4: &str = "47,97,36";
pub const COLOR_SOIL5: &str = "40,108,42";
pub const COLOR_SOIL6: &str = "33,118,47";
pub const COLOR_SOIL7: &str = "27,126,52";
pub const COLOR_SOIL8: &str = "21,135,56";
pub const COLOR_SOIL9: &str = "11,149,64";
pub const COLOR_SOIL10: &str = "0,163,71";

pub const COLOR_BLACK: &str = "0,0,0";
pub const COLOR_DARK_RED: &str = "128,0,0";
pub const COLOR_DARK_GREEN: &str = "0,128,0";
pub const COLOR_DARK_YELLOW: &str = "128,128,0";
pub const COLOR_DARK_BLUE: &str = "0,0,128";
pub const COLOR_PURPLE: &str = "128,0,128";
pub const COLOR_DARK_CYAN: &str = "0,128,128";
pub const COLOR_LIGHT_GREY: &str = "192,192,192";
pub const COLOR_GREY: &str = "128,128,128";
pub const COLOR_RED: &str = "255,0,0";
pub const COLOR_LIGHT_GREEN: &str = "0,255,0";
pub const COLOR_YELLOW: &str = "255,255,0";
pub const COLOR_BLUE: &str = "0,255,255";
pub const COLOR_PINK: &str = "255,0,255";
pub const COLOR_CYAN: &str = "0,255,255";
pub const COLOR_WHITE: &str = "255,255,255";

pub fn add_fg_color_with_reset(str: &String, color: &str, options: &Options) -> String {
  if !options.paint_colors || !options.paint_fg_colors {
    return format!("{}", str);
  }
  return if options.html_mode {
    format!("<span style='color:rgb({})'>{}</span>", color, str)
  } else {
    format!("\x1b[38;2;{}m{}\x1b[0m", color, str)
  }
}

pub fn add_bg_color_with_reset(str: &String, color: &str, options: &Options) -> String {
  if !options.paint_colors || !options.paint_bg_colors {
    return format!("{}", str);
  }

  return if options.html_mode {
    format!("<span style='background-color:rgb({})'>{}</span>", color, str)
  } else {
    format!("\x1b[48;{}m{}\x1b[0m", color, str)
  }
}

pub fn add_fg_color_with_reset_double(str: &String, color: &str, options: &Options) -> String {
  if !options.paint_colors || !options.paint_fg_colors {
    return format!("{}", str);
  }

  return if options.html_mode {
    format!("<span style='color:rgb({})'>{}{}</span>", color, str, str)
  } else {
    format!("\x1b[38;{}m{}{}\x1b[0m", color, str, str)
  }
}

pub fn add_bg_color_with_reset_double(str: &String, color: &str, options: &Options) -> String {
  if !options.paint_colors || !options.paint_bg_colors {
    return format!("{}", str);
  }

  return if options.html_mode {
    format!("<span style='background-color:rgb({})'>{}{}</span>", color, str, str)
  } else {
    format!("\x1b[48;{}m{}{}\x1b[0m", color, str, str)
  }
}
