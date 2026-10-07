use super::{
    chest::{Chest, ChestContent, ChestKind, ChestRow, ChestRowStrips, chest_from_texts, chest_strips},
    detect::{TitleBar, find_title},
    icon::tooltip_icon,
    items::match_title,
    layout::parse_layout,
    title::{join_title, text_image, title_lines, yellow_image},
};
use crate::{
    constants::{ALL_SLOT_ADDRESSS, COMBINED_NUMBER_HEIGHT},
    image_utils::brightness::brightness_lut,
    scanner_state::{InventoryType, ScannerState, SlotAddress, Tradability},
    setup::OneIconConfig,
    timing::timed,
};
use ahash::AHashMap;
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::{hash::Hash, mem::take};

const SAME_HOVER: f64 = 6.0; // the tooltip is the same one while it stays within this many px
const MAX_MISSED: usize = 4;
const TITLE_CHANGED: f64 = 0.1; // share of the title bar's columns that gained or lost text
const CENTRED_VOTE: usize = 100; // a title read with nothing over it outvotes any number of covered ones
const SETTLED: usize = 3; // agreeing reads after which the OCR stops for this hover
const MAX_READS: usize = 6; // amount and chest reads one hover gets, agreeing or not
const LOOK_CHANGED: f64 = 4.0; // title columns that differ once the cursor has moved over it
// Reads of one kind a hover may have out at once. With one, a hover asks for its next read only
// when the last came back, so what it asks for follows what the OCR workers can do. Three were
// tried: twice the OCR work on the recordings and nothing more read.
const IN_FLIGHT: usize = 1;
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
    #[serde(skip)]
    pub id: u32,
    pub position: (usize, usize),
    // which columns of the title bar have text
    #[serde(default)]
    pub title_columns: Vec<bool>,
    pub missed: usize,
    pub title_votes: AHashMap<String, usize>,
    pub amount_votes: AHashMap<String, usize>,
    pub tradability_votes: AHashMap<Tradability, usize>,
    // the slot icon its large icon is
    #[serde(default)]
    pub icon_votes: AHashMap<String, usize>,
    #[serde(default)]
    pub chest_kind_votes: AHashMap<ChestKind, usize>,
    #[serde(default)]
    pub chest_votes: Vec<(Vec<ChestContent>, usize)>,
    #[serde(default)]
    pub chest_rows: Vec<ChestRow>,
    // its entry in the scanner state's chests
    #[serde(default)]
    pub chest_index: Option<usize>,

    // Taken off the frame the tooltip was last seen on, because its texts are read later, when
    // the page or the tooltip may be gone: the bar, the slots beside it, and where a chest goes.
    #[serde(skip)]
    pub bar: TitleBar,
    #[serde(skip)]
    pub candidates: Vec<(SlotAddress, f64)>,
    #[serde(skip)]
    pub chest_place: (Option<(InventoryType, usize, usize)>, Option<SlotAddress>),
    // title, amount and chest reads sent off and not answered yet, and sent in all
    #[serde(skip)]
    pub waiting: [usize; 3],
    #[serde(skip)]
    pub sent: [usize; 3],
    // the title's columns when it was last sent off
    #[serde(skip)]
    pub read_columns: Vec<bool>,
    // strips of the last frame it was seen on that could not go yet
    #[serde(skip)]
    pub held: Strips,

    pub last_read_title: String,
    pub title: Option<String>,
    // from the large icon, else the one the title is drawn with
    #[serde(default)]
    pub icon: Option<String>,
    pub amount: Option<String>,
    pub tradability: Option<Tradability>,
    pub slot: Option<SlotAddress>,
}

// the text strips of one frame of a hover, not sent off yet
#[derive(Debug, Default, Clone)]
pub struct Strips {
    // one per line, and whether every line was centred
    pub title: Option<(Vec<RgbaImage>, bool)>,
    pub amount: Option<[RgbaImage; 3]>,
    pub chest: Option<Vec<ChestRowStrips>>,
}

// The OCR jobs of one frame of a hover. Its votes are cast once every text is back.
#[derive(Debug, Default)]
pub struct PendingRead {
    pub hover: u32,
    // one job per line, and whether every line was centred
    pub title: Option<(Vec<u32>, bool)>,
    pub amount: Option<[u32; 3]>,
    // per row: the name's lines, the count, the debug crop
    pub chest: Option<Vec<(Vec<u32>, u32, Option<OneIconConfig>)>>,
}

impl PendingRead {
    fn jobs(&self) -> impl Iterator<Item = u32> + '_ {
        let title = self.title.iter().flat_map(|x| x.0.iter().copied());
        let amount = self.amount.iter().flatten().copied();
        let chest = self.chest.iter().flatten();
        let chest = chest.flat_map(|row| row.0.iter().copied().chain([row.1]));
        title.chain(amount).chain(chest)
    }
}

fn winner<T: Clone + Eq + Hash>(votes: &AHashMap<T, usize>) -> Option<(T, usize)> {
    votes
        .iter()
        .max_by_key(|(_, count)| **count)
        .map(|(value, count)| (value.clone(), *count))
}

fn most<T: Clone + Eq + Hash>(votes: &AHashMap<T, usize>) -> usize {
    winner(votes).map_or(0, |x| x.1)
}

impl ScannerState {
    // A hover that is over stays around until its last texts are in. What it held goes off now:
    // a short hover is over before its first read is back, and its last frame reads better than
    // its first, which is often still fading in.
    fn retire_hover(&mut self, mut hover: Hover) {
        let held = take(&mut hover.held);
        self.send_read(&mut hover, held);
        if hover.waiting != [0; 3] {
            self.past_hovers.push(hover);
        } else {
            self.write_hover(&mut hover, true);
        }
    }

    // sends strips off to be read, as one read of this hover
    fn send_read(&mut self, hover: &mut Hover, strips: Strips) {
        // a hover's first read goes ahead of other hovers' later ones
        let priority = hover.sent.into_iter().max().unwrap().min(255) as u8;
        let mut read = PendingRead { hover: hover.id, ..Default::default() };
        if let Some((lines, centred)) = strips.title {
            let jobs = lines.into_iter().map(|line| self.request_ocr(line, priority)).collect();
            read.title = Some((jobs, centred));
        }
        if let Some(images) = strips.amount {
            read.amount = Some(images.map(|image| self.request_ocr(image, priority)));
        }
        if let Some(rows) = strips.chest {
            let mut jobs = vec![];
            for row in rows {
                let names = row.names.into_iter().map(|x| self.request_ocr(x, priority)).collect();
                jobs.push((names, self.request_ocr(row.count, priority), row.crop));
            }
            read.chest = Some(jobs);
        }
        let kinds = [read.title.is_some(), read.amount.is_some(), read.chest.is_some()];
        for (kind, sent) in kinds.into_iter().enumerate() {
            hover.waiting[kind] += sent as usize;
            hover.sent[kind] += sent as usize;
        }
        if kinds.contains(&true) {
            self.pending_reads.push(read);
        }
    }

    // Finds the tooltip and sends its text lines off to be read. Nothing here waits for OCR:
    // the votes are cast in resolve_reads, when the texts are back.
    pub fn update_tooltip(&mut self) {
        let Some(brightness) = self.screen_info.brightness else {
            return;
        };
        let s = self.screen_info.scale_factor;
        self.buffer.lut = Some(brightness_lut(brightness));
        let buffer = self.buffer;
        let Some(bar) = timed("tooltip/find_title", || find_title(&buffer, s)) else {
            // tooltips fade in and single frames get lost, so a hover survives a few misses
            if let Some(mut hover) = self.hover.take() {
                hover.missed += 1;
                if hover.missed > MAX_MISSED {
                    self.retire_hover(hover);
                } else {
                    self.hover = Some(hover);
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
        let mut hover = match self.hover.take() {
            Some(hover) if same => hover,
            old => {
                if let Some(old) = old {
                    self.retire_hover(old);
                }
                self.hover_count += 1;
                Hover { id: self.hover_count, ..Default::default() }
            }
        };
        // A title that is not settled by reads with nothing over it is read again whenever it looks
        // different, which is the cursor moving. A few reads of the same look are enough.
        let look_changed = hover.read_columns.iter().zip(&title_columns).filter(|(a, b)| a != b).count()
            as f64
            > LOOK_CHANGED * s;
        hover.position = (bar.x, bar.y);
        hover.title_columns = title_columns;
        hover.missed = 0;
        hover.bar = bar;
        hover.candidates = self.slots_beside(&bar, s);

        // Each of the three is read until enough reads agree, counting those still on their way.
        // What cannot go yet, because a read of its kind is still out, is held instead.
        let mut strips = Strips::default();
        if most(&hover.title_votes) + hover.waiting[0] * CENTRED_VOTE < SETTLED * CENTRED_VOTE
            && (hover.sent[0] < SETTLED || look_changed)
        {
            strips.title = Some(timed("tooltip/title", || title_lines(&buffer, &bar, s)));
        }

        if most(&hover.icon_votes) < SETTLED
            && let Some(icon) = timed("tooltip/icon", || tooltip_icon(&buffer, &bar, s))
        {
            *hover.icon_votes.entry(icon).or_default() += 1;
        }

        let layout = timed("tooltip/layout", || parse_layout(&buffer, &bar, s));
        if let Some(tradability) = layout.tradability {
            *hover.tradability_votes.entry(tradability).or_default() += 1;
        }
        if let Some((x0, y0, x1, y1)) = layout.amount
            && most(&hover.amount_votes) + hover.waiting[1] < SETTLED
            && hover.sent[1] < MAX_READS
        {
            // The recogniser doubles or drops a digit when the crop is a pixel wider or narrower, and
            // which crop does it differs from number to number. So the number is read off three
            // crops, one of them of the yellow alone, and two have to agree.
            let margin = |across: f64, down: f64| {
                let (across, down) = ((across * s).round() as usize, (down * s).round() as usize);
                (x0 - across, y0 - down, x1 + across, y1 + down)
            };
            let (a, b, c) = (margin(4.0, 5.33), margin(4.0, 4.0), margin(2.67, 2.67));
            strips.amount = Some([
                text_image(&buffer, a.0, a.1, a.2, a.3),
                text_image(&buffer, b.0, b.1, b.2, b.3),
                yellow_image(&buffer, c.0, c.1, c.2, c.3),
            ]);
        }

        if let Some(chest) = &layout.chest {
            *hover.chest_kind_votes.entry(chest.kind).or_default() += 1;
            let agreed = hover.chest_votes.iter().map(|x| x.1).max().unwrap_or(0);
            if agreed + hover.waiting[2] < SETTLED && hover.sent[2] < MAX_READS {
                strips.chest =
                    Some(timed("tooltip/chest", || chest_strips(&buffer, chest, self.debugging)));
            }
        }
        let mut now = Strips::default();
        if hover.waiting[0] < IN_FLIGHT && strips.title.is_some() {
            now.title = strips.title.take();
            hover.read_columns = hover.title_columns.clone();
        }
        if hover.waiting[1] < IN_FLIGHT {
            now.amount = strips.amount.take();
        }
        if hover.waiting[2] < IN_FLIGHT {
            now.chest = strips.chest.take();
        }
        hover.held = strips;
        self.send_read(&mut hover, now);
        // a tooltip that would run off the screen is pushed up until its hints end at the bottom
        let ui_bottom = self.screen_info.ui_origin.1 + self.screen_info.effective_height as f64;
        let pushed_up = layout.bottom as f64 >= ui_bottom - PUSHED_UP_MARGIN * s;
        hover.chest_place = self.tooltip_slot(&bar, s, pushed_up);

        self.hover = Some(hover);
    }

    // Casts the votes of every read whose texts are all back, then writes what its hover now says
    // onto the hovered slot and the chest list.
    pub fn resolve_reads(&mut self) {
        let mut hovers: Vec<Hover> = take(&mut self.past_hovers);
        let current = self.hover.take().map(|hover| {
            hovers.push(hover);
            hovers.len() - 1
        });
        let mut touched = vec![false; hovers.len()];
        if let Some(index) = current {
            touched[index] = hovers[index].missed == 0;
        }

        for read in take(&mut self.pending_reads) {
            let Some(index) = hovers.iter().position(|hover| hover.id == read.hover) else {
                continue;
            };
            if !read.jobs().all(|id| self.ocr_texts.contains_key(&id)) {
                self.pending_reads.push(read);
                continue;
            }
            let hover = &mut hovers[index];
            touched[index] = true;
            let text = |id: &u32| self.ocr_texts[id].clone();

            if let Some((lines, centred)) = &read.title {
                let read = join_title(&lines.iter().map(text).collect::<Vec<_>>());
                if let Some(title) = match_title(&read).and_then(|item| item.title.clone()) {
                    *hover.title_votes.entry(title).or_default() +=
                        if *centred { CENTRED_VOTE } else { 1 };
                }
                hover.last_read_title = read;
                hover.waiting[0] -= 1;
            }
            if let Some(jobs) = &read.amount {
                let [first, second, third] = jobs
                    .each_ref()
                    .map(|id| text(id).chars().filter(char::is_ascii_digit).collect::<String>());
                // the third is only there to break a tie
                let amount = if first == second {
                    first
                } else {
                    [first, second].into_iter().find(|x| *x == third).unwrap_or_default()
                };
                if !amount.is_empty() {
                    *hover.amount_votes.entry(amount).or_default() += 1;
                }
                hover.waiting[1] -= 1;
            }
            if let Some(rows) = read.chest {
                let texts = rows
                    .into_iter()
                    .map(|(names, count, crop)| {
                        (names.iter().map(text).collect::<Vec<_>>().join(" "), text(&count), crop)
                    })
                    .collect();
                let (contents, rows) = chest_from_texts(texts);
                hover.chest_rows = rows;
                // a tooltip that is still fading in reads as nothing, which is not a vote for an empty chest
                if let Some(contents) = contents.filter(|x| !x.is_empty()) {
                    match hover.chest_votes.iter_mut().find(|(old, _)| *old == contents) {
                        Some((_, count)) => *count += 1,
                        None => hover.chest_votes.push((contents, 1)),
                    }
                }
                hover.waiting[2] -= 1;
            }
        }

        for (index, mut hover) in hovers.into_iter().enumerate() {
            if touched[index] {
                self.write_hover(&mut hover, Some(index) != current);
            }
            if Some(index) == current {
                self.hover = Some(hover);
            } else {
                self.retire_hover(hover);
            }
        }
    }

    // `over`: the hover has ended and nothing more of it will be read
    fn write_hover(&mut self, hover: &mut Hover, over: bool) {
        let s = self.screen_info.scale_factor;
        hover.title = winner(&hover.title_votes).map(|x| x.0);
        hover.amount = winner(&hover.amount_votes).map(|x| x.0);
        hover.tradability = winner(&hover.tradability_votes).map(|x| x.0);
        hover.icon = winner(&hover.icon_votes).map(|x| x.0).or_else(|| {
            hover.title.as_ref().and_then(|title| match_title(title)?.icon.clone())
        });
        hover.slot = hover.icon.as_ref().and_then(|icon| {
            self.hovered_slot(&hover.candidates, hover.bar.y as f64, s, icon, hover.amount.as_ref())
        });
        // a slot the user edited is left alone
        let slot = hover.slot.filter(|slot| !self.edits.contains_key(slot));
        if let Some(slot) = slot.and_then(|slot| self.slot_infos.get_mut(&slot)) {
            slot.hovered = true;
            slot.tradability = hover.tradability;
            slot.tooltip_amount = hover.amount.clone();
            // no amount line was found, or no two crops of it ever agreed
            slot.tooltip_failed = hover.amount.is_none()
                && hover.waiting[1] == 0
                && (over || hover.sent[1] >= MAX_READS);
        }
        // a hover is often only seen for a frame or two, so one read is enough to store the chest
        if let Some((contents, _)) = hover.chest_votes.iter().max_by_key(|(_, count)| *count) {
            let chest = Chest {
                kind: winner(&hover.chest_kind_votes).unwrap().0,
                contents: contents.clone(),
                amount: hover.amount.clone(),
                tradability: hover.tradability,
                last_read_title: hover.last_read_title.clone(),
                icon: hover.icon.clone(),
                title: hover.title.clone(),
                column: hover.chest_place.0,
                slot: hover.chest_place.1,
                rows: hover.chest_rows.clone(),
            };
            hover.chest_index = Some(self.store_chest(chest, hover.chest_index));
        }
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
    // seen to hold: the one in that column, level with or below the tooltip, with this icon.
    fn hovered_slot(
        &self,
        beside: &[(SlotAddress, f64)],
        bar_top: f64,
        s: f64,
        icon: &str,
        amount: Option<&String>,
    ) -> Option<SlotAddress> {
        let tolerance = SLOT_TOLERANCE * s;

        let candidates: Vec<(SlotAddress, bool)> = beside
            .iter()
            .copied()
            .filter(|(address, top)| {
                *top >= bar_top - SLOT_ABOVE_TOOLTIP * s
                    && self.slot_infos.get(address).is_some_and(|info| {
                        info.icon_name_score.as_ref().is_some_and(|x| x.0 == icon)
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
