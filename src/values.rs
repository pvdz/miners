pub const WIDTH: i32 = 50;
pub const HEIGHT: i32 = 50;

pub const INIT_BLOCKS_PER_ROW: i32 = WIDTH >> 1; // Half?

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
