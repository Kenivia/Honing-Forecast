use super::{
    chest::{Chest, ChestContent, ChestKind, ChestRow, read_chest},
    detect::{TitleBar, find_title},
    items::match_title,
    layout::parse_layout,
    title::{read_title, text_image, yellow_image},
};
use crate::{
    constants::{ALL_SLOT_ADDRESSS, COMBINED_NUMBER_HEIGHT},
    image_utils::{brightness::brightness_lut, ocr::recognize_line},
    scanner_state::{InventoryType, ScannerState, SlotAddress, Tradability},
};
use ahash::AHashMap;
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

const SAME_HOVER: f64 = 6.0; // the tooltip is the same one while it stays within this many px
const MAX_MISSED: usize = 4;
const TITLE_CHANGED: f64 = 0.1; // share of the title bar's columns that gained or lost text
const CENTRED_VOTE: usize = 100; // a title read with nothing over it outvotes any number of covered ones
const SETTLED: usize = 3; // agreeing reads after which the OCR stops for this hover
const SLOT_RIGHT_OF_TITLE: f64 = 10.67; // tooltip on the slot's right: gap from the slot's right edge to the title
const SLOT_LEFT_OF_TITLE: f64 = 16.0; // tooltip on the slot's left: gap from the title's right edge to the slot
const SLOT_TOLERANCE: f64 = 5.5;
const SLOT_ABOVE_TOOLTIP: f64 = 35.0; // a pushed-up tooltip can start a little below its slot's top
const LEVEL_TOLERANCE: f64 = 2.0;
const PUSHED_UP_MARGIN: f64 = 40.0;
const TITLE_BAR_HEIGHTS: [f64; 2] = [45.33, 69.33]; // one and two lines

// one tooltip staying in place over consecutive frames; what it says is voted on across them
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Hover {
    pub position: (usize, usize),
    // which columns of the title bar have text
    #[serde(default)]
    pub title_columns: Vec<bool>,
    pub missed: usize,
    pub title_votes: AHashMap<String, usize>,
    pub amount_votes: AHashMap<String, usize>,
    pub tradability_votes: AHashMap<Tradability, usize>,
    #[serde(default)]
    pub chest_kind_votes: AHashMap<ChestKind, usize>,
    #[serde(default)]
    pub chest_votes: Vec<(Vec<ChestContent>, usize)>,
    #[serde(default)]
    pub chest_rows: Vec<ChestRow>,
    // its entry in the scanner state's chests
    #[serde(default)]
    pub chest_index: Option<usize>,

    pub last_read_title: String,
    pub title: Option<String>,
    pub amount: Option<String>,
    pub tradability: Option<Tradability>,
    pub slot: Option<SlotAddress>,
}

fn winner<T: Clone + Eq + Hash>(votes: &AHashMap<T, usize>) -> Option<(T, usize)> {
    votes
        .iter()
        .max_by_key(|(_, count)| **count)
        .map(|(value, count)| (value.clone(), *count))
}

impl ScannerState {
    pub fn update_tooltip(&mut self) {
        let Some(brightness) = self.screen_info.brightness else {
            return;
        };
        let s = self.screen_info.scale_factor;
        self.buffer.lut = Some(brightness_lut(brightness));
        let buffer = self.buffer;
        let Some(bar) = find_title(&buffer, s) else {
            // tooltips fade in and single frames get lost, so a hover survives a few misses
            if let Some(hover) = &mut self.hover {
                hover.missed += 1;
                if hover.missed > MAX_MISSED {
                    self.hover = None;
                }
            }
            return;
        };

        // Tooltips pushed up by the screen bottom end up in the same place for every slot of a
        // column, so a tooltip that has not moved is only the same one while its title looks the same.
        let title_columns: Vec<bool> = (0..bar.width)
            .map(|x| {
                (0..bar.height).any(|y| buffer.rgb(bar.x + x, bar.y + y).into_iter().max().unwrap() > 120)
            })
            .collect();
        let same = self.hover.as_ref().is_some_and(|hover| {
            let changed = hover.title_columns.iter().zip(&title_columns).filter(|(a, b)| a != b).count();
            (hover.position.0.abs_diff(bar.x) as f64) <= SAME_HOVER * s
                && (hover.position.1.abs_diff(bar.y) as f64) <= SAME_HOVER * s
                && (changed as f64) < TITLE_CHANGED * bar.width as f64
        });
        let mut hover = self.hover.take().filter(|_| same).unwrap_or_default();
        hover.position = (bar.x, bar.y);
        hover.title_columns = title_columns;
        hover.missed = 0;

        if winner(&hover.title_votes).is_none_or(|(_, count)| count < SETTLED * CENTRED_VOTE) {
            let (read, centred) = read_title(&buffer, &bar, s);
            if let Some(title) = match_title(&read).and_then(|item| item.title.clone()) {
                *hover.title_votes.entry(title).or_default() +=
                    if centred { CENTRED_VOTE } else { 1 };
            }
            hover.last_read_title = read;
        }

        let layout = parse_layout(&buffer, &bar, s);
        if let Some(tradability) = layout.tradability {
            *hover.tradability_votes.entry(tradability).or_default() += 1;
        }
        if let Some((x0, y0, x1, y1)) = layout.amount
            && winner(&hover.amount_votes).is_none_or(|(_, count)| count < SETTLED)
        {
            // The recogniser doubles or drops a digit when the crop is a pixel wider or narrower, and
            // which crop does it differs from number to number. So the number is read off up to
            // three crops, one of them of the yellow alone, and two have to agree.
            let margin = |across: f64, down: f64| {
                let (across, down) = ((across * s).round() as usize, (down * s).round() as usize);
                (x0 - across, y0 - down, x1 + across, y1 + down)
            };
            let digits = |image: RgbaImage| -> String {
                recognize_line(&image).chars().filter(char::is_ascii_digit).collect()
            };
            let (a, b, c) = (margin(4.0, 5.33), margin(4.0, 4.0), margin(2.67, 2.67));
            let first = digits(text_image(&buffer, a.0, a.1, a.2, a.3));
            let second = digits(text_image(&buffer, b.0, b.1, b.2, b.3));
            // the third is only needed to break a tie
            let amount = if first == second {
                first
            } else {
                let third = digits(yellow_image(&buffer, c.0, c.1, c.2, c.3));
                [first, second].into_iter().find(|x| *x == third).unwrap_or_default()
            };
            if !amount.is_empty() {
                *hover.amount_votes.entry(amount).or_default() += 1;
            }
        }

        if let Some(chest) = &layout.chest {
            *hover.chest_kind_votes.entry(chest.kind).or_default() += 1;
            if hover.chest_votes.iter().all(|(_, count)| *count < SETTLED) {
                let (contents, rows) = read_chest(&buffer, chest, self.debugging);
                hover.chest_rows = rows;
                // a tooltip that is still fading in reads as nothing, which is not a vote for an empty chest
                if let Some(contents) = contents.filter(|x| !x.is_empty()) {
                    match hover.chest_votes.iter_mut().find(|(old, _)| *old == contents) {
                        Some((_, count)) => *count += 1,
                        None => hover.chest_votes.push((contents, 1)),
                    }
                }
            }
        }

        hover.title = winner(&hover.title_votes).map(|x| x.0);
        hover.amount = winner(&hover.amount_votes).map(|x| x.0);
        hover.tradability = winner(&hover.tradability_votes).map(|x| x.0);
        hover.slot = hover
            .title
            .as_ref()
            .and_then(|title| self.hovered_slot(&bar, s, title, hover.amount.as_ref()));
        if let Some(slot) = hover.slot.and_then(|slot| self.slot_infos.get_mut(&slot)) {
            slot.tradability = hover.tradability;
            slot.tooltip_amount = hover.amount.clone();
        }
        // the browser only gets a frame or two of a hover, so one read is enough to store the chest
        if let Some((contents, _)) = hover.chest_votes.iter().max_by_key(|(_, count)| *count) {
            // a tooltip that would run off the screen is pushed up until its hints end at the bottom
            let ui_bottom = self.screen_info.ui_origin.1 + self.screen_info.effective_height as f64;
            let pushed_up = layout.bottom as f64 >= ui_bottom - PUSHED_UP_MARGIN * s;
            let (column, slot) = self.tooltip_slot(&bar, s, pushed_up);
            let chest = Chest {
                kind: winner(&hover.chest_kind_votes).unwrap().0,
                contents: contents.clone(),
                amount: hover.amount.clone(),
                tradability: hover.tradability,
                last_read_title: hover.last_read_title.clone(),
                column,
                slot,
                rows: hover.chest_rows.clone(),
            };
            hover.chest_index = Some(self.store_chest(chest, hover.chest_index));
        }
        self.hover = Some(hover);
    }

    // slots on the active pages in the column the tooltip sits against, with their top
    fn slots_beside(&self, bar: &TitleBar, s: f64) -> Vec<(SlotAddress, f64)> {
        let active_pages = self.active_page_num();
        let tolerance = SLOT_TOLERANCE * s;
        let (bar_left, bar_right) = (bar.x as f64, (bar.x + bar.width) as f64);
        ALL_SLOT_ADDRESSS
            .keys()
            .filter(|address| active_pages[&address.inventory_type] == Some(address.page_num))
            .filter_map(|address| {
                let position = self.anchored_slot_address_position(address)?;
                let (left, top) = position.top_left;
                let in_column = (bar_left - SLOT_RIGHT_OF_TITLE * s - (left + position.width)).abs()
                    <= tolerance
                    || (bar_right + SLOT_LEFT_OF_TITLE * s - left).abs() <= tolerance;
                in_column.then_some((*address, top - COMBINED_NUMBER_HEIGHT * s))
            })
            .collect()
    }

    // For things only the tooltip identifies: the column it sits against, and the slot when it was
    // not pushed up, since it then shares the slot's top. The bar's top is found a few px low when
    // something is over it, its bottom is not, so the top is counted back from there.
    fn tooltip_slot(
        &self,
        bar: &TitleBar,
        s: f64,
        pushed_up: bool,
    ) -> (Option<(InventoryType, usize, usize)>, Option<SlotAddress>) {
        let height = TITLE_BAR_HEIGHTS
            .into_iter()
            .min_by(|a, b| (a * s - bar.height as f64).abs().total_cmp(&(b * s - bar.height as f64).abs()))
            .unwrap();
        let bar_top = (bar.y + bar.height) as f64 - height * s;
        let slots = self.slots_beside(bar, s);
        let column = slots
            .first()
            .map(|(x, _)| (x.inventory_type, x.page_num, x.pos_in_inv.1));
        let slot = slots
            .iter()
            .find(|(_, top)| !pushed_up && (top - bar_top).abs() <= LEVEL_TOLERANCE * s)
            .map(|x| x.0);
        (column, slot)
    }

    // The tooltip hugs the hovered slot's column on either side and shares its top, unless the screen
    // bottom pushed it up. The cursor hides the slot itself, so this goes by what the slots were last
    // seen to hold: the one in that column, level with or below the tooltip, with this title's icon.
    fn hovered_slot(
        &self,
        bar: &TitleBar,
        s: f64,
        title: &str,
        amount: Option<&String>,
    ) -> Option<SlotAddress> {
        let icon = match_title(title)?.icon.as_ref()?;
        let tolerance = SLOT_TOLERANCE * s;
        let bar_top = bar.y as f64;

        let candidates: Vec<(SlotAddress, bool)> = self
            .slots_beside(bar, s)
            .into_iter()
            .filter(|(address, top)| {
                *top >= bar_top - SLOT_ABOVE_TOOLTIP * s
                    && self.slot_infos.get(address).is_some_and(|info| {
                        info.icon_name_score.as_ref().is_some_and(|x| &x.0 == icon)
                    })
            })
            .map(|(address, top)| (address, (top - bar_top).abs() <= tolerance))
            .collect();
        match candidates.len() {
            1 => Some(candidates[0].0),
            _ => candidates.iter().find(|x| x.1).map(|x| x.0),
        }
        .or_else(|| {
            // several stacks of it in the column: the one whose own count matches the tooltip's
            let mut matching = candidates.iter().filter(|(address, _)| {
                amount.is_some_and(|amount| {
                    self.slot_infos[address].amount.as_ref().is_some_and(|x| x.trim() == amount)
                })
            });
            matching.next().filter(|_| matching.next().is_none()).map(|x| x.0)
        })
    }
}
