use super::tile::*;
use super::pickup::*;

#[derive(Debug)]
pub struct Cell {
  pub tile: Tile, // Immovable type of this cell
  pub value: u32, // For certain kinds of cells this indicates its value
  pub pickup: Pickup, // What does the miner receive when moving onto this tile? (Potentially blocked by the tile, like wall)
  pub visited: u32, // How often has the miner visited this coord?
}
