use super::miner::*;
use super::world::*;
use super::helix::*;
use super::options::*;
use super::app_state::*;

pub struct Biome {
  pub ticks: u32,
  pub world: World,
  pub miner: Miner,

  // The real path this miner has taken in this world
  pub path: Vec<i32>,
}

pub fn generate_biomes(options: &mut Options, state: &mut AppState, curr_root_helix: &mut Helix) -> Vec<Biome> {
  // Generate a bunch of biomes. Create a world for them and put a miner in there.
  // Each biome shares the same world (governed by the seed). But since the world is destructible
  // we have to give each biome their own world state.
  let mut biomes: Vec<Biome> = vec!();
  for i in 0..options.batch_size {
    let cur_miner: Miner =
      if state.load_best_as_miner_zero {
        state.load_best_as_miner_zero = false;
        println!("loading best miner into biome {}... {}", i, curr_root_helix);
        create_miner_from_helix(curr_root_helix)
      } else {
        create_miner_from_helix(&mutate_helix(&mut state.instance_rng, curr_root_helix, &options)) // The helix will clone/copy. Can/should we prevent this?
      };
    let own_world: World = generate_world(&options);
    let biome = Biome {
      ticks: 0,
      world: own_world,
      miner: cur_miner,
      path: vec!(0, 0),
    };
    // println!("====== miner ======");
    // println!("miner slots: {:?}", &biome.miner.slots);
    // println!("===================");
    biomes.push(biome);
  }


  state.total_miner_count += biomes.len() as u32;
  state.current_miner_count += biomes.len() as u32;
  state.miner_count_since_last_best += biomes.len() as u32;

  return biomes;
}
