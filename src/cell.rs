use super::tile::*;
use super::pickup::*;

#[derive(Debug)]
pub struct Cell {
  pub tile: Tile, // Immovable type of this cell
  pub tile_value: u32, // For certain kinds of tiles this indicates its value
  pub pickup: Pickup, // What does the miner receive when moving onto this tile? (Potentially blocked by the tile, like wall)
  pub pickup_value: u32, // For certain kinds of pickups this indicates its value
  pub visited: u32, // How often has the miner visited this coord?
}
