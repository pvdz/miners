// Should drones pick up anything at all? Or be specialized? (Drones that mine, drones that collect)
// Some kind of drone that builds a certain area? like a house or whatever? / special zone

// One range of items could be crafting items that focus on a particular tree. This may give
// players the ability to actively guide the tech tree exploration on a high level. Like perhaps
// an item could prefer to build towards the drone launcher while another could build towards
// a better purifier, or energy preservation or whatever.

// Do I want any kind of enemy npcs?
// Do I want any kind of interactable npcs? Active or passively interact.
// Passive interaction could be a preset of "when this do this" or smth. Or maybe item driven.

// Factorio belts? Running towards factories which convert mined stuff into other stuffs?

// Edibles (doesn't have to be real, eh) which affects hunger, which is non-lethal (unlike energy)
// but which affects other stats and effects?

// Fluid dynamics? Exploring water wells, toxic/oil spills, gas clouds, etc?

// Is fossils something to toy with?



use std::fmt::Write;

use std::fmt;
use rand_pcg::{Lcg128Xsl64};
use rand::distributions::{Distribution, Uniform};

use super::icons::*;
use super::miner::*;
use super::color::*;
use super::options::*;
use super::slottable::*;

#[derive(Debug)]
pub struct Inventory {
  pub stone_white: u32,
  pub stone_blue: u32,
  pub stone_green: u32,
  pub stone_yellow: u32,

  pub diamond_white: u32,
  pub diamond_blue: u32,
  pub diamond_green: u32,
  pub diamond_yellow: u32,

  pub energy: u32,
  pub wind: u32,
  pub wood: u32,
}

pub fn create_inventory() -> Inventory {
  return Inventory {
    stone_white: 0,
    stone_green: 0,
    stone_blue: 0,
    stone_yellow: 0,

    diamond_white: 0,
    diamond_green: 0,
    diamond_blue: 0,
    diamond_yellow: 0,

    energy: 0,
    wind: 0,
    wood: 0,
  };
}

pub fn clone_inventory(inventory: &Inventory) -> Inventory {
  return Inventory {
    stone_white: inventory.stone_white,
    stone_green: inventory.stone_green,
    stone_blue: inventory.stone_blue,
    stone_yellow: inventory.stone_yellow,

    diamond_white: inventory.diamond_white,
    diamond_green: inventory.diamond_green,
    diamond_blue: inventory.diamond_blue,
    diamond_yellow: inventory.diamond_yellow,

    energy: inventory.energy,
    wind: inventory.wind,
    wood: inventory.wood,
  };
}

pub fn ui_inventory(inventory: &Inventory) -> String {
  return format!(
    "{}: {: <5} {}: {: <5} {}: {: <5} {}: {: <5} {}: {: <5} {}: {: <5} {}: {: <5} {}: {: <5}  {}: {: <5}   {} : {: <5}  {}: {: <5} {: <50}",
    add_fg_color_with_reset(&ICON_STONE.to_string(), COLOR_LEVEL_1), inventory.stone_white,
    add_fg_color_with_reset(&ICON_STONE.to_string(), COLOR_LEVEL_2), inventory.stone_green,
    add_fg_color_with_reset(&ICON_STONE.to_string(), COLOR_LEVEL_3), inventory.stone_blue,
    add_fg_color_with_reset(&ICON_STONE.to_string(), COLOR_LEVEL_4), inventory.stone_yellow,

    add_fg_color_with_reset(&ICON_DIAMOND.to_string(), COLOR_LEVEL_1), inventory.diamond_white,
    add_fg_color_with_reset(&ICON_DIAMOND.to_string(), COLOR_LEVEL_2), inventory.diamond_green,
    add_fg_color_with_reset(&ICON_DIAMOND.to_string(), COLOR_LEVEL_3), inventory.diamond_blue,
    add_fg_color_with_reset(&ICON_DIAMOND.to_string(), COLOR_LEVEL_4), inventory.diamond_yellow,

    add_fg_color_with_reset(&ICON_ENERGY.to_string(), COLOR_ENERGY), inventory.energy,
    add_fg_color_with_reset(&ICON_BUILDER_POWER.to_string(), COLOR_WIND), inventory.wind,
    add_fg_color_with_reset(&ICON_WOOD.to_string(), COLOR_WOOD), inventory.wood,
    ' '
  );
}

pub fn get_points(inventory: &Inventory) -> u64 {
  return inventory.stone_white as u64 + (inventory.stone_blue as u64 * 5u64) + (inventory.stone_yellow as u64 * 10u64) + (inventory.stone_yellow as u64 * 50u64) + (inventory.diamond_white as u64 * 100u64) + (inventory.diamond_blue as u64 * 250u64) + (inventory.diamond_green as u64 * 500u64) + (inventory.diamond_yellow as u64 * 1000u64);
}
