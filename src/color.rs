use super::options::*;

macro_rules! rgb {
  ( $r:literal , $g:literal , $b:literal) => {
    if cfg!(target_arch = "wasm32") {
      // Web; css `rgb(r,g,b)` requires commas
      concat!(stringify!($r), ",", stringify!($g), ",", stringify!($b))
    } else {
      // CLI; ansi color codes `2;r;g;bm` requires semi's
      concat!(stringify!($r), ";", stringify!($g), ";", stringify!($b))
    }
  }
}

// These colors should be the part following `\x1b[38;` (fg color) or `\x1b[48;` (bg color)
// They can be 5;x (https://i.stack.imgur.com/KTSQa.png) or 2;r;g;b
pub const COLOR_LEVEL_1: &str = rgb!(190,190,190);
pub const COLOR_LEVEL_2: &str = rgb!(85,125,0);
pub const COLOR_LEVEL_3: &str = rgb!(170,0,0);
pub const COLOR_LEVEL_4: &str = rgb!(255,255,0);
pub const COLOR_DRONE: &str = rgb!(0,255,255);
pub const COLOR_ENERGY: &str = rgb!(255,255,0);
pub const COLOR_EXPANDO_WATER: &str = rgb!(0,0,135);
pub const COLOR_FOUNTAIN: &str = rgb!(135,135,255);
pub const COLOR_MINER: &str = rgb!(255,0,255);
pub const COLOR_GHOST: &str = rgb!(228,228,228);
pub const COLOR_PUSH: &str = rgb!(168,137,102); // sand
pub const COLOR_WATER: &str = rgb!(0,0,255);
pub const COLOR_FOOD: &str = rgb!(0,128,0);
pub const COLOR_WOOD: &str = rgb!(139,69,19); // "saddlebrown"
pub const COLOR_WIND: &str = rgb!(128,128,0);
pub const COLOR_SAND: &str = rgb!(215,135,135);
pub const COLOR_IMPOSSIBLE: &str = rgb!(75,55,13);

pub const COLOR_SOIL0: &str = rgb!(75,55,13);
pub const COLOR_SOIL1: &str = rgb!(70,63,17);
pub const COLOR_SOIL2: &str = rgb!(64,73,22);
pub const COLOR_SOIL3: &str = rgb!(56,85,29);
pub const COLOR_SOIL4: &str = rgb!(47,97,36);
pub const COLOR_SOIL5: &str = rgb!(40,108,42);
pub const COLOR_SOIL6: &str = rgb!(33,118,47);
pub const COLOR_SOIL7: &str = rgb!(27,126,52);
pub const COLOR_SOIL8: &str = rgb!(21,135,56);
pub const COLOR_SOIL9: &str = rgb!(11,149,64);
pub const COLOR_SOIL10: &str = rgb!(0,163,71);

pub const COLOR_BLACK: &str = rgb!(0,0,0);
pub const COLOR_DARK_RED: &str = rgb!(128,0,0);
pub const COLOR_DARK_GREEN: &str = rgb!(0,128,0);
pub const COLOR_DARK_YELLOW: &str = rgb!(128,128,0);
pub const COLOR_DARK_BLUE: &str = rgb!(0,0,128);
pub const COLOR_PURPLE: &str = rgb!(128,0,128);
pub const COLOR_DARK_CYAN: &str = rgb!(0,128,128);
pub const COLOR_LIGHT_GREY: &str = rgb!(192,192,192);
pub const COLOR_GREY: &str = rgb!(128,128,128);
pub const COLOR_RED: &str = rgb!(255,0,0);
pub const COLOR_LIGHT_GREEN: &str = rgb!(0,255,0);
pub const COLOR_YELLOW: &str = rgb!(255,255,0);
pub const COLOR_BLUE: &str = rgb!(0,255,255);
pub const COLOR_PINK: &str = rgb!(255,0,255);
pub const COLOR_CYAN: &str = rgb!(0,255,255);
pub const COLOR_WHITE: &str = rgb!(255,255,255);

fn get_fg_open_html(options: &Options, color: &str) -> String {
  return if options.html_mode {
    // Browser emojis can be colored (depends in linux; firefox yes, chrome no)
    // This hack works around it so we can (mono) color them as we please
    // https://stackoverflow.com/questions/32413731/color-for-unicode-emoji
    format!("<span style='color: transparent; text-shadow: 0 0 0 rgb({})'>", color)
  } else {
    format!("\x1b[38;2;{}m", color)
  }
}

fn get_bg_open_html(options: &Options, color: &str) -> String {
  return if options.html_mode {
    format!("<span style='background-color:rgb({})'>", color)
  } else {
    format!("\x1b[48;2;{}m", color)
  }
}
pub fn add_fg_color_with_reset(str: &String, color: &str, options: &Options) -> String {
  if !options.paint_colors || !options.paint_fg_colors {
    return format!("{}", str);
  }
  let open_tag = get_fg_open_html(options, color);
  return if options.html_mode {
    format!("{}{}</span>", open_tag, str)
  } else {
    format!("{}{}\x1b[0m", open_tag, str)
  }
}

pub fn add_bg_color_with_reset(str: &String, color: &str, options: &Options) -> String {
  if !options.paint_colors || !options.paint_bg_colors {
    return format!("{}", str);
  }

  let open_tag = get_bg_open_html(options, color);
  return if options.html_mode {
    format!("{}{}</span>", open_tag, str)
  } else {
    format!("{}{}\x1b[0m", open_tag, str)
  }
}

pub fn add_fg_color_with_reset_double(str: &String, color: &str, options: &Options) -> String {
  if !options.paint_colors || !options.paint_fg_colors {
    return format!("{}", str);
  }

  let open_tag = get_fg_open_html(options, color);
  return if options.html_mode {
    format!("{}{}{}</span>", open_tag, str, str)
  } else {
    format!("{}{}{}\x1b[0m", open_tag, str, str)
  }
}

pub fn add_bg_color_with_reset_double(str: &String, color: &str, options: &Options) -> String {
  if !options.paint_colors || !options.paint_bg_colors {
    return format!("{}", str);
  }

  let open_tag = get_bg_open_html(options, color);
  return if options.html_mode {
    format!("{}{}{}</span>", open_tag, str, str)
  } else {
    format!("{}{}{}\x1b[0m", open_tag, str, str)
  }
}
