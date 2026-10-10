use super::{
    chest::{Chest, ChestKind, ChestRow, ChestRowStrips, chest_strips, chests_listing},
    detect::{TitleBar, find_title},
    icon::tooltip_icon,
    items::{ITEMS, item_from_body, match_title, shares_icon, variant},
    layout::parse_layout,
    title::{Ink, join_title, text_image, title_lines, yellow_image},
};
use crate::{
    constants::{COMBINED_NUMBER_HEIGHT, SLOT_ORDER},
    image_utils::brightness::brightness_lut,
    scanner_state::{InventoryType, ScannerState, SlotAddress, Tradability},
    timing::timed,
};
use ahash::AHashMap;
use std::{hash::Hash, mem::take};

const SAME_HOVER: f64 = 6.0; // the tooltip is the same one while it stays within this many px
const MAX_MISSED: usize = 4;
const TITLE_CHANGED: f64 = 0.1; // share of the title bar's columns that gained or lost text
const CENTRED_VOTE: usize = 100; // a title read with nothing over it outvotes any number of covered ones
const SETTLED: usize = 3; // agreeing reads after which the OCR stops for this hover
const MAX_READS: usize = 6; // amount, chest and icon reads one hover gets, agreeing or not
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
// From a window's origin to the right end of its header, measured on a 1440p recording, and the
// rows above and below the origin its header is on.
const WINDOW_WIDTHS: [(InventoryType, f64); 3] = [
    (InventoryType::Roster, 442.4),
    (InventoryType::CharStorage, 723.3),
    (InventoryType::CharInventory, 722.4),
];
const HEADER_ROWS: (f64, f64) = (10.0, 60.0);
// Where a title bar is looked for. A tooltip sits against a slot's column, level with the slot or
// pushed up, so its bar is within a title's width of a window and not below the last row of slots.
const WINDOW_REACH: f64 = 440.0;
const WINDOW_ROWS: f64 = 800.0;
// the last hover's place is tried first, with this much around it
const NEAR: f64 = 8.0;

// one tooltip staying in place over consecutive frames; what it says is voted on across them
#[derive(Debug, Default, Clone)]
pub struct Hover {
    pub id: u32,
    pub position: (usize, usize),
    // which columns of the title bar have text
    pub title_columns: Vec<bool>,
    pub missed: usize,
    pub title_votes: AHashMap<String, usize>,
    pub amount_votes: AHashMap<String, usize>,
    pub tradability_votes: AHashMap<Tradability, usize>,
    // the slot icon its large icon is, and how often that was looked at
    pub icon_votes: AHashMap<String, usize>,
    pub icon_tries: usize,
    // its large icon is one several materials are drawn with, so its description is read too
    pub shared_icon: bool,
    // the material its description says it is
    pub body_votes: AHashMap<String, usize>,
    pub chest_kind_votes: AHashMap<ChestKind, usize>,
    // by id, for the chests of templates/chests.json its rows read as
    pub chest_votes: AHashMap<u32, usize>,
    pub chest_rows: Vec<ChestRow>,
    // its entry in the scanner state's chests
    pub chest_index: Option<usize>,

    // Taken off the frame the tooltip was last seen on, because its texts are read later, when
    // the page or the tooltip may be gone: the bar, the slots beside it, and where a chest goes.
    pub bar: TitleBar,
    pub candidates: Vec<(SlotAddress, f64)>,
    pub chest_place: (Option<(InventoryType, usize, usize)>, Option<SlotAddress>),
    // title, amount, chest and description reads sent off and not answered yet, and sent in all
    pub waiting: [usize; 4],
    pub sent: [usize; 4],
    // the title's columns when it was last sent off
    pub read_columns: Vec<bool>,
    // strips of the last frame it was seen on that could not go yet
    pub held: Strips,

    pub last_read_title: String,
    pub title: Option<String>,
    // from the large icon, else the one the title is drawn with
    pub icon: Option<String>,
    pub amount: Option<String>,
    pub tradability: Option<Tradability>,
    // the material, from the description, else the title
    pub label: Option<String>,
    pub slot: Option<SlotAddress>,
}

// the text lines of one frame of a hover, not sent off yet
#[derive(Debug, Default, Clone)]
pub struct Strips {
    // one per line, and whether every line was centred
    pub title: Option<(Vec<Ink>, bool)>,
    pub amount: Option<[Ink; 3]>,
    pub chest: Option<Vec<ChestRowStrips>>,
    // one per description line
    pub body: Option<Vec<Ink>>,
}

// The OCR jobs of one frame of a hover. Its votes are cast once every text is back.
#[derive(Debug, Default)]
pub struct PendingRead {
    pub hover: u32,
    // one job per line, and whether every line was centred
    pub title: Option<(Vec<u32>, bool)>,
    pub amount: Option<[u32; 3]>,
    // per row: the name's lines, the count
    pub chest: Option<Vec<(Vec<u32>, u32)>>,
    pub body: Option<Vec<u32>>,
}

impl PendingRead {
    fn jobs(&self) -> impl Iterator<Item = u32> + '_ {
        let title = self.title.iter().flat_map(|x| x.0.iter().copied());
        let amount = self.amount.iter().flatten().copied();
        let chest = self.chest.iter().flatten();
        let chest = chest.flat_map(|row| row.0.iter().copied().chain([row.1]));
        title.chain(amount).chain(chest).chain(self.body.iter().flatten().copied())
    }
}

// a tie goes to the smaller value, not by map order, so that runs repeat
fn winner<T: Clone + Ord + Hash>(votes: &AHashMap<T, usize>) -> Option<(T, usize)> {
    votes
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(value, count)| (value.clone(), *count))
}

fn most<T: Clone + Ord + Hash>(votes: &AHashMap<T, usize>) -> usize {
    winner(votes).map_or(0, |x| x.1)
}

impl ScannerState {
    // A hover that is over stays around until its last texts are in. What it held goes off now:
    // a short hover is over before its first read is back, and its last frame reads better than
    // its first, which is often still fading in.
    fn retire_hover(&mut self, mut hover: Hover) {
        let held = take(&mut hover.held);
        self.send_read(&mut hover, held);
        if hover.waiting != [0; 4] {
            self.past_hovers.push(hover);
        } else {
            self.write_hover(&mut hover, true);
        }
    }

    // sends lines off to be read, as one read of this hover
    fn send_read(&mut self, hover: &mut Hover, strips: Strips) {
        let start = crate::timing::now();
        // a hover's first read goes ahead of other hovers' later ones
        let priority = hover.sent.into_iter().max().unwrap().min(255) as u8;
        let mut read = PendingRead { hover: hover.id, ..Default::default() };
        if let Some((lines, centred)) = strips.title {
            let jobs = lines.into_iter().map(|line| self.request_ocr(line.strip(), priority, false)).collect();
            read.title = Some((jobs, centred));
        }
        if let Some(images) = strips.amount {
            read.amount = Some(images.map(|image| self.request_ocr(image.strip(), priority, false)));
        }
        if let Some(rows) = strips.chest {
            let mut jobs = vec![];
            for row in rows {
                let names = row.names.into_iter().map(|x| self.request_ocr(x.strip(), priority, false)).collect();
                jobs.push((names, self.request_ocr(row.count.strip(), priority, false)));
            }
            read.chest = Some(jobs);
        }
        if let Some(lines) = strips.body {
            read.body = Some(lines.into_iter().map(|line| self.request_ocr(line.strip(), priority, false)).collect());
        }
        let kinds =
            [read.title.is_some(), read.amount.is_some(), read.chest.is_some(), read.body.is_some()];
        for (kind, sent) in kinds.into_iter().enumerate() {
            hover.waiting[kind] += sent as usize;
            hover.sent[kind] += sent as usize;
        }
        if kinds.contains(&true) {
            self.pending_reads.push(read);
            crate::timing::record("tooltip/strips", crate::timing::now() - start);
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
        // each located window: its origin and its header's right edge
        let located: Vec<((f64, f64), f64)> = WINDOW_WIDTHS
            .iter()
            .filter_map(|(inventory, width)| {
                let root = self.inventory_root(*inventory)?;
                Some((root, root.0 + width * s))
            })
            .collect();
        // each header's right edge, and the rows a bar found in it starts on
        let headers: Vec<(f64, f64, f64)> = located
            .iter()
            .map(|((_, y), edge)| (*edge, y - HEADER_ROWS.0 * s, y + HEADER_ROWS.1 * s))
            .collect();
        let last = self.hover.as_ref().map(|hover| hover.bar);
        let Some(bar) = timed("tooltip/find_title", || {
            // no window, no slot to hover
            if headers.is_empty() {
                return None;
            }
            let (left, right, bottom) = located.iter().fold(
                (f64::MAX, 0f64, 0f64),
                |(left, right, bottom), ((x, y), edge)| {
                    (left.min(*x), right.max(*edge), bottom.max(y + WINDOW_ROWS * s))
                },
            );
            let reach = WINDOW_REACH * s;
            let windows = ((left - reach).max(0.0) as usize, 0, (right + reach) as usize, bottom as usize);
            let near = (NEAR * s).ceil() as usize;
            let mut regions = vec![];
            if let Some(bar) = last {
                // all of the bar's rows: which column most of them end on is where it is put
                regions.push((
                    bar.x.saturating_sub(near),
                    bar.y.saturating_sub(near),
                    bar.x + bar.width + near,
                    bar.y + bar.height + near,
                ));
            }
            regions.push(windows);
            regions.into_iter().find_map(|region| find_title(&buffer, s, &headers, region))
        }) else {
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

        let layout = timed("tooltip/layout", || parse_layout(&buffer, &bar, s));
        // A chest's icon is no material's, and a book's never leads the other book's, so both
        // would be compared on every frame for nothing.
        if layout.chest.is_none() && most(&hover.icon_votes) < SETTLED && hover.icon_tries < MAX_READS {
            hover.icon_tries += 1;
            if let Some((icon, clear)) = timed("tooltip/icon", || tooltip_icon(&buffer, &bar, s)) {
                hover.shared_icon |= shares_icon(&icon);
                if clear {
                    *hover.icon_votes.entry(icon).or_default() += 1;
                }
            }
        }
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
            if most(&hover.chest_votes) + hover.waiting[2] < SETTLED && hover.sent[2] < MAX_READS {
                strips.chest =
                    Some(timed("tooltip/chest", || chest_strips(&buffer, chest)));
            }
        }
        // which book a book is: its description says so more often than its title, whose start
        // the cursor covers when the tooltip is level with the slot
        if hover.shared_icon
            && !layout.description.is_empty()
            && most(&hover.body_votes) + hover.waiting[3] < SETTLED
            && hover.sent[3] < MAX_READS
        {
            let lines = layout.description.iter();
            strips.body = Some(lines.map(|x| text_image(&buffer, x.0, x.1, x.2, x.3)).collect());
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
        if hover.waiting[3] < IN_FLIGHT {
            now.body = strips.body.take();
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
                let mut texts = vec![];
                hover.chest_rows = rows
                    .into_iter()
                    .map(|(names, count)| {
                        let name_read = names.iter().map(text).collect::<Vec<_>>().join(" ");
                        texts.push((name_read.clone(), text(&count)));
                        ChestRow { name_read, count_read: text(&count) }
                    })
                    .collect();
                // The chests that list these rows best get the vote. Several do when they list the
                // same things, or when a numeral that tells them apart did not read. Those that
                // open the way the header says go first: the header's wording varies ("Obtain one
                // of the following items at random" reads as obtain all), so it only breaks ties.
                let mut listing = chests_listing(&texts);
                let kind = winner(&hover.chest_kind_votes).map(|x| x.0);
                if listing.iter().any(|x| Some(x.0.kind) == kind) {
                    listing.retain(|x| Some(x.0.kind) == kind);
                }
                let best = listing.iter().map(|x| x.1).fold(0.0, f64::max);
                for (variant, _) in listing.iter().filter(|x| x.1 >= best - 1e-6) {
                    *hover.chest_votes.entry(variant.id).or_default() += 1;
                }
                hover.waiting[2] -= 1;
            }
            if let Some(lines) = &read.body {
                let lines: Vec<String> = lines.iter().map(text).collect();
                if let Some(label) = item_from_body(&lines).and_then(|item| item.label.clone()) {
                    *hover.body_votes.entry(label).or_default() += 1;
                }
                hover.waiting[3] -= 1;
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
        // The text says more than the picture: books of one kind share a picture, and the two
        // kinds look too alike for the picture match to pick one.
        let titled = hover.title.as_ref().and_then(|title| match_title(title));
        hover.label = winner(&hover.body_votes)
            .map(|x| x.0)
            .or_else(|| titled?.label.clone());
        let labelled = || ITEMS.iter().find(|item| item.label.is_some() && item.label == hover.label);
        hover.icon = titled
            .and_then(|item| item.icon.clone())
            .or_else(|| labelled()?.icon.clone())
            .or_else(|| winner(&hover.icon_votes).map(|x| x.0));
        let chest = self.hovered_chest(hover);
        // a chest is drawn with the icon of any of the chests it can be
        let icons: Vec<String> = match &chest {
            Some(chest) => chest.icons.clone(),
            None => hover.icon.clone().into_iter().collect(),
        };
        hover.slot =
            self.hovered_slot(&hover.candidates, hover.bar.y as f64, s, &icons, hover.amount.as_ref());
        // a slot the user edited is left alone
        let slot = hover.slot.filter(|slot| !self.edits.contains_key(slot));
        if let Some(slot) = slot.and_then(|slot| self.slot_infos.get_mut(&slot)) {
            slot.hovered = true;
            // A hover that read nothing leaves what an earlier one read: a tooltip seen for a
            // frame or two while fading in has neither.
            if hover.tradability.is_some() {
                slot.tradability = hover.tradability;
            }
            if hover.amount.is_some() {
                slot.tooltip_amount = hover.amount.clone();
            }
            if hover.label.is_some() {
                slot.label = hover.label.clone();
            }
            // no amount line was found, or no two crops of it ever agreed
            slot.tooltip_failed = slot.tooltip_amount.is_none()
                && hover.waiting[1] == 0
                && (over || hover.sent[1] >= MAX_READS);
        }
        // a hover is often only seen for a frame or two, so one read is enough to store the chest
        if let Some(mut chest) = chest {
            chest.amount = hover.amount.clone();
            chest.tradability = hover.tradability;
            chest.column = hover.chest_place.0;
            // The slot the tooltip was level with can be a row it only happened to line up with,
            // pushed up. Then that slot holds something else.
            let other = |address: &SlotAddress| {
                let info = self.slot_infos.get(address).filter(|info| info.icon_name_score.is_some());
                info.is_some_and(|info| !info.shows(&chest.icons))
            };
            chest.slot = hover.slot.or(hover.chest_place.1.filter(|address| !other(address)));
            chest.rows = hover.chest_rows.clone();
            hover.chest_index = Some(self.store_chest(chest, hover.chest_index));
        }
    }

    // The chest a hover's rows read as: the chests with the most votes, and of those the ones
    // drawn like the slot the tooltip is level with, else like any slot beside it, if some are.
    fn hovered_chest(&self, hover: &Hover) -> Option<Chest> {
        let top = most(&hover.chest_votes);
        let mut variants: Vec<u32> =
            hover.chest_votes.iter().filter(|x| *x.1 == top).map(|x| *x.0).collect();
        variants.sort();
        let icons_of = |address: &SlotAddress| self.slot_infos.get(address).into_iter().flat_map(|info| info.icons());
        let level: Vec<&String> = hover.chest_place.1.iter().flat_map(icons_of).collect();
        let beside: Vec<&String> = hover.candidates.iter().flat_map(|(address, _)| icons_of(address)).collect();
        let drawn_like = |icons: &[&String]| {
            let icons: Vec<String> = icons.iter().map(|x| x.to_string()).collect();
            move |id: &u32| variant(*id).template().is_some_and(|x| icons.contains(&x))
        };
        for icons in [level, beside] {
            if variants.iter().any(drawn_like(&icons)) {
                variants.retain(drawn_like(&icons));
                break;
            }
        }
        (!variants.is_empty()).then(|| Chest::new(variants))
    }

    // slots on the active pages in the column the tooltip sits against, with their top
    fn slots_beside(&self, bar: &TitleBar, s: f64) -> Vec<(SlotAddress, f64)> {
        let active_pages = self.tooltip_page_num();
        let tolerance = SLOT_TOLERANCE * s;
        let (bar_left, bar_right) = (bar.x as f64, (bar.x + bar.width) as f64);
        SLOT_ORDER
            .iter()
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
        icons: &[String],
        amount: Option<&String>,
    ) -> Option<SlotAddress> {
        let tolerance = SLOT_TOLERANCE * s;

        let candidates: Vec<(SlotAddress, bool)> = beside
            .iter()
            .copied()
            .filter(|(address, top)| {
                *top >= bar_top - SLOT_ABOVE_TOOLTIP * s
                    && self.slot_infos.get(address).is_some_and(|info| {
                        info.shows(icons)
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
        .or_else(|| {
            // Else the only one no other tooltip was already tied to. A slot's own count misreads
            // under the cursor; a tooltip's amount does not, and a stack has one amount.
            let mut free = candidates.iter().filter(|(address, _)| {
                let other = self.slot_infos[address].tooltip_amount.as_ref();
                amount.is_some() && other.is_none_or(|other| Some(other) == amount)
            });
            free.next().filter(|_| free.next().is_none()).map(|x| x.0)
        })
    }
}
