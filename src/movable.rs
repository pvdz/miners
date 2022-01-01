use crate::slottable::SlotKind;
use super::miner::*;
use super::world::*;
use super::values::*;
// use super::icons::*;
use super::options::*;
use super::cell_contents::*;

pub struct Movable {
    pub what: i32,
    pub x: i32,
    pub y: i32,
    pub dir: i32,
    pub energy: f32,
}

fn drill_deeper(drills: i32, hammers: i32, x: i32, y: i32, dx: i32, dy: i32, world: &mut World, options: &Options) {
    // From where you're standing, move drill count+1 steps into dx and dy direction
    // For every block encountered decrease the drill count by one
    // For every block encountered passed the first, apply a bump of the drill count left
    // Respect the world wrapping around edges

    // Offset the first block. No action here, this is the one we already bumped
    let mut next_x = x + dx;
    let mut next_y = y + dy;
    let mut strength = if hammers > 0 { hammers - 1 } else { 0 }; // Start with the hammer strength - 1
    let mut remaining = drills; // Stop after punching through this many blocks

    // Now for each step and as long as there are drills and as long as the next step is a block
    while remaining > 0 && strength > 0 {

        ensure_cell_in_world(world, options, next_x, next_y);

        let mut unext_x = (world.min_x.abs() + next_x) as usize;
        let mut unext_y = (world.min_y.abs() + next_y) as usize;

        // Apply the drill power
        match world.tiles[unext_y][unext_x] {
            (Cell::Wall4, _) => {
                world.tiles[unext_y][unext_x] = match strength {
                    1 => create_tile(Cell::Wall3),
                    2 => create_tile(Cell::Wall2),
                    3 => create_tile(Cell::Wall1),
                    _ => {
                        remaining = 1;
                        create_tile(Cell::Diamond)
                    },
                };
            },
            (Cell::Wall3, _) => {
                world.tiles[unext_y][unext_x] = match strength {
                    1 => create_tile(Cell::Wall2),
                    2 => create_tile(Cell::Wall1),
                    _ => {
                        remaining = 1;
                        create_tile(Cell::Diamond)
                    },
                };
            },
            (Cell::Wall2, _) => {
                world.tiles[unext_y][unext_x] = match strength {
                    1 => create_tile(Cell::Wall1),
                    _ => {
                        remaining = 1;
                        create_tile(Cell::Diamond)
                    },
                };
            },
            (Cell::Wall1, _) => {
                world.tiles[unext_y][unext_x] = create_tile(Cell::Diamond); // Or a different powerup?
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

fn move_it_xy(movable: &mut Movable, meta: &mut MinerMeta, world: &mut World, options: &Options, nextx: i32, nexty: i32, deltax: i32, deltay: i32, nextdir: i32) {
    let mut was_boring = false; // Did we just move forward? No blocks, no pickups?
    if movable.what == WHAT_MINER {
        meta.points_last_move = 0;
    }

    // If this move would go OOB, expand the world to make sure that does not happen

    // println!("");
    // println!("world A: {:?}", world);
    ensure_cell_in_world(world, options, nextx, nexty);
    // println!("world B: {:?}", world);

    let mut unextx = (world.min_x.abs() + nextx) as usize;
    let mut unexty = (world.min_y.abs() + nexty) as usize;

    // println!("Stepping to: {}x{} ({}x{}) world is {}x{} - {}x{}", nextx, nexty, unextx, unexty, world.min_x, world.min_y, world.max_x, world.max_y);
    // println!("Actual world has {} lines and the first row has {} cols", world.tiles.len(), world.tiles[0].len());
    // println!("Wot? {} + {} = {} -> {}", world.min_y, nexty, world.min_y + nexty, unexty);

    if world.tiles.len() <= unexty { assert_eq!((unexty, "unexty"), (world.tiles.len(), "len"), "OOB: world is not high enough"); }
    if world.tiles[unexty].len() <= unextx { assert_eq!((unextx, "unextx"), (world.tiles[unexty].len(), "len"), "OOB: world is not wide enough"); }
    assert!(world.tiles.len() > unexty);
    assert!(unexty >= 0);
    assert!(world.tiles[unexty].len() > unextx);
    assert!(unextx >= 0);

    let drills = meta.kind_counts[SlotKind::Drill as usize];
    let hammers = meta.kind_counts[SlotKind::Hammer as usize];

    match world.tiles[unexty][unextx] {
        (Cell::Wall4, _) => {
            world.tiles[unexty][unextx] = match if movable.what == WHAT_MINER { hammers } else { 1 } {
                0 => create_tile(Cell::Wall3),
                1 => create_tile( Cell::Wall2 ),
                2 => create_tile( Cell::Wall1 ),
                _ => create_tile( Cell::Diamond ),
            };
            movable.dir = nextdir;
            movable.energy = movable.energy - meta.block_bump_cost;
            if movable.what == WHAT_MINER {
                if drills > 0 {
                    drill_deeper(drills, hammers, nextx, nexty, deltax, deltay, world, options);
                }
                meta.prev_move_bumped = true;
            }
        },
        (Cell::Wall3, _) => {
            world.tiles[unexty][unextx] = match if movable.what == WHAT_MINER { hammers } else { 1 } {
                0 => create_tile( Cell::Wall2 ),
                1 => create_tile( Cell::Wall1 ),
                _ => create_tile( Cell::Diamond ),
            };
            movable.dir = nextdir;
            movable.energy = movable.energy - meta.block_bump_cost;
            if movable.what == WHAT_MINER {
                if drills > 0 {
                    drill_deeper(drills, hammers, nextx, nexty, deltax, deltay, world, options);
                }
                meta.prev_move_bumped = true;
            }
        },
        (Cell::Wall2, _) => {
            world.tiles[unexty][unextx] = match if movable.what == WHAT_MINER { hammers } else { 1 } {
                0 => create_tile( Cell::Wall1 ),
                _ => create_tile( Cell::Diamond ),
            };
            movable.dir = nextdir;
            movable.energy = movable.energy - meta.block_bump_cost;
            if movable.what == WHAT_MINER {
                if drills > 0 {
                    drill_deeper(drills, hammers, nextx, nexty, deltax, deltay, world, options);
                }
                meta.prev_move_bumped = true;
            }
        },
        (Cell::Wall1, _) => {
            world.tiles[unexty][unextx] = create_tile( Cell::Diamond ); // Or a different powerup?
            movable.dir = nextdir; // Or maybe not? Could be a miner property or powerup
            movable.energy = movable.energy - meta.block_bump_cost;
            if movable.what == WHAT_MINER {
                if drills > 0 {
                    drill_deeper(drills, hammers, nextx, nexty, deltax, deltay, world, options);
                }
                meta.prev_move_bumped = true;
            }
        },
        (Cell::Energy, _) => {
            movable.energy = movable.energy + (E_VALUE as f64 * ((100.0 + meta.multiplier_energy_pickup as f64) / 100.0)) as f32;
            if movable.energy > meta.max_energy {
                movable.energy = meta.max_energy;
            }
            world.tiles[unexty][unextx] = create_tile( Cell::Empty );
            movable.x = nextx;
            movable.y = nexty;
        },
        (Cell::Diamond, value) => {
            // Different gems with different points. Miners could have properties or powerups to affect this, too.
            let gem_value = match value {
                0 => 1, // TODO: currently every tile has zero here :)
                1 => 2,
                2 => 3,
                3 => 4,
                _ => panic!("what value did this block have: {:?}", world.tiles[unexty][unextx]),
            };
            meta.points = meta.points + gem_value;
            if movable.what == WHAT_MINER {
                meta.points_last_move = gem_value;
            }
            world.tiles[unexty][unextx] = create_tile( Cell::Empty );
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
            movable.energy = movable.energy - meta.boredom_level as f32;
            // The cost grows the longer nothing keeps happening ("You're getting antsy, thirsty for an event")
            meta.boredom_level = meta.boredom_level + 1;
        } else {
            meta.boredom_level = 0;
        }
    }

    if movable.energy < 0.0 {
        movable.energy = 0.0;
    }
}

pub fn move_movable(movable: &mut Movable, meta: &mut MinerMeta, world: &mut World, options: &Options) {
    // println!("moving from {}x{}", movable.x, movable.y);
    match movable.dir {
        DIR_UP => {
            let nexty = movable.y - 1;
            move_it_xy(movable, meta, world, options, movable.x, nexty, 0, -1,DIR_LEFT);
        },
        DIR_LEFT => {
            let nextx = movable.x - 1;
            move_it_xy(movable, meta, world, options, nextx, movable.y, -1, 0, DIR_DOWN);
        },
        DIR_DOWN => {
            let nexty = movable.y + 1;
            move_it_xy(movable, meta, world, options, movable.x, nexty, 0, 1, DIR_RIGHT);
        },
        DIR_RIGHT => {
            let nextx = movable.x + 1;
            move_it_xy(movable, meta, world, options, nextx, movable.y, 1, 0, DIR_UP);
        },

        _ => {
            println!("unexpected dir is: {}", movable.dir);
            panic!("dir is enum");
        },
    }
}
