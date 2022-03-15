use super::slottable::*;

/*
 A magnet allows you to pick up pickups from additional neighboring cells.
 The forward cell, which you move into, is always collected. Then each additional
 magnet will collect pickups from one additional neighboring cell, up to 7 magnets.
 In order: forward, back, left, right, forward-left, forward-right, back-left, back-right

 Advantages:
 - More pickups (obviously)
 - Less likely to be bored

 Disadvantages:
 - Takes up a slot
 - TBD (energy?)

 The magnets are currently not charged so they're always active.
*/

pub const TITLE_MAGNET: &str = "Magnet";

pub fn create_slot_magnet(slot_index: usize, nth: i32) -> Slottable {
    return Slottable {
        kind: SlotKind::Magnet,
        slot: slot_index,
        title: TITLE_MAGNET.to_owned(),
        max_cooldown: 0.0,
        cur_cooldown: 0.0,
        nth,
        val: 0.0,
        sum: 0.0,
    };
}

pub fn ui_slot_magnet(slot: &Slottable) -> (String, String, String) {
    return (
        TITLE_MAGNET.to_string(),
        "".to_string(),
        match slot.nth {
            0 => format!("back tile; {} picks", slot.sum),
            1 => format!("left tile; {} picks", slot.sum),
            2 => format!("right tile; {} picks", slot.sum),
            3 => format!("forward-left tile; {} picks", slot.sum),
            4 => format!("forward-right tile; {} picks", slot.sum),
            5 => format!("backard-left tile; {} picks", slot.sum),
            6 => format!("backward-right tile; {} picks", slot.sum),
            _ => format!("unused; {} picks", slot.sum),
        }
    );
}
