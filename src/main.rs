// Generate wasm build for web:
//   wasm-pack build --target web
// Generate and run in CLI:
//   RUST_BACKTRACE=1 cargo run -- --seed 210143 --batch-size 10 --miner '[210143,43.0,129.0,0.0,8.0,0.0,"..DDDDDDDDd.h.dd.EEE.EdEEP.EdPhh"]' --no-visual --batch-size 1

#[cfg(target_arch = "wasm32")]
pub mod main_web;

#[cfg(not(target_arch = "wasm32"))]
pub mod main_cli;

pub mod main_loop;
pub mod options;
pub mod drone_san;
pub mod fountain;
pub mod slottable;
pub mod color;
pub mod slot_windrone;
pub mod slot_sandrone;
pub mod drone_win;
pub mod world;
pub mod values;
pub mod icons;
pub mod drone_me;
pub mod movable;
pub mod cell;
pub mod miner;
pub mod helix;
pub mod inventory;
pub mod biome;
pub mod pickup;
pub mod slot_hammer;
pub mod slot_drill;
pub mod async_stdin;
pub mod slot_purity_scanner;
pub mod slot_broken_gps;
pub mod slot_drone_launcher;
pub mod slot_energy_cell;
pub mod slot_emptiness;
pub mod slot_jacks_compass;
pub mod tile;
pub mod utils;
pub mod expando;
pub mod bridge;
pub mod app_state;
pub mod initialize;

// Web has src/lib as entry point. This function won't be called there.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
  main_cli::main();
}
