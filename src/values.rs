pub const WIDTH: usize = 50;
pub const HEIGHT: usize = 50;

pub const INIT_BLOCKS_PER_ROW: i32 = WIDTH as i32 >> 1; // Half?

pub const DELAY_MS: u64 = 10;

pub const E_COUNT: i32 = 50; // How many energy pickups do we spawn
pub const E_VALUE: i32 = 125; // Energy pickup bonus. 5%?
pub const INIT_ENERGY: i32 = 5000;

// TODO: this must be typeable :)
pub const DIR_UP   : i32 = 1;
pub const DIR_RIGHT: i32 = 2;
pub const DIR_DOWN : i32 = 3;
pub const DIR_LEFT : i32 = 4;

pub const WHAT_MINER: i32 = 0;
pub const WHAT_DRONE: i32 = 1;

pub const TITLE_EMPTINESS: &str = "Empty";
pub const TITLE_DRONE_LAUNCHER: &str = "Drone Launcher";
pub const TITLE_ENERGY_CELL: &str = "Energy Cell";

// ░ ▒ ▓ █
pub const ICON_BORDER_TL: char = '╔';
pub const ICON_BORDER_BL: char = '╚';
pub const ICON_BORDER_TR: char = '╗';
pub const ICON_BORDER_BR: char = '╝';
pub const ICON_BORDER_V: char = '║';
pub const ICON_BORDER_H: char = '═';
pub const ICON_DIAMOND: char = '💎';
pub const ICON_ENERGY: char = '🔋';
pub const ICON_TURN_RIGHT: char = '🗘';
pub const ICON_HEAVY_UP: char = '🡅';
pub const ICON_HEAVY_RIGHT: char = '🡆';
pub const ICON_HEAVY_DOWN: char = '🡇';
pub const ICON_HEAVY_LEFT: char = '🡄';
pub const ICON_INDEX_UP: char = '👆';
pub const ICON_INDEX_RIGHT: char = '👉';
pub const ICON_INDEX_DOWN: char = '👇';
pub const ICON_INDEX_LEFT: char = '👈';

pub const ICON_MINER_UP: char = ICON_HEAVY_UP;
pub const ICON_MINER_RIGHT: char = ICON_HEAVY_RIGHT;
pub const ICON_MINER_DOWN: char = ICON_HEAVY_DOWN;
pub const ICON_MINER_LEFT: char = ICON_HEAVY_LEFT;

pub const ICON_DRONE_UP: char = ICON_INDEX_UP;
pub const ICON_DRONE_RIGHT: char = ICON_INDEX_RIGHT;
pub const ICON_DRONE_DOWN: char = ICON_INDEX_DOWN;
pub const ICON_DRONE_LEFT: char = ICON_INDEX_LEFT;
