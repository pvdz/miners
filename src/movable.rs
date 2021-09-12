use crate::miner::*;
use crate::world::*;
use crate::values::*;

pub struct Movable {
    pub what: i32,
    pub x: usize,
    pub y: usize,
    pub dir: i32,
    pub energy: i32,
}

fn move_it_xy(movable: &mut Movable, meta: &mut MinerMeta, world: &mut World, nextx: usize, nexty: usize, nextdir: i32) {
    let mut was_boring = false; // Did we just move forward? No blocks, no pickups?
    match world[nextx][nexty] {
        '█' => {
            world[nextx][nexty] = '▓';
            movable.dir = nextdir;
            movable.energy = movable.energy - meta.block_bump_cost;
        },
        '▓' => {
            world[nextx][nexty] = '▒';
            movable.dir = nextdir;
            movable.energy = movable.energy - meta.block_bump_cost;
        },
        '▒' => {
            world[nextx][nexty] = '░';
            movable.dir = nextdir;
            movable.energy = movable.energy - meta.block_bump_cost;
        },
        '░' => {
            world[nextx][nexty] = ICON_DIAMOND; // Or a different powerup?
            movable.dir = nextdir; // Or maybe not? Could be a miner property or powerup
            movable.energy = movable.energy - meta.block_bump_cost;
        },
        ICON_ENERGY => {
            movable.energy = movable.energy + (E_VALUE as f64 * ((100.0 + meta.multiplier_energy_pickup as f64) / 100.0)) as i32;
            if movable.energy > meta.max_energy {
                movable.energy = meta.max_energy;
            }
            world[nextx][nexty] = ' ';
            movable.x = nextx;
            movable.y = nexty;
        },
        ICON_DIAMOND => {
            meta.points = meta.points + 1; // Different gems with different points. Miners could have properties or powerups to affect this.
            world[nextx][nexty] = ' ';
            movable.x = nextx;
            movable.y = nexty;
        },
        _ => {
            movable.x = nextx;
            movable.y = nexty;
            was_boring = true;
        },
    }

    if movable.what == WHAT_MINER {
        if was_boring {
            // Prevent endless loops by making it increasingly more difficult to make consecutive moves that where nothing happens
            movable.energy = movable.energy - meta.boredom_level;
            // The cost grows the longer nothing keeps happening ("You're getting antsy, thirsty for an event")
            meta.boredom_level = meta.boredom_level + 1;
        } else {
            meta.boredom_level = 0;
        }
    }
}

pub fn move_movable(movable: &mut Movable, meta: &mut MinerMeta, world: &mut World) {
    match movable.dir {
        DIR_UP => {
            let nexty: usize = if movable.y == 0 { HEIGHT - 1 } else { movable.y - 1 };
            move_it_xy(movable, meta, world, movable.x, nexty, DIR_LEFT);
        },
        DIR_LEFT => {
            let nextx = if movable.x == 0 { WIDTH - 1 } else { movable.x - 1 };
            move_it_xy(movable, meta, world, nextx, movable.y, DIR_DOWN);
        },
        DIR_DOWN => {
            let nexty = if movable.y == HEIGHT - 1 { 0 } else { movable.y + 1 };
            move_it_xy(movable, meta, world, movable.x, nexty, DIR_RIGHT);
        },
        DIR_RIGHT => {
            let nextx = if movable.x == WIDTH - 1 { 0 } else { movable.x + 1 };
            move_it_xy(movable, meta, world, nextx, movable.y, DIR_UP);
        },

        _ => {
            println!("unexpected dir is: {}", movable.dir);
            panic!("dir is enum");
        },
    }
}
