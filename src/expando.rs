use std::collections::HashSet;

use crate::tile::*;
use crate::options::*;
use crate::world::*;
use crate::pickup::*;

pub const MIN_EXPANDO_SIZE: u32 = 5;
pub const MAX_EXPANDO_SIZE: u32 = 15;

#[derive(Debug)]
pub struct Expando {
  // An "expando" is any source of fluid/gas that still has the capacity to expand.
  // The idea is that you open a well, spill, or cloud when removing a rock. The contents then
  // spills over the empty cells on the map. But the extend of the spill is limited.
  // The expando is an actor that permanently changes the world. Once it depletes it will
  // disappear and no longer make any further changes.
  // Its contents may be used as part of a crafting tree, or affect other actors, or ... smth.

  kind: ExpandoKind,

  pub x: i32,
  pub y: i32,

  // When disabled we should consider this expando not to exist.
  pub disabled: bool,

  // The volume basically tells us how far this expando can still spread
  // Once it reaches zero, it should be removed
  volume: u32,

  // Expansion rate in world ticks
  speed: u32,
  ticks_since_last_update: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum ExpandoKind {
  Water,
  Gas,
  Toxic,
  Oil,
  Lava,
}

pub fn create_expando(x: i32, y: i32, size: u32) -> Expando {
  return Expando {
    kind: ExpandoKind::Water,
    x,
    y,
    disabled: false,

    volume: size,
    speed: 100,
    ticks_since_last_update: 0,
  };
}

pub fn tick_expando(expando_index: usize, world: &mut World, options: &Options) {
  assert!(expando_index < world.expandos.len());

  let expando = &world.expandos[expando_index];

  if expando.disabled {
    return;
  }

  if !matches!(get_cell_tile_at(options, world, expando.x, expando.y), Tile::ExpandoWater) {
    // The tile is no longer an expando. Disable this expando.
    world.expandos[expando_index].disabled = true;
    return;
  }

  if expando.volume == 0 {
    // This expando is done expanding
    return;
  }

  if expando.ticks_since_last_update < expando.speed {
    world.expandos[expando_index].ticks_since_last_update += 1;
    return;
  }

  // Expandos only expand to empty cells (items and actors are ignored).
  // When expanding (that's now) they consider the neighbors in a cross (left/right/up/down).
  // A single expando can "feed" only a predetermined number of cells. Once it has, it can
  // be removed. My aim is to have a cheap hacky fluid dynamic.
  // When a neighbor cell is already the target type, repeat the action from that tile. Will
  // have to remember which cells were already visited to prevent an infinite loop.

  let x = expando.x;
  let y = expando.y;
  let volume = expando.volume;

  // Find all empty neighbour cells, recursively
  let mut set: HashSet<(i32, i32)> = HashSet::new();
  collect_empty_neighbors(x, y, expando_index, volume, world, options, &mut set);

  // We should now have all neighbouring cells that we might expand to.
  // Expand.
  let mut n = 0;
  for (wx, wy) in set {
    let ax = world.min_x.abs() + wx;
    let ay = world.min_y.abs() + wy;

    assert!(wx >= world.min_x);
    assert!(wy >= world.min_y);
    assert!(wx <= world.max_x);
    assert!(wy <= world.max_y);
    assert!(ax >= 0);
    assert!(ay >= 0);
    assert!(ax < (world.min_x.abs() + 1 + world.max_x));
    assert!(ay < (world.min_y.abs() + 1 + world.max_y));

    // Convert the tile to a water tile
    world.tiles[ay as usize][ax as usize].tile = Tile::ExpandoWater;
    // "swallow" whatever item is here.
    world.tiles[ay as usize][ax as usize].pickup = Pickup::Nothing;

    n += 1;
    if n > volume {
      // println!("Removing expando {} of {}", expando_index, world.expandos.len());
      // world.expandos.remove(expando_index);
      // Keep the expando but remove the remaining volume
      world.expandos[expando_index].volume = 0;
      break;
    }
  }

  world.expandos[expando_index].ticks_since_last_update = 0;
}

fn collect_empty_neighbors(x: i32, y: i32, expando_index: usize, expando_volume: u32, world: &mut World, options: &Options, set: &mut HashSet<(i32, i32)>) {
  if set.len() > expando_volume as usize {
    return;
  }

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

  match get_cell_tile_at(options, world, x, y) {
    Tile::Empty => {
      set.insert((x, y));
    },
    Tile::ExpandoWater => {
      let xy = (x, y);
      if !set.contains(&xy) {
        set.insert(xy);

        ensure_cell_in_world(world, options, x - 1, y - 1);
        ensure_cell_in_world(world, options, x + 1, y + 1);

        collect_empty_neighbors(x-1, y, expando_index, expando_volume, world, options, set);
        collect_empty_neighbors(x+1, y, expando_index, expando_volume, world, options, set);
        collect_empty_neighbors(x, y-1, expando_index, expando_volume, world, options, set);
        collect_empty_neighbors(x, y+1, expando_index, expando_volume, world, options, set);

      }
    }
    _ => {},
  };
}
