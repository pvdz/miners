use super::values::*;
use super::options::*;
use super::world::*;
use super::movable::*;
use super::tile::*;
use super::biome::*;
use super::pickup::*;

pub struct MeDrone {
  // Each drone has its own x, y, direction, and energy
  pub movable: Movable,
}

pub fn tick_me_drone(options: &mut Options, biome: &mut Biome, _slot_index: usize, drone_index: usize) {
  // ticks: u32, drone: &mut MeDrone, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World, options: &mut Options

  move_drone(options, biome, drone_index);

  // Collect the current value as food / grass / plants / whatever
  if matches!(get_cell_tile_at(options, &biome.world, biome.miner.drones[drone_index].movable.x, biome.miner.drones[drone_index].movable.y), Tile::Soil) {
    // Reset the value.
    let soil_value = get_cell_tile_value_at(options, &biome.world, biome.miner.drones[drone_index].movable.x, biome.miner.drones[drone_index].movable.y);
    set_cell_tile_value_at(options, &mut biome.world, biome.miner.drones[drone_index].movable.x, biome.miner.drones[drone_index].movable.y, 0);
    if soil_value > 2 {
      biome.miner.meta.inventory.food += 1; // 1? Depends on state of soil and items, I guess.
      biome.miner.movable.now_energy = (biome.miner.movable.now_energy + 100.0).min(biome.miner.movable.init_energy); // ? TBD
    }
  }
}

fn move_drone(options: &mut Options, biome: &mut Biome, drone_index: usize) {
  let dx = biome.miner.drones[drone_index].movable.x;
  let dy = biome.miner.drones[drone_index].movable.y;
  let dir = biome.miner.drones[drone_index].movable.dir;
  let (deltax, deltay) = delta_forward(dir);
  let nextx = dx + deltax;
  let nexty = dy + deltay;

  // If this move would go OOB, expand the world to make sure that does not happen
  ensure_cell_in_world(&mut biome.world, options, nextx, nexty);

  let unextx = (biome.world.min_x.abs() + nextx) as usize;
  let unexty = (biome.world.min_y.abs() + nexty) as usize;

  // println!("Stepping to: {}x{} ({}x{}) world is {}x{} - {}x{}", nextx, nexty, unextx, unexty, biome.world.min_x, biome.world.min_y, biome.world.max_x, biome.world.max_y);
  // println!("Actual world has {} lines and the first row has {} cols", biome.world.tiles.len(), biome.world.tiles[0].len());
  // println!("Wot? {} + {} = {} -> {}", biome.world.min_y, nexty, biome.world.min_y + nexty, unexty);

  let tile = biome.world.tiles[unexty][unextx].tile;
  match tile {
    Tile::Wall4 => move_drone_bump_wall(options, biome, drone_index, 4, nextx, nexty, unextx, unexty),
    Tile::Wall3 => move_drone_bump_wall(options, biome, drone_index, 3, nextx, nexty, unextx, unexty),
    Tile::Wall2 => move_drone_bump_wall(options, biome, drone_index, 2, nextx, nexty, unextx, unexty),
    Tile::Wall1 => move_drone_bump_wall(options, biome, drone_index, 1, nextx, nexty, unextx, unexty),

    | Tile::Push
    | Tile::Impassible
    => {
      // Moving to a push tile or an impassible (dead end) tile. Must turn and try to make sure
      // not to send the movable into an infinite loop.

        let (tx, ty, _fill): (i32, i32, bool) = push_corner_move(options, &mut biome.world, biome.miner.drones[drone_index].movable.x, biome.miner.drones[drone_index].movable.y, deltax, deltay, false, false, dir);

        // We have the new delta xy for the turn. Act accordingly. If they're 0 flip-flop. The normal rule has a reasonable chance to loop so flip-flopping is more efficient.
        biome.miner.drones[drone_index].movable.dir = match (tx, ty) {
          (-1, 0) => Direction::Left,
          (1, 0) => Direction::Right,
          (0, 1) => Direction::Down,
          (0, -1) => Direction::Up,
          (0, 0) => {
            let v = get_cell_tile_value_at(options, &biome.world, biome.miner.drones[drone_index].movable.x, biome.miner.drones[drone_index].movable.y, );
            set_cell_tile_value_at(options, &mut biome.world, biome.miner.drones[drone_index].movable.x, biome.miner.drones[drone_index].movable.y, if v == 1 { 0 } else { 1 });

            match biome.miner.drones[drone_index].movable.dir {
              Direction::Up => if v == 1 { Direction::Left } else { Direction::Right },
              Direction::Right => if v == 1 { Direction::Up } else { Direction::Down },
              Direction::Down => if v == 1 { Direction::Right } else { Direction::Left },
              Direction::Left => if v == 1 { Direction::Down } else { Direction::Up },
            }
          },
          _ => panic!("This delta should not be possible {},{}", tx, ty),
        };
    }

    // The rest is considered an empty or at least passable tile

    | Tile::ExpandoWater
    | Tile::Empty
    | Tile::Fountain
    | Tile::Soil
    | Tile::ZeroZero
    | Tile::TenLine
    | Tile::HideWorld
    | Tile::Test2
    | Tile::Test3
      => {
      move_drone_pickup_from_empty_tile(options, biome, drone_index, unextx, unexty);
      biome.miner.drones[drone_index].movable.x = nextx;
      biome.miner.drones[drone_index].movable.y = nexty;
    },
  }
}

fn move_drone_pickup_from_empty_tile(options: &mut Options, biome: &mut Biome, drone_index: usize, unextx: usize, unexty: usize) {
  let cell = &mut biome.world.tiles[unexty][unextx];

  match cell.pickup {
    Pickup::Diamond => {
      // Different gems with different points.
      // Drones could have properties or powerups to affect this, too.
      match cell.pickup_value.min(3) {
        0 => biome.miner.meta.inventory.diamond_white += 1,
        1 => biome.miner.meta.inventory.diamond_green += 1,
        2 => biome.miner.meta.inventory.diamond_blue += 1,
        3 => biome.miner.meta.inventory.diamond_yellow += 1,
        _ => panic!("what value did this diamond have: {:?}", cell),
      };

      biome.world.tiles[unexty][unextx] = create_visited_cell(cell.tile, Pickup::Nothing, 0, 0, cell.visited + 1);
    },
    Pickup::Energy => {
      let drone = &mut biome.miner.drones[drone_index];
      // Who picks up the energy? The drone? The miner? Both? Items may determine this. ("drone modifications")
      drone.movable.now_energy = (drone.movable.now_energy + (E_VALUE as f64 * ((100.0 + biome.miner.meta.multiplier_energy_pickup as f64) / 100.0)) as f32).min(biome.miner.meta.max_energy);
      biome.miner.meta.inventory.energy += 1;
      biome.world.tiles[unexty][unextx] = create_visited_cell(cell.tile, Pickup::Nothing, 0, 0, cell.visited + 1);
    },
    Pickup::Stone => {
      // Do we have any purity scanners primed? Bump the value by that many.
      // Note: purity scanner only works for the miner itself. For drones, slots is empty
      match cell.pickup_value.min(3) {
        0 => biome.miner.meta.inventory.stone_white += 1,
        1 => biome.miner.meta.inventory.stone_green += 1,
        2 => biome.miner.meta.inventory.stone_blue += 1,
        3 => biome.miner.meta.inventory.stone_yellow += 1,
        _ => panic!("what value did this stone have: {:?}", cell),
      }
      biome.world.tiles[unexty][unextx] = create_visited_cell(cell.tile, Pickup::Nothing, 0, 0, cell.visited + 1);
    },
    Pickup::Wind => {
      biome.miner.meta.inventory.wind += 1;
      biome.world.tiles[unexty][unextx] = create_visited_cell(cell.tile, Pickup::Nothing, 0, 0, cell.visited + 1);
    },
    Pickup::Water => {
      biome.miner.meta.inventory.water += 1;
      biome.world.tiles[unexty][unextx] = create_visited_cell(cell.tile, Pickup::Nothing, 0, 0, cell.visited + 1);
    },
    Pickup::Wood => {
      biome.miner.meta.inventory.wood += 1;
      biome.world.tiles[unexty][unextx] = create_visited_cell(cell.tile, Pickup::Nothing, 0, 0, cell.visited + 1);
    },
    | Pickup::Nothing
    | Pickup::Expando // Ignore, fake pickup
    | Pickup::Fountain // Ignore, fake pickup... TODO: probably some special behavior?
    => {
      biome.world.tiles[unexty][unextx] = create_visited_cell(cell.tile, cell.pickup, cell.tile_value, cell.pickup_value, cell.visited + 1);
    },
  }
}

fn move_drone_bump_wall(
  options: &mut Options, biome: &mut Biome, drone_index: usize, strength: i32, nextx: i32, nexty: i32, unextx: usize, unexty: usize,
  //world: &mut World, options: &Options, movable: &mut Movable, hammers: i32, drills: i32, pickup: Pickup, tile_value: u32, pickup_value: u32, nextx: i32, nexty: i32, deltax: i32, deltay: i32, unextx: usize, unexty: usize, meta: &mut MinerMeta, _building_sandcastle: bool, _magic_min_x: i32, _magic_min_y: i32, _magic_max_x: i32, _magic_max_y: i32
) {
  let cell = &mut biome.world.tiles[unexty][unextx];
  let drone = &mut biome.miner.drones[drone_index];
  let n = strength - 1;

  // The pickup is set at tile generation time so we just need to clear the tile here
  biome.world.tiles[unexty][unextx] = match n.max(0) {
    3 => create_unvisited_cell(Tile::Wall3, cell.pickup, cell.tile_value, cell.pickup_value),
    2 => create_unvisited_cell(Tile::Wall2, cell.pickup, cell.tile_value, cell.pickup_value),
    1 => create_unvisited_cell(Tile::Wall1, cell.pickup, cell.tile_value, cell.pickup_value),
    0 => create_unvisited_cell(Tile::Empty, cell.pickup, cell.tile_value, cell.pickup_value),
    // always at least -1
    _ => panic!("A bump should always at least decrease the wall by one so it can never stay 4: {}", n),
  };

  if n <= 0 {
    // Broke a wall. Add sand.
    // TODO: what about the drill? What about bonuses? Should it be u32 or f32?
    biome.miner.meta.inventory.sand += 1;
  }

  // TODO: do drones have a different bump cost?
  drone.movable.now_energy = (drone.movable.now_energy - biome.miner.meta.block_bump_cost).max(0.0);
  // TODO: should drones use same "prefer visited tiles" heuristic as miner?
  drone.movable.dir = get_most_visited_dir_from_xydir(options, &biome.world, nextx, nexty, drone.movable.dir);
}
