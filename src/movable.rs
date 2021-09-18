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

fn drill_deeper(drills: i32, hammers: i32, x: usize, y: usize, dx: i32, dy: i32, world: &mut World) {
    // From where you're standing, move drill count+1 steps into dx and dy direction
    // For every block encountered decrease the drill count by one
    // For every block encountered passed the first, apply a bump of the drill count left
    // Respect the world wrapping around edges

    // Offset the first block. No action here, this is the one we already bumped
    let mut next_x = x as i32 + dx;
    let mut next_y = y as i32 + dy;
    let mut strength = if hammers > 0 { hammers - 1 } else { 0 }; // Start with the hammer strength - 1
    let mut remaining = drills; // Stop after punching through this many blocks

    // Now for each step and as long as there are drills and as long as the next step is a block
    while remaining > 0 && strength > 0 {

        if next_x < 0 { next_x = WIDTH as i32 - 1; }
        else if next_x >= WIDTH as i32 { next_x = 0; }
        if next_y < 0 { next_y = HEIGHT as i32 - 1; }
        else if next_y >= HEIGHT as i32 { next_y = 0; }

        let nx: usize = next_x as usize;
        let ny: usize = next_y as usize;

        // Apply the drill power
        match world[nx][ny] {
            ICON_BLOCK_100 => {
                world[nx][ny] = match strength {
                    1 => ICON_BLOCK_75,
                    2 => ICON_BLOCK_50,
                    3 => ICON_BLOCK_25,
                    _ => {
                        remaining = 1;
                        ICON_DIAMOND
                    },
                };
            },
            ICON_BLOCK_75 => {
                world[nx][ny] = match strength {
                    1 => ICON_BLOCK_50,
                    2 => ICON_BLOCK_25,
                    _ => {
                        remaining = 1;
                        ICON_DIAMOND
                    },
                };
            },
            ICON_BLOCK_50 => {
                world[nx][ny] = match strength {
                    1 => ICON_BLOCK_25,
                    _ => {
                        remaining = 1;
                        ICON_DIAMOND
                    },
                };
            },
            ICON_BLOCK_25 => {
                world[nx][ny] = ICON_DIAMOND; // Or a different powerup?
                remaining = 1;
            },
            _ => {
                remaining = 1;
            }
        }

        next_x = next_x + dx;
        next_y = next_y + dy;

        remaining = remaining - 1;
        strength = strength - 1;
    }

}

fn move_it_xy(movable: &mut Movable, meta: &mut MinerMeta, world: &mut World, nextx: usize, nexty: usize, deltax: i32, deltay: i32, nextdir: i32) {
    let mut was_boring = false; // Did we just move forward? No blocks, no pickups?
    match world[nextx][nexty] {
        ICON_BLOCK_100 => {
            world[nextx][nexty] = match if movable.what == WHAT_MINER { meta.hammers } else { 1 } {
                0 => ICON_BLOCK_75,
                1 => ICON_BLOCK_50,
                2 => ICON_BLOCK_25,
                _ => ICON_DIAMOND,
            };
            movable.dir = nextdir;
            movable.energy = movable.energy - meta.block_bump_cost;
            if movable.what == WHAT_MINER && meta.drills > 0 {
                drill_deeper(meta.drills, meta.hammers, nextx, nexty, deltax, deltay, world);
            }
        },
        ICON_BLOCK_75 => {
            world[nextx][nexty] = match if movable.what == WHAT_MINER { meta.hammers } else { 1 } {
                0 => ICON_BLOCK_50,
                1 => ICON_BLOCK_25,
                _ => ICON_DIAMOND,
            };
            movable.dir = nextdir;
            movable.energy = movable.energy - meta.block_bump_cost;
            if movable.what == WHAT_MINER && meta.drills > 0 {
                drill_deeper(meta.drills, meta.hammers, nextx, nexty, deltax, deltay, world);
            }
        },
        ICON_BLOCK_50 => {
            world[nextx][nexty] = match if movable.what == WHAT_MINER { meta.hammers } else { 1 } {
                0 => ICON_BLOCK_25,
                _ => ICON_DIAMOND,
            };
            movable.dir = nextdir;
            movable.energy = movable.energy - meta.block_bump_cost;
            if movable.what == WHAT_MINER && meta.drills > 0 {
                drill_deeper(meta.drills, meta.hammers, nextx, nexty, deltax, deltay, world);
            }
        },
        ICON_BLOCK_25 => {
            world[nextx][nexty] = ICON_DIAMOND; // Or a different powerup?
            movable.dir = nextdir; // Or maybe not? Could be a miner property or powerup
            movable.energy = movable.energy - meta.block_bump_cost;
            if movable.what == WHAT_MINER && meta.drills > 0 {
                drill_deeper(meta.drills, meta.hammers, nextx, nexty, deltax, deltay, world);
            }
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

    if movable.energy < 0 {
        movable.energy = 0;
    }
}

pub fn move_movable(movable: &mut Movable, meta: &mut MinerMeta, world: &mut World) {
    match movable.dir {
        DIR_UP => {
            let nexty: usize = if movable.y == 0 { HEIGHT - 1 } else { movable.y - 1 };
            move_it_xy(movable, meta, world, movable.x, nexty, 0, -1,DIR_LEFT);
        },
        DIR_LEFT => {
            let nextx = if movable.x == 0 { WIDTH - 1 } else { movable.x - 1 };
            move_it_xy(movable, meta, world, nextx, movable.y, -1, 0, DIR_DOWN);
        },
        DIR_DOWN => {
            let nexty = if movable.y == HEIGHT - 1 { 0 } else { movable.y + 1 };
            move_it_xy(movable, meta, world, movable.x, nexty, 0, 1, DIR_RIGHT);
        },
        DIR_RIGHT => {
            let nextx = if movable.x == WIDTH - 1 { 0 } else { movable.x + 1 };
            move_it_xy(movable, meta, world, nextx, movable.y, 1, 0, DIR_UP);
        },

        _ => {
            println!("unexpected dir is: {}", movable.dir);
            panic!("dir is enum");
        },
    }
}
