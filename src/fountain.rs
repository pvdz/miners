use std::collections::HashSet;

use super::tile::*;
use super::biome::*;
use super::options::*;
use super::world::*;
use super::pickup::*;

#[derive(Debug)]
pub struct Fountain {
  pub x: i32,
  pub y: i32,
  pub water_tiles: HashSet<(i32, i32)>,
  pub ticks: u32,
  pub disabled: bool,
}

pub fn create_fountain(
  options: &mut Options, biome: &mut Biome
  // biome.miner.windrone.movable.x, biome.miner.windrone.movable.y, &mut biome.world, options
  // x: i32, y: i32, world: &World, options: &Options
) -> Fountain {
  let x = biome.miner.windrone.movable.x;
  let y = biome.miner.windrone.movable.y;
  let mut set: HashSet<(i32, i32)> = HashSet::new();
  collect_connected_water_coords(x, y, &mut biome.world, options, &mut set);

  return Fountain {
    x,
    y,
    water_tiles: set,
    ticks: 0,
    disabled: false,
  };
}

pub fn tick_fountain(fountain_index: usize, world: &mut World, options: &Options) {
  assert!(world.fountains.len() > fountain_index);

  if world.fountains[fountain_index].disabled { return; }
  if !matches!(get_cell_stuff_at(options, world, world.fountains[fountain_index].x, world.fountains[fountain_index].y).1, Pickup::Fountain) {
    world.fountains[fountain_index].disabled = true;
    return;
  }

  world.fountains[fountain_index].ticks += 1;

  if world.fountains[fountain_index].ticks > 100 {
    world.fountains[fountain_index].ticks = 0;
    // println!("its doing the thing!");
    // Find a tile that is not yet full and fill it. Otherwise ignore.
    for (wx, wy) in world.fountains[fountain_index].water_tiles.iter() {
      let stuff = get_cell_stuff_at(options, world, *wx, *wy);
      if matches!(stuff.1, Pickup::Nothing) {
        set_cell_pickup_at(options, world, *wx, *wy, Pickup::Water);
        break;
      }
    }
  }
}

fn collect_connected_water_coords(x: i32, y: i32, world: &World, options: &Options, set: &mut HashSet<(i32, i32)>) {
  /*
    // The tuple (x,y) should work in a set like this. I just tried it like this to confirm:

    let mut set: HashSet<(i32, i32)> = HashSet::new();

    let s = (1, 2);
    set.insert(s);

    let t = (1, 2);
    println!("has? {}", set.contains(&t));
    let r = (-1, 2);
    println!("has? {}", set.contains(&r));
    let q = (1, 2);
    println!("has? {}", set.contains(&q));
   */

  // The initial cell is empty but has a fountain. The other (targeted) cells are water and empty.
  if matches!(get_cell_tile_at(options, world, x, y), Tile::ExpandoWater) || matches!(get_cell_stuff_at(options, world, x, y).1, Pickup::Fountain){
    let xy = (x, y);
    if !set.contains(&xy) {
      set.insert(xy);

      // Since water tiles are not procedurally generates and we only care about water tiles
      // we should be able to ignore any tile that is not inside the already generated world.
      // Visit all cells in a cross if they are generated.
      for (tx, ty) in [(x -1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
        if tx >= world.min_x && tx <= world.max_x && ty >= world.min_y && ty <= world.max_y {
          collect_connected_water_coords(tx, ty, world, options, set);
        }
      }
    }
  };
}

// water -> hydrone
// wind -> windrone
// earth -> sandrone
// fire -> ignidrone
// buildrone
// woodrone
// redrone
// walldrone


