use std::fmt::Write;

use rand::prelude::*;
use rand_pcg::Pcg64;
use rand::distributions::{Distribution, Uniform};

use super::miner::*;
use super::values::*;
use super::options::*;
use super::helix::*;
use super::dome::*;

pub type Grid = [[char; HEIGHT]; WIDTH];

#[derive(Clone, Copy)]
pub struct World {
  pub max_points: i32,
  // Inanimate objects like blocks and pickups
  pub tiles: Grid,
  // Values of object at each tile, when applicable
  pub values: Grid,
}

pub fn generate_world(options: &Options) -> World {
    let mut map_rng = Pcg64::seed_from_u64(options.seed);

    let diex = Uniform::from(0..WIDTH);
    let diey = Uniform::from(0..HEIGHT);
    let ten = Uniform::from(0..10);

    // Generate the map for this run. We'll clone it for each cycle.
    let mut golden_map: World = World {
        max_points: 0,
        tiles: [[' '; WIDTH]; HEIGHT],
        values: [[' '; WIDTH]; HEIGHT],
    };

    let mut max_points = 0;

    // Add energy modules
    for _ in 0..E_COUNT {
        let x = diex.sample(&mut map_rng);
        let y = diey.sample(&mut map_rng);
        golden_map.tiles[x][y] = ICON_ENERGY;
    }

    // Add blocks
    for x in 0..WIDTH {
        for _n in 0..INIT_BLOCKS_PER_ROW {
            let y = diey.sample(&mut map_rng);
            // Do not erase energy modules
            if golden_map.tiles[x][y] == ' ' {
                golden_map.tiles[x][y] = '▓';
                golden_map.values[x][y] = match ten.sample(&mut map_rng) { 0 => '3', 1 | 2 => '2', _ => '1' };

                max_points = max_points + match golden_map.values[x][y] {
                    '1' => 1,
                    '2' => 1,
                    '3' => 1,
                    v => panic!("Unknown block value: {}", v),
                }
            }
        }
    }

    golden_map.max_points = max_points;

    return golden_map;
}

pub fn serialize_world(world: &World, domes: &[Dome; 20], best: (Helix, i32), options: &Options) -> String {
    let miner: &Miner = &domes[0].miner;

    // We assume a 150x80 terminal screen space (half my ultra wide)
    // We draw every cell twice because the terminal cells have a 1:2 w:h ratio

    // Clone the world tiles so we can print the moving entities on it
    // Otherwise for each cell we'd have to scan all the entities to check if they're on it
    // We could also construct an empty world with just the entities and check for non-zero instead
    let mut painting: Grid = world.tiles.clone();
    for dome in domes.iter() {
        paint(&dome.miner, &mut painting, ICON_GHOST);
    }
    paint(miner, &mut painting, ' ');
    for slot in miner.slots.iter() {
        slot.paint(&mut painting, world);
    }

    let mut buf : String = ICON_BORDER_TL.to_string();

    write!(buf, "{}", std::iter::repeat(ICON_BORDER_H).take(WIDTH*2).collect::<String>()).unwrap(); // cache this :shrug:
    write!(buf, "{} {: >100}", ICON_BORDER_TR, ' ').unwrap();
    write!(buf, "\n").unwrap();

    for y in 0..HEIGHT {
        write!(buf, "{}", ICON_BORDER_V).unwrap_or_else(|err| panic!("{:?}", err));
        for x in 0..WIDTH {
            let c: char = painting[x][y];
            match c {
                | ICON_ENERGY
                => write!(buf, "\x1b[33;1m{}\x1b[0m", c),
                | ICON_DIAMOND
                => match world.values[x][y] {
                    '1' => write!(buf, "\x1b[;1;1m{0}\x1b[0m", c),
                    '2' => write!(buf, "\x1b[;1;1m\x1b[32m{0}\x1b[0m", c),
                    '3' => write!(buf, "\x1b[;1;1m\x1b[34m{0}\x1b[0m", c),
                    _ => panic!("Unexpected world value: {}", c),
                },
                | ICON_TURN_RIGHT
                | ICON_INDEX_UP
                | ICON_INDEX_RIGHT
                | ICON_INDEX_LEFT
                | ICON_INDEX_DOWN
                | ICON_GHOST
                => write!(buf, "{}", c),

                | ICON_MINER_UP
                | ICON_MINER_RIGHT
                | ICON_MINER_DOWN
                | ICON_MINER_LEFT
                => write!(buf, "\x1b[;1;1m\x1b[31m{} \x1b[0m", c),

                | ICON_BLOCK_100
                | ICON_BLOCK_75
                | ICON_BLOCK_50
                | ICON_BLOCK_25
                => match world.values[x][y] {
                    '1' => write!(buf, "{0}{0}\x1b[0m", c),
                    '2' => write!(buf, "\x1b[32m{0}{0}\x1b[0m", c),
                    '3' => write!(buf, "\x1b[34m{0}{0}\x1b[0m", c),
                    _ => write!(buf, "\x1b[34m{0}{0}\x1b[0m", c),
                },

                v => write!(buf, "{0}{0}", v),
            }.unwrap_or_else(|err| panic!("{:?}", err));
        }
        write!(buf, "{}", ICON_BORDER_V).unwrap_or_else(|err| panic!("{:?}", err));

        const HEADER: usize = 13;
        match if y < HEADER { y } else { y - HEADER + 100 } {
            // Miner meta information
             0  => write!(buf, "  Miner:  {: <2}  x  {: <2} {: >60}\n", miner.movable.x, miner.movable.y, ' ').unwrap(),
             1  => write!(buf, "  Energy: {}{} ({: >3}%) {} / {} {: >60}\n",
                         std::iter::repeat('|').take(((miner.movable.energy as f32 / miner.meta.max_energy as f32) * 20.0) as usize).collect::<String>(),
                         std::iter::repeat('-').take(20 - ((miner.movable.energy as f64 / miner.meta.max_energy as f64) * 20.0) as usize).collect::<String>(),
                         ((miner.movable.energy as f64 / miner.meta.max_energy as f64) * 100.0) as i32,
                         miner.movable.energy,
                         miner.meta.max_energy,
                         ' '
             ).unwrap(),
             2  => write!(buf, "  Boredom: Rate: {: <2} per level. Level: {: <3}. Cost per step: {} {: >60}\n", miner.meta.boredom_rate as i32, miner.meta.boredom_level, (miner.meta.boredom_rate * miner.meta.boredom_level as f32) as i32, ' ').unwrap(),
             3  => write!(buf, "  Points: {} {: >60}\n", miner.meta.points, ' ').unwrap(),
             4  => write!(buf, "  Block bump cost: {} {: >60}\n", miner.meta.block_bump_cost, ' ').unwrap(),

             6  => write!(buf, "  Helix:                         Current:                Best:{: >60}\n", ' ').unwrap(),
             7  => write!(buf, "  Max energy:               {: >20} {: >20} {: >60}\n", miner.helix.multiplier_energy_start, best.0.multiplier_energy_start, ' ').unwrap(),
             8  => write!(buf, "  Multiplier points:        {: >20} {: >20} {: >60}\n", miner.helix.multiplier_points, best.0.multiplier_points, ' ').unwrap(),
             9  => write!(buf, "  Multiplier energy pickup: {: >20} {: >20} {: >60}\n", miner.meta.multiplier_energy_pickup, 0.0, ' ').unwrap(),
            10  => write!(buf, "  Block bump cost:          {: >20} {: >20} {: >60}\n", miner.helix.block_bump_cost, best.0.block_bump_cost, ' ').unwrap(),
            11  => write!(buf, "  Drone gen cooldown:       {: >20} {: >20} {: >60}\n", miner.helix.drone_gen_cooldown, best.0.drone_gen_cooldown, ' ').unwrap(),

            // The slots
            100  => write!(buf, "  Slots: {: >120}\n", ' ').unwrap(),
            101  => write!(buf, "    - {: <20} {}\n", miner.slots[0].title(), miner.slots[0]).unwrap(),
            102  => write!(buf, "    - {: <20} {}\n", miner.slots[1].title(), miner.slots[1]).unwrap(),
            103  => write!(buf, "    - {: <20} {}\n", miner.slots[2].title(), miner.slots[2]).unwrap(),
            104  => write!(buf, "    - {: <20} {}\n", miner.slots[3].title(), miner.slots[3]).unwrap(),
            105  => write!(buf, "    - {: <20} {}\n", miner.slots[4].title(), miner.slots[4]).unwrap(),
            106  => write!(buf, "    - {: <20} {}\n", miner.slots[5].title(), miner.slots[5]).unwrap(),
            107  => write!(buf, "    - {: <20} {}\n", miner.slots[6].title(), miner.slots[6]).unwrap(),
            108  => write!(buf, "    - {: <20} {}\n", miner.slots[7].title(), miner.slots[7]).unwrap(),
            109  => write!(buf, "    - {: <20} {}\n", miner.slots[8].title(), miner.slots[8]).unwrap(),
            110  => write!(buf, "    - {: <20} {}\n", miner.slots[9].title(), miner.slots[9]).unwrap(),
            111  => write!(buf, "    - {: <20} {}\n", miner.slots[10].title(), miner.slots[10]).unwrap(),
            112  => write!(buf, "    - {: <20} {}\n", miner.slots[11].title(), miner.slots[11]).unwrap(),
            113  => write!(buf, "    - {: <20} {}\n", miner.slots[12].title(), miner.slots[12]).unwrap(),
            114  => write!(buf, "    - {: <20} {}\n", miner.slots[13].title(), miner.slots[13]).unwrap(),
            115  => write!(buf, "    - {: <20} {}\n", miner.slots[14].title(), miner.slots[14]).unwrap(),
            116  => write!(buf, "    - {: <20} {}\n", miner.slots[15].title(), miner.slots[15]).unwrap(),
            117  => write!(buf, "    - {: <20} {}\n", miner.slots[16].title(), miner.slots[16]).unwrap(),
            118  => write!(buf, "    - {: <20} {}\n", miner.slots[17].title(), miner.slots[17]).unwrap(),
            119  => write!(buf, "    - {: <20} {}\n", miner.slots[18].title(), miner.slots[18]).unwrap(),
            120  => write!(buf, "    - {: <20} {}\n", miner.slots[19].title(), miner.slots[19]).unwrap(),
            121  => write!(buf, "    - {: <20} {}\n", miner.slots[20].title(), miner.slots[20]).unwrap(),
            122  => write!(buf, "    - {: <20} {}\n", miner.slots[21].title(), miner.slots[21]).unwrap(),
            123  => write!(buf, "    - {: <20} {}\n", miner.slots[22].title(), miner.slots[22]).unwrap(),
            124  => write!(buf, "    - {: <20} {}\n", miner.slots[23].title(), miner.slots[23]).unwrap(),
            125  => write!(buf, "    - {: <20} {}\n", miner.slots[24].title(), miner.slots[24]).unwrap(),
            126  => write!(buf, "    - {: <20} {}\n", miner.slots[25].title(), miner.slots[25]).unwrap(),
            127  => write!(buf, "    - {: <20} {}\n", miner.slots[26].title(), miner.slots[26]).unwrap(),
            128  => write!(buf, "    - {: <20} {}\n", miner.slots[27].title(), miner.slots[27]).unwrap(),
            129  => write!(buf, "    - {: <20} {}\n", miner.slots[28].title(), miner.slots[28]).unwrap(),
            130  => write!(buf, "    - {: <20} {}\n", miner.slots[29].title(), miner.slots[29]).unwrap(),
            131  => write!(buf, "    - {: <20} {}\n", miner.slots[30].title(), miner.slots[30]).unwrap(),
            132  => write!(buf, "    - {: <20} {}\n", miner.slots[31].title(), miner.slots[31]).unwrap(),


            133  => write!(buf, "{: <100}\n", ' ').unwrap(),
            134  => write!(buf, "{: <100}\n", ' ').unwrap(),
            135  => {
                let mut he : String = "".to_string();
                helix_to_string(&mut he, &best.0);
                write!(buf, "    Best {}{: <40}\n", he, ' ').unwrap();
            },
            136  => write!(buf, "    Seed: {} Speed: {} Gene rate: {} Slot rate: {} (+⏎/-⏎ to change speed, v⏎ to toggle visual mode) {: <100}\n", options.seed, options.speed, options.mutation_rate_genes, options.mutation_rate_slots, ' ').unwrap(),


            _ => {
                if y <= HEADER {
                    write!(buf, " {: >120}", ' ').unwrap();
                }
                write!(buf, "\n").unwrap()
            }
        }
    }

    // std::iter::repeat("X").take(10).collect::<String>()

    write!(buf, "{}", ICON_BORDER_BL).unwrap();
    write!(buf, "{}", std::iter::repeat(ICON_BORDER_H).take(WIDTH*2).collect::<String>()).unwrap();
    write!(buf, "{}", ICON_BORDER_BR).unwrap();

    buf
}
