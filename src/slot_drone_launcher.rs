use std::fmt;
use crate::tile::Tile;

use super::slottable::*;
// use super::movable::*;
// use super::miner::*;
// use super::world::*;
// use super::values::*;
// use super::icons::*;
// use super::drone::*;
// use super::options::*;
// use super::cell_contents;

pub const TITLE_DRONE_LAUNCHER: &str = "Drone Launcher";

pub fn create_drone_launcher(nth: i32, drone_id: i32) -> Slottable {
    return Slottable {
        kind: SlotKind::DroneLauncher,
        title: TITLE_DRONE_LAUNCHER.to_owned(),
        max_cooldown: 0.0,
        cur_cooldown: 0.0,
        nth,
        val: drone_id,
        sum: 0,
    };
}

/*
pub struct DroneLauncher {
    // Each launcher has one drone
    pub drone: Drone,
}

struct Viewport {
    // ( min x, min y, max x, max y )

    // What tile of the world is showing in the viewport?
    world: (i32, i32, i32, i32),
    // Where is the viewport printed in output?
    output: (i32, i32, i32, i32),
}

impl Slottable for DroneLauncher {
    fn tick(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World, options: &Options) {
        if self.drone.movable.energy <= 0 && miner_meta.drone_gen_cooldown == 0 {
            self.drone.movable.energy = 100;
            self.drone.movable.x = miner_movable.x;
            self.drone.movable.y = miner_movable.y;
            self.drone.movable.dir = match miner_movable.dir {
                DIR_UP => DIR_RIGHT,
                DIR_RIGHT => DIR_DOWN,
                DIR_DOWN => DIR_LEFT,
                DIR_LEFT => DIR_UP,
                _ => panic!("Fix dir in drone_launcher::after_paint"),
            };
            miner_meta.drone_gen_cooldown = 50;
            miner_movable.energy = miner_movable.energy - 100;
        }

        if self.drone.movable.energy > 0 {
            move_movable(&mut self.drone.movable, miner_meta, world, options);
        }
    }

    fn paint_entity(&self, world: &World, options: &Options) -> (Cell, i32, i32) {
        // Returns the tile to paint and whether it is a double width icon
        if self.drone.movable.energy > 0 {
            let cell = match self.drone.movable.dir {
                DIR_UP => Cell::DroneUp,
                DIR_RIGHT => Cell::DroneRight,
                DIR_DOWN => Cell::DroneDown,
                DIR_LEFT => Cell::DroneLeft,
                _ => {
                    println!("unexpected dir: {:?}", self.drone.movable.dir);
                    panic!("dir is enum");
                },
            };

            return (cell, self.drone.movable.x, self.drone.movable.y);
        }

        // Do not paint
        return (Cell::Empty, 0, 0);
    }

    fn paint_ui(&self, world: &World, options: &Options) -> Vec<char> { vec!() }
    fn paint_log(&self, world: &World, options: &Options) -> Vec<char> { vec!() }

    fn title(&self) -> &str { return TITLE_DRONE_LAUNCHER; }

    fn to_symbol(&self) -> &str { return "D"; }


    fn get_cooldown(&self) -> f32 {
        // TODO: relocate this field from the miner (?)
        return 0.0;
    }

    fn set_cooldown(&mut self, _v: f32) -> f32 {
        return 0.0;
    }

    fn get_max_cooldown(&self) -> f32 {
        return 0.0;
    }

    fn set_max_cooldown(&mut self, _v: f32) -> f32 {
        return 0.0;
    }
}

impl fmt::Display for DroneLauncher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.drone.movable.energy <= 0 {
            write!(f, "Drone inactive {:>50}", ' ')
        } else {
            write!(f, "x: {: <2}, y: {: <2}, dir: {}, energy: {} {:>50}", self.drone.movable.x, self.drone.movable.y, self.drone.movable.dir, self.drone.movable.energy, ' ')
        }
    }
}
*/
