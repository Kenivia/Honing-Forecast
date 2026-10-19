// Silver, gold and shards. Their totals are shown in three places, each beside the currency's
// icon: the inventory, the currency window and the bar at the top of the screen. How gold splits
// into bands is only on its tooltip. Reads of the same thing are pooled and voted on, apart from
// the slots, and stop once they agree.
use crate::{
    anchor_to_slot::{anchor_defs::with_slack, anchors::grown},
    buffer::Buffer,
    dev_utils::timing::timed,
    image_utils::{
        brightness::{contrast, tint},
        common::{Rect, get_resizer},
    },
    interface::{scan_result::SlotStatus, setup::icon_lookup},
    ocr::{jobs::Kind, text::line_of},
    scanner_state::{AnchorType, InventoryType, ScannerState},
    tooltip::{common::winner, ocr_interface::SETTLED},
};
use ahash::AHashMap;
use serde::Serialize;
use std::sync::LazyLock;

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    Silver,
    Gold,
    Shards,
}

// what a vote is about: a total, or a line of gold's tooltip
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Field {
    Total(Currency),
    Owned,
    Roster,
    Char,
    Deposit(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Place {
    Inventory,
    Window,
    Bar,
}

struct PlaceSpec {
    place: Place,
    // each icon, and where it is looked for from the place's origin (1440p)
    icons: Vec<(Currency, &'static str, Rect)>,
    // from the gold icon to its tooltip's title, which flips to the left only off the inventory
    tooltip: Vec<(f64, f64)>,
}

const ICON_SLACK: f64 = 6.0;
// of an icon's own contrast, what a match has to show
const MIN_CONTRAST: f64 = 0.4;
// apart from its template's, which the matched icon's tint may be: gold's is 0.35, silver's 0.04
const MAX_TINT_SHIFT: f64 = 0.15;
// the bar shows whichever currencies the user picked, in any order
const BAR: Rect = Rect::ui((0.0, 0.0), 1900.0, 40.0);

static PLACES: LazyLock<Vec<PlaceSpec>> = LazyLock::new(|| {
    let at =
        |x: f64, y: f64, width: f64, height: f64| with_slack((x, y), width, height, ICON_SLACK);
    vec![
        PlaceSpec {
            place: Place::Inventory,
            icons: vec![
                (
                    Currency::Silver,
                    "Silver coin",
                    at(461.0, 815.0, 24.0, 24.0),
                ),
                (Currency::Gold, "Gold coin", at(684.0, 815.0, 24.0, 24.0)),
            ],
            tooltip: vec![(44.0, 9.0), (-544.0, 9.0)],
        },
        // where the game first puts it; its rows are taken to be where they start
        PlaceSpec {
            place: Place::Window,
            icons: vec![
                (
                    Currency::Silver,
                    "Silver coin",
                    at(415.0, 271.0, 24.0, 24.0),
                ),
                (Currency::Gold, "Gold coin", at(416.0, 322.0, 24.0, 24.0)),
                (
                    Currency::Shards,
                    "Destiny shard",
                    at(419.0, 680.0, 16.0, 24.0),
                ),
            ],
            tooltip: vec![(56.0, 9.0)],
        },
        PlaceSpec {
            place: Place::Bar,
            icons: vec![
                (Currency::Silver, "Bar silver coin", BAR),
                (Currency::Gold, "Bar gold coin", BAR),
                (Currency::Shards, "Bar destiny shard", BAR),
            ],
            tooltip: vec![(63.0, 15.0)],
        },
    ]
});

const TOOLTIP_TITLE: &str = "Gold tooltip title";
// a tooltip pushed up by the bottom of the screen has its title this far down
const PUSHED_UP_TITLE: f64 = 520.0;
const TOOLTIP_SLACK: f64 = 10.0;
// its lines from the title: what each is, where it starts and how wide it may run
const TOOLTIP_LINES: [(Field, f64, f64, f64); 6] = [
    (Field::Owned, -2.0, 582.0, 392.0),
    (Field::Roster, -2.0, 661.0, 392.0),
    (Field::Char, -2.0, 740.0, 392.0),
    // "D+1: 0", read whole: a lone digit reads badly
    (Field::Deposit(1), -2.0, 819.0, 392.0),
    (Field::Deposit(2), -2.0, 846.0, 392.0),
    (Field::Deposit(3), -2.0, 873.0, 392.0),
];
const TOOLTIP_LINE_HEIGHT: f64 = 22.0;
const PARTS: [Field; 6] = [
    Field::Owned,
    Field::Roster,
    Field::Char,
    Field::Deposit(1),
    Field::Deposit(2),
    Field::Deposit(3),
];
// lit, as captured, for the title's colour to count
const INK: i32 = 90;
// a column with anything this much brighter than what is around it has text in it: the window is
// see-through, and some colours are dim
const INK_ABOVE: i32 = 40;
// a number ends where this many columns in a row have none
const WORD_GAP: f64 = 12.0;
// past the icon's template, where a number may start; how far left of it a number starts at
// most; and how long it runs
const CLEAR: f64 = 2.0;
const TO_ICON: f64 = 24.0;
const LONGEST: f64 = 260.0;
// a number's line, either side of its icon's middle
const HALF_LINE: f64 = 13.0;
// Left and right of the text, one per read in turn: the recogniser drops a first or last digit at
// some widths, and at most of them past 3 px (1080 lines), and the same crop is the same read. One
// thing gets this many reads, agreeing or not.
const MARGINS: [(f64, f64); 6] = [
    (2.0, 2.0),
    (1.0, 3.0),
    (3.0, 1.0),
    (0.0, 4.0),
    (3.0, 3.0),
    (1.0, 1.0),
];
const MAX_READS: usize = MARGINS.len();

#[derive(Debug, Default)]
pub struct FieldReads {
    pub votes: AHashMap<u64, usize>,
    // one batch out at a time
    pub waiting: Vec<u32>,
    pub sent: usize,
    // lines already counted: a crop seen before is the same read
    pub voted: Vec<u32>,
}

#[derive(Debug, Default)]
pub struct Currencies {
    pub fields: AHashMap<Field, FieldReads>,
    // where each icon was last found, looked at first
    pub last_at: AHashMap<(Place, Currency), Rect>,
    // something unsettled was on screen this scan
    pub busy: bool,
}

pub enum Read {
    Unseen,
    Reading,
    Settled(u64),
    Disagree,
}

impl FieldReads {
    fn read(&self) -> Read {
        match winner(&self.votes) {
            Some((value, count)) if count >= SETTLED => Read::Settled(value),
            _ if self.waiting.is_empty() && self.sent >= MAX_READS => Read::Disagree,
            _ => Read::Reading,
        }
    }

    fn wants(&self) -> bool {
        matches!(self.read(), Read::Reading) && self.waiting.is_empty()
    }
}

fn lit(buffer: &Buffer, x: usize, y: usize) -> i32 {
    buffer.raw(x, y).into_iter().max().unwrap()
}

// what text in a rectangle is brighter than: mostly it is background
fn ink_level(buffer: &Buffer, (x0, y0, x1, y1): (usize, usize, usize, usize)) -> i32 {
    let mut levels: Vec<i32> = (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| lit(buffer, x, y)))
        .collect();
    let middle = levels.len() / 2;
    *levels.select_nth_unstable(middle).1 + INK_ABOVE
}

fn inked(buffer: &Buffer, x: usize, y0: usize, y1: usize, ink: i32) -> bool {
    (y0..y1).any(|y| lit(buffer, x, y) > ink)
}

// the number just left of an icon, up to the first wide gap
fn number_left_of(buffer: &Buffer, icon: Rect, s: f64) -> Option<(usize, usize, usize, usize)> {
    let middle = icon.top_left.1 + icon.height / 2.0;
    let y0 = (middle - HALF_LINE * s).max(0.0) as usize;
    let y1 = ((middle + HALF_LINE * s) as usize).min(buffer.height);
    let start = (icon.top_left.0 - CLEAR * s).max(0.0) as usize;
    let lowest = (icon.top_left.0 - LONGEST * s).max(0.0) as usize;
    let ink = ink_level(buffer, (lowest, y0, start, y1));
    let (mut right, mut left, mut gap) = (None, 0, 0);
    for x in (lowest..start).rev() {
        if inked(buffer, x, y0, y1, ink) {
            right.get_or_insert(x);
            (left, gap) = (x, 0);
        } else if right.is_some() {
            gap += 1;
            if gap as f64 > WORD_GAP * s {
                break;
            }
        } else if (start - x) as f64 > TO_ICON * s {
            return None;
        }
    }
    Some((left, y0, right? + 1, y1))
}

// the text in a rectangle, cut to where it starts and ends
fn text_in(buffer: &Buffer, area: Rect) -> Option<(usize, usize, usize, usize)> {
    let (x0, y0) = (area.top_left.0 as usize, area.top_left.1 as usize);
    let (x1, y1) = (x0 + area.width as usize, y0 + area.height as usize);
    let ink = ink_level(buffer, (x0, y0, x1, y1));
    let mut columns = (x0..x1).filter(|x| inked(buffer, *x, y0, y1, ink));
    let left = columns.next()?;
    Some((left, y0, columns.next_back().unwrap_or(left) + 1, y1))
}

// The title is yellow; white lines of the tooltip say the same words.
fn is_yellow(buffer: &Buffer, area: Rect) -> bool {
    let (x0, y0) = (area.top_left.0 as usize, area.top_left.1 as usize);
    let pixels: Vec<[i32; 3]> = (y0..y0 + area.height as usize)
        .flat_map(|y| (x0..x0 + area.width as usize).map(move |x| (x, y)))
        .map(|(x, y)| buffer.raw(x, y))
        .filter(|p| p.iter().max().unwrap() > &INK)
        .collect();
    let yellow = pixels.iter().filter(|[r, _, b]| b * 2 < *r).count();
    !pixels.is_empty() && yellow * 5 > pixels.len() * 4
}

// "15,275,481": the recogniser drops a first or last digit, which breaks the groups of three. It
// also misses a separator now and then, which is no harm.
fn grouped(text: &str) -> Option<u64> {
    let groups: Vec<&str> = text.trim().split([',', '.']).collect();
    let well_formed = groups
        .iter()
        .all(|x| !x.is_empty() && x.chars().all(|c| c.is_ascii_digit()))
        && groups[1..].iter().all(|x| x.len() % 3 == 0);
    well_formed.then(|| groups.concat().parse().unwrap())
}

fn parse(field: Field, text: &str) -> Option<u64> {
    match field {
        // the amount, then "/99,999,999,999"
        Field::Owned | Field::Roster | Field::Char => grouped(text.split_once('/')?.0),
        // after "D+1:", or without the colon after the day's digit
        Field::Deposit(_) => match text.split_once(':') {
            Some((_, amount)) => grouped(amount),
            None => grouped(text.split_once(|c: char| c.is_ascii_digit())?.1),
        },
        Field::Total(_) => grouped(text),
    }
}

#[derive(Debug, Serialize)]
pub struct CurrencyResult {
    pub currency: Currency,
    pub status: SlotStatus,
    // why it is not good yet, for the user
    pub reason: String,
    pub value: Option<u64>,
    // char bound, roster bound, tradable
    pub bands: Option<[u64; 3]>,
}

impl ScannerState {
    fn place_origin(&self, place: Place) -> Option<(f64, f64)> {
        match place {
            Place::Inventory => self.inventory_root(InventoryType::CharInventory),
            Place::Window => {
                let anchor = &self.anchors[&AnchorType::Currency];
                anchor.is_found().then_some(anchor.position_root?)
            }
            // hidden while the storage layout is open
            Place::Bar => (!self.storage_open()).then_some(self.screen_info.corner),
        }
    }

    fn field_read(&self, field: Field) -> Read {
        self.currency
            .fields
            .get(&field)
            .map_or(Read::Unseen, FieldReads::read)
    }

    fn wants(&self, field: Field) -> bool {
        self.currency
            .fields
            .get(&field)
            .is_none_or(FieldReads::wants)
    }

    // Something on screen is unsettled, or a read is out and the next is cut when it is back:
    // a still screen is scanned on for either.
    pub fn currency_unfinished(&self) -> bool {
        self.currency.busy || self.currency.fields.values().any(|x| !x.waiting.is_empty())
    }

    // looked for where it last was first, the bar being wide
    fn find_icon(&mut self, key: (Place, Currency), name: &str, bound: Rect) -> Option<Rect> {
        let s = self.screen_info.scale_factor;
        let near = self
            .currency
            .last_at
            .get(&key)
            .map(|at| grown(*at, 3.0 * s));
        let found = [near, Some(bound)].into_iter().flatten().find_map(|area| {
            let (found, ..) = self.find_template(name, area)?;
            self.icon_shown(name, found).then_some(found)
        })?;
        self.currency.last_at.insert(key, found);
        Some(found)
    }

    // Matching does not go by contrast, so an icon showing faintly through a tooltip passes it,
    // and goes by grey, so the silver coin passes for the gold one.
    fn icon_shown(&mut self, name: &str, found: Rect) -> bool {
        let seen = self.crop_buffer(found);
        let ui_height = self.screen_info.effective_height;
        let own = icon_lookup(name, ui_height, get_resizer(&mut self.resizer));
        contrast(&seen) > MIN_CONTRAST * contrast(&own)
            && (tint(&seen) - tint(&own)).abs() < MAX_TINT_SHIFT
    }

    // Enough to settle at once, each cut with the next margin: a tooltip may be up only briefly.
    fn send_currency_read(
        &mut self,
        field: Field,
        (x0, y0, x1, y1): (usize, usize, usize, usize),
        kind: Kind,
    ) {
        let sent = self.currency.fields.entry(field).or_default().sent;
        // scaled down they round to the same pixels, and the same crop is the same read
        let px = |x: f64| (x * self.screen_info.scale_factor).round() as usize;
        let mut margins: Vec<(usize, usize)> = vec![];
        for (left, right) in MARGINS {
            let mut margin = (px(left), px(right));
            while margins.contains(&margin) {
                margin.1 += 1;
            }
            margins.push(margin);
        }
        for (left, right) in margins[sent..(sent + SETTLED).min(MAX_READS)].to_vec() {
            let line = line_of(
                kind.clone(),
                &self.buffer,
                x0.saturating_sub(left),
                y0,
                (x1 + right).min(self.buffer.width),
                y1,
            );
            self.read_count += 1;
            let id = self.request_ocr(line, 0, self.read_count);
            let reads = self.currency.fields.get_mut(&field).unwrap();
            reads.waiting.push(id);
            reads.sent += 1;
        }
        self.currency.busy = true;
    }

    // its title beside one of the places gold was found
    fn find_gold_tooltip(&mut self, golds: &[(Rect, Vec<(f64, f64)>)]) -> Option<Rect> {
        let s = self.screen_info.scale_factor;
        let ui_height = self.screen_info.effective_height;
        let lowest = self.screen_info.corner.1 + PUSHED_UP_TITLE * s;
        let (width, height) = icon_lookup(TOOLTIP_TITLE, ui_height, get_resizer(&mut self.resizer))
            .offset
            .pixel_size();
        for (gold, offsets) in golds {
            for (dx, dy) in offsets {
                let top_left = (
                    gold.top_left.0 + dx * s,
                    (gold.top_left.1 + dy * s).min(lowest),
                );
                let at = Rect::screen(top_left, width as f64, height as f64, ui_height);
                if let Some((found, ..)) =
                    self.find_template(TOOLTIP_TITLE, grown(at, TOOLTIP_SLACK * s))
                    && is_yellow(&self.buffer, found.whole_pixels())
                {
                    return Some(found);
                }
            }
        }
        None
    }

    // Finds the icons, and the gold tooltip beside one, and sends off what is still unread.
    pub fn update_currency(&mut self) {
        let s = self.screen_info.scale_factor;
        let ui_height = self.screen_info.effective_height;
        // lines are cut as captured, and read at what the brightness is thought to be
        self.buffer.brightness = self.screen_info.brightness.unwrap_or(50.0);
        self.currency.busy = false;
        let parts_wanted = PARTS
            .iter()
            .any(|x| !matches!(self.field_read(*x), Read::Settled(_)));
        let mut golds = vec![];
        for spec in PLACES.iter() {
            let Some(origin) = self.place_origin(spec.place) else {
                continue;
            };
            for (currency, name, bound) in &spec.icons {
                let total = Field::Total(*currency);
                let is_gold = *currency == Currency::Gold;
                // gold read off its tooltip needs nothing more
                let needed = if is_gold {
                    parts_wanted
                } else {
                    !matches!(self.field_read(total), Read::Settled(_))
                };
                if !needed {
                    continue;
                }
                let bound = bound.scaled(ui_height).shifted(origin);
                let key = (spec.place, *currency);
                let icon = timed("currency/icons", || self.find_icon(key, name, bound));
                if is_gold {
                    // The cursor is over it while its tooltip is up, so that is looked for from
                    // where it should be, or on the bar from where it last was.
                    let expected = match spec.place {
                        Place::Bar => self.currency.last_at.get(&key).copied(),
                        _ => Some(grown(bound, -ICON_SLACK * s)),
                    };
                    golds.extend(icon.or(expected).map(|at| (at, spec.tooltip.clone())));
                }
                let Some(icon) = icon else {
                    continue;
                };
                if self.wants(total)
                    && let Some(span) = number_left_of(&self.buffer, icon, s)
                {
                    self.send_currency_read(total, span, Kind::Digits);
                }
            }
        }
        if !parts_wanted {
            return;
        }
        let Some(title) = timed("currency/tooltip", || self.find_gold_tooltip(&golds)) else {
            return;
        };
        for (field, dx, dy, width) in TOOLTIP_LINES {
            if !self.wants(field) {
                continue;
            }
            let line = Rect::screen(
                (title.top_left.0 + dx * s, title.top_left.1 + dy * s),
                width * s,
                TOOLTIP_LINE_HEIGHT * s,
                ui_height,
            );
            if let Some(span) = text_in(&self.buffer, self.clipped(line)) {
                self.send_currency_read(field, span, Kind::Text);
            }
        }
    }

    // takes in the texts that came back
    pub fn resolve_currency(&mut self) {
        for (field, reads) in self.currency.fields.iter_mut() {
            let ocr_texts = &self.ocr_texts;
            let back: Vec<(u32, &String)> = reads
                .waiting
                .iter()
                .filter_map(|id| Some((*id, ocr_texts.get(id)?)))
                .collect();
            reads.waiting.retain(|id| !ocr_texts.contains_key(id));
            for (id, text) in back {
                if reads.voted.contains(&id) {
                    continue;
                }
                reads.voted.push(id);
                if let Some(value) = parse(*field, text) {
                    *reads.votes.entry(value).or_default() += 1;
                }
            }
        }
    }

    pub fn currency_results(&self) -> Vec<CurrencyResult> {
        let disagree = "Its reads disagree. Set it by hand in the manifest.".to_string();
        let single = |currency: Currency, bands: fn(u64) -> [u64; 3], unseen: &str| {
            let (status, reason, value) = match self.field_read(Field::Total(currency)) {
                Read::Settled(value) => (SlotStatus::Good, String::new(), Some(value)),
                Read::Reading => (SlotStatus::Pending, "Being read.".to_string(), None),
                Read::Unseen => (SlotStatus::Pending, unseen.to_string(), None),
                Read::Disagree => (SlotStatus::Error, disagree.clone(), None),
            };
            CurrencyResult {
                currency,
                status,
                reason,
                value,
                bands: value.map(bands),
            }
        };
        let silver = single(
            Currency::Silver,
            |x| [0, x, 0],
            "Not seen yet. Open the inventory or the currency window, or show silver in the bar at the top of the screen.",
        );
        let shards = single(
            Currency::Shards,
            |x| [x, 0, 0],
            "Not seen yet. Open the currency window, or show Destiny Shards in the bar at the top of the screen.",
        );
        vec![silver, self.gold_result(disagree), shards]
    }

    fn gold_result(&self, disagree: String) -> CurrencyResult {
        let total = self.field_read(Field::Total(Currency::Gold));
        let parts = PARTS.map(|x| self.field_read(x));
        let total_value = match total {
            Read::Settled(value) => Some(value),
            _ => None,
        };
        let result = |status, reason: String, value, bands| CurrencyResult {
            currency: Currency::Gold,
            status,
            reason,
            value,
            bands,
        };
        let settled: Vec<u64> = parts
            .iter()
            .filter_map(|x| match x {
                Read::Settled(value) => Some(*value),
                _ => None,
            })
            .collect();
        // the tooltip goes over what is shown, which can be half covered
        if settled.len() == PARTS.len() {
            // what is deposited becomes tradable in a few days, so it counts as such
            let tradable = settled[0] + settled[3..].iter().sum::<u64>();
            let bands = [settled[2], settled[1], tradable];
            let sum: u64 = bands.iter().sum();
            return result(SlotStatus::Good, String::new(), Some(sum), Some(bands));
        }
        if matches!(total, Read::Disagree) || parts.iter().any(|x| matches!(x, Read::Disagree)) {
            return result(SlotStatus::Error, disagree, total_value, None);
        }
        if parts.iter().any(|x| matches!(x, Read::Reading)) {
            return result(
                SlotStatus::Pending,
                "Its tooltip is being read. If this stays, hover your gold again.".into(),
                total_value,
                None,
            );
        }
        result(
            SlotStatus::NeedHover,
            "How much of it is bound is only on its tooltip. Hover your gold in the inventory, the currency window or the bar at the top of the screen.".into(),
            total_value,
            None,
        )
    }
}
