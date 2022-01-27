use super::slottable::*;

pub const TITLE_HAMMER: &str = "Hammer";

pub fn create_hammer(slot_index: usize, nth: i32) -> Slottable {
    return Slottable {
        kind: SlotKind::Hammer,
        slot: slot_index,
        title: TITLE_HAMMER.to_owned(),
        max_cooldown: 0.0,
        cur_cooldown: 0.0,
        nth,
        val: 0.0,
        sum: 0.0,
    };
}

pub fn ui_slot_hammer(_slot: &Slottable) -> (String, String, String) {
    return (
        TITLE_HAMMER.to_string(),
        "".to_string(),
        "".to_string()
    );
}

/*
pub struct Hammer {}

impl Slottable for Hammer {
    fn tick(&mut self, miner_movable: &mut Movable, miner_meta: &mut MinerMeta, world: &mut World, options: &Options) {}

    fn paint_entity(&self, world: &World, options: &Options) -> (Cell, i32, i32) { return (Cell::Empty, 0, 0); }
    fn paint_ui(&self, world: &World, options: &Options) -> Vec<char> { vec!() }
    fn paint_log(&self, world: &World, options: &Options) -> Vec<char> { vec!() }

    fn title(&self) -> &str { return TITLE_HAMMER; }

    fn to_symbol(&self) -> &str { return "H"; }


    fn get_cooldown(&self) -> f32 {
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

impl fmt::Display for Hammer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: >100}", ' ')
    }
}

*/
