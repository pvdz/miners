use std::fmt;

use crate::slottable::*;
use crate::movable::*;
use crate::miner::*;
use crate::world::*;
use crate::values::*;
use crate::drone::*;

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

    fn paint(&self, painting: &mut Grid, world: &World) {
        if self.drone.movable.energy > 0 {
            painting[self.drone.movable.x][self.drone.movable.y] = match self.drone.movable.dir {
                DIR_UP => ICON_DRONE_UP,
                DIR_DOWN => ICON_DRONE_DOWN,
                DIR_LEFT => ICON_DRONE_LEFT,
                DIR_RIGHT => ICON_DRONE_RIGHT,
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
            self.drone.movable.dir = if miner_movable.dir == DIR_UP { DIR_DOWN } else { DIR_UP };
            miner_meta.drone_gen_cooldown = 50;
            miner_movable.energy = miner_movable.energy - 100;
        }
    }

    fn title(&self) -> &str { return TITLE_DRONE_LAUNCHER; }
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
