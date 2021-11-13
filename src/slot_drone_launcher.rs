use std::fmt;

use super::slottable::*;
use super::movable::*;
use super::miner::*;
use super::world::*;
use super::values::*;
use super::drone::*;

pub const TITLE_DRONE_LAUNCHER: &str = "Drone Launcher";

pub struct DroneLauncher {
    pub drone: Drone,
}

impl Slottable for DroneLauncher {
    fn before_paint(&mut self, _miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World) {
        if self.drone.movable.energy > 0 {
            move_movable(&mut self.drone.movable, miner_meta, world);
        }
    }

    fn paint(&self, painting: &mut Grid, _world: &World) {
        if self.drone.movable.energy > 0 {
            painting[self.drone.movable.x][self.drone.movable.y] = match self.drone.movable.dir {
                DIR_UP => ICON_DRONE_UP,
                DIR_RIGHT => ICON_DRONE_RIGHT,
                DIR_DOWN => ICON_DRONE_DOWN,
                DIR_LEFT => ICON_DRONE_LEFT,
                _ => {
                    println!("unexpected dir: {:?}", self.drone.movable.dir);
                    panic!("dir is enum");
                },
            }
        }
    }

    fn after_paint(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, _world: &mut World) {
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
    }

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
