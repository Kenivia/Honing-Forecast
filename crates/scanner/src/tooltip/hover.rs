use super::{
    detect::{TitleBar, find_title},
    items::match_title,
    layout::parse_layout,
    title::{read_title, text_image},
};
use crate::{
    constants::COMBINED_NUMBER_HEIGHT,
    image_utils::{brightness::brightness_lut, ocr::recognize_line},
    scanner_state::{ScannerState, SlotAddress, Tradability},
};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

const SAME_HOVER: f64 = 6.0; // the tooltip is the same one while it stays within this many px
const MAX_MISSED: usize = 4;
const CENTRED_VOTE: usize = 100; // a title read with nothing over it outvotes any number of covered ones
const SETTLED: usize = 3; // agreeing reads after which the OCR stops for this hover
const SLOT_RIGHT_OF_TITLE: f64 = 10.67; // tooltip on the slot's right: gap from the slot's right edge to the title
const SLOT_LEFT_OF_TITLE: f64 = 16.0; // tooltip on the slot's left: gap from the title's right edge to the slot
const SLOT_TOLERANCE: f64 = 5.5;
const SLOT_ABOVE_TOOLTIP: f64 = 35.0; // a pushed-up tooltip can start a little below its slot's top

// one tooltip staying in place over consecutive frames; what it says is voted on across them
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Hover {
    pub position: (usize, usize),
    pub missed: usize,
    pub title_votes: AHashMap<String, usize>,
    pub amount_votes: AHashMap<String, usize>,
    pub tradability_votes: AHashMap<Tradability, usize>,

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

        let same = self.hover.as_ref().is_some_and(|hover| {
            (hover.position.0.abs_diff(bar.x) as f64) <= SAME_HOVER * s
                && (hover.position.1.abs_diff(bar.y) as f64) <= SAME_HOVER * s
        });
        let mut hover = self.hover.take().filter(|_| same).unwrap_or_default();
        hover.position = (bar.x, bar.y);
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
            let amount: String = recognize_line(&text_image(&buffer, x0, y0, x1, y1))
                .chars()
                .filter(char::is_ascii_digit)
                .collect();
            if !amount.is_empty() {
                *hover.amount_votes.entry(amount).or_default() += 1;
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
        self.hover = Some(hover);
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
        let active_pages = self.active_page_num();
        let tolerance = SLOT_TOLERANCE * s;
        let (bar_left, bar_right, bar_top) = (bar.x as f64, (bar.x + bar.width) as f64, bar.y as f64);

        let candidates: Vec<(SlotAddress, bool)> = self
            .slot_infos
            .iter()
            .filter(|(address, info)| {
                active_pages[&address.inventory_type] == Some(address.page_num)
                    && info.icon_name_score.as_ref().is_some_and(|x| &x.0 == icon)
            })
            .filter_map(|(address, _)| {
                let position = self.anchored_slot_address_position(address)?;
                let (left, top) = position.top_left;
                let top = top - COMBINED_NUMBER_HEIGHT * s;
                let in_column = (bar_left - SLOT_RIGHT_OF_TITLE * s - (left + position.width)).abs()
                    <= tolerance
                    || (bar_right + SLOT_LEFT_OF_TITLE * s - left).abs() <= tolerance;
                (in_column && top >= bar_top - SLOT_ABOVE_TOOLTIP * s)
                    .then_some((*address, (top - bar_top).abs() <= tolerance))
            })
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
