use crate::{anchor_to_slot::slots::utils::Overlay, tooltip::items::has_level};
use ahash::AHashMap;
use parking_lot::RwLock;
use std::sync::LazyLock;

pub const UNMATCHED_UNCHANGED: f64 = 0.995;
// What a slot has to score as an icon. With the decoys in, true matches start at 0.946 and the
// closest other slot is 0.924. What a slot may also be keeps the old limit.
pub const SLOT_CONFIDENCE: f64 = 0.935;
// Slots change slowly and a tooltip can be gone in a few frames, so the slots are only looked
// at on every third scan. That keeps the average scan well under a frame at 30 frames a second.
pub const SLOT_UPDATE_FREQ: u64 = 3;
// ms of matching and number clean-up per look; slots left over are picked up by the next ones,
// so opening a page does not hold up the frames that follow it
pub const SLOT_BUDGET: f64 = 60.0;
// Something the game draws over an icon.

pub const LEVEL: usize = 0;
pub const OVERLAYS: [Overlay; 2] = [
    // The item level a chest asks for, a helmet and "1640" across the lower part. Above it is the
    // strip under the number, below it the last few rows.
    Overlay {
        area: (0.0, 18.0, 61.0, 33.0),
        gain: 0.02,
        on: has_level,
    },
    // the clock of an item that expires, in the bottom left corner; any item can have one
    Overlay {
        area: (0.0, 13.0, 20.0, 39.0),
        gain: 0.02,
        on: |_| true,
    },
];
// No overlay hides more of an icon than this (0.071 seen), so one that scores further under the
// limit is not tried with anything left out.
pub const OVERLAY_REACH: f64 = 0.1;
// Rows of the icon the shortlist goes by (1440p): the ones above the item level.
pub const AFAR_ROWS: f64 = 18.0;
// Only the icons that look most like the slot from afar are compared in full, and as many of the
// decoys: comparing all 340 icons takes 13 ms a slot against 0.7, and on the recordings recognises
// nothing more.
pub const SHORTLIST: usize = 16;
// Icons this close to the best one are kept as what the slot may also be: one plain chest scored
// 0.967, 0.963 and 0.957 against three different chests.
pub const ALTERNATIVE: f64 = 0.02;
pub const CELLS: usize = 6;
// each icon, and whether it is a decoy, as the mean colour of a few cells of its top rows, per resolution
pub static FROM_AFAR: LazyLock<RwLock<AHashMap<u32, Vec<(String, bool, Vec<[f32; 3]>)>>>> =
    LazyLock::new(|| RwLock::new(AHashMap::new()));
