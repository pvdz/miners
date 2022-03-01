// This whole file is assumed to be #[cfg(target_arch = "wasm32")]
// This is the entry point for creating the wasm binary for web

// Simplify JSON de/serialization to/from JS
extern crate serde_json;

// This crate dumps panics to console.log in the browser
extern crate console_error_panic_hook;

// This is required to export panic to the web
use std::panic;

// Obviously, this is just to compile stuff to wasm.
use wasm_bindgen::prelude::*;

// temp
use std::collections::BTreeMap;

use super::main_loop::*;
use super::options::*;
use super::helix::*;
use super::biome::*;
use super::app_state::*;
use super::initialize::*;

#[wasm_bindgen]
extern {
  pub fn log(s: &str);
  pub fn print_world(s: &str);
  pub fn print_options(options: &str);
  pub fn dnow() -> u64;
  pub async fn await_next_frame() -> JsValue;
  pub async fn suspend_app_to_start() -> JsValue;
}

#[wasm_bindgen]
pub fn dbg(s: &str) {
  println!("{}", s);
  log(s);
}

// This is required to generate an export
#[wasm_bindgen]
pub async fn web_main() {
  log("Running sync main_web.rs.... :)");

  // This works to get a string
  // let str = match suspend_app_to_start().await.as_string() {
  //   Some(str) => str,
  //   None => "".to_string(),
  // };
  // log(format!("Bwtf? {}", str).as_str());

  let wat = suspend_app_to_start().await;
  let mut input_options: Options = match wat.into_serde() {
    Ok(json) => json,
    Err(e) => {
      log(format!("Hard crashing now. Unable to parse given value as an Options: {:?}", e).as_str());
      panic!("nope")
    },
  };
  let str = format!("Bwtf? {:?}", input_options);
  log(str.as_str());

  main_async(&mut input_options).await;
}

async fn main_async(options: &mut Options) {
  // Must run this once in web-mode to enable dumping panics to console.log
  panic::set_hook(Box::new(console_error_panic_hook::hook));

  log("Running async main.rs.... :)");

  let (mut state, mut next_root_helix, mut btree) = initialize(options);
  print_options(&serde_json::to_string_pretty(&options).unwrap());
  ga_loop_async(options, &mut state, &mut next_root_helix, &mut btree).await;
}

pub async fn ga_loop_async(options: &mut Options, state: &mut AppState, next_root_helix: &mut Helix, btree: &mut BTreeMap<String, (u64, usize, SerializedHelix)>) {
  loop {
    if options.visual || state.startup {
      print_options(&serde_json::to_string_pretty(&options).unwrap());
    }
    state.startup = false;
    *next_root_helix = ga_step_async(options, state, next_root_helix, btree).await;
  }
}

pub async fn ga_step_async(options: &mut Options, state: &mut AppState, curr_root_helix: &mut Helix, btree: &mut BTreeMap<String, (u64, usize, SerializedHelix)>) -> Helix {
  let mut biomes: Vec<Biome> = pre_ga_loop(options, state, curr_root_helix);

  let mut ticks = 0;

  while state.has_energy && !state.reset {
    if options.visual {
      suspend_app_till_next_frame(options, state, btree).await;
    } else {
      ticks += 1;
      if ticks > 1000 {
        // While wasm will be fast enough in the browser, it does run AND block in the main
        // thread. So we need to give it some breathing room every now and then to update the screen
        // The tick interval to do this at is arbitrary. The current setting works for me :shrug:
        ticks = 0;
        suspend_app_till_next_frame(options, state, btree).await;
      }
    }
    if state.load_best_as_miner_zero || state.reset {
      break;
    }
    go_iteration(options, state, &mut biomes, btree);
  }

  return post_ga_loop(options, state, biomes, curr_root_helix, btree);
}

pub async fn suspend_app_till_next_frame(options: &mut Options, state: &mut AppState, btree: &mut BTreeMap<String, (u64, usize, SerializedHelix)>) -> bool {
  let str = await_next_frame().await.as_string();

  return match str {
    Some(key) => {
      if key != "" {
        log(format!("Received input: {}", key.as_str()).as_str());
      }
      parse_input(key, options, state, btree)
    },
    None => false,
  };
}

pub fn platform_log(s: &str) {
  log(s);
}

pub fn platform_date_now() -> u64 {
  return dnow();
}

pub fn platform_print_world(table_str: &str) {
  print_world(table_str);
}
