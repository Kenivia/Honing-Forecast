use crate::ocr::text::text_line;
use crate::tooltip::ocr_interface::ChestRowStrips;
use crate::{
    buffer::Buffer,
    scanner_state::{InventoryType, ScannerState, SlotAddress, Tradability},
    tooltip::items::{VARIANTS, Variant, numerals, numerals_are, variant, without_numerals},
};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use strsim::{levenshtein, normalized_levenshtein};

type Rect = (usize, usize, usize, usize); // frame rectangle (x0, y0, x1, y1)

const ROW_PASS: f64 = 0.8; // how alike a row's name and a content's title have to be
const ROW_CAP: usize = 10; // a tooltip lists no more rows than this, whatever the chest holds

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum ChestKind {
    SelectOne,
    Random,
    ObtainAll,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct ChestContent {
    pub item: String,
    pub amount: u32,
    // the chest of scanner_setup/templates/chests.json it is, when it is one
    pub chest: Option<u32>,
}

// what one row read as
#[derive(Debug, Serialize, Clone)]
pub struct ChestRow {
    pub name_read: String,
    pub count_read: String,
}

// A chest is one of scanner_setup/templates/chests.json, found by what its tooltip lists: its title is not read,
// and its icon only says which few it can be.
#[derive(Debug, Serialize, Clone)]
pub struct Chest {
    // The chests its rows read as, by id. More than one when they list the same things, which
    // matters where a chest inside them is not the same one.
    pub variants: Vec<u32>,
    // their titles, for display
    pub title: String,
    // of the first of them
    pub kind: ChestKind,
    pub contents: Vec<ChestContent>,
    // the slot icons those chests are drawn with; a chest counts for the slots showing one
    pub icons: Vec<String>,
    // every one of those chests opens to nothing that is counted
    pub irrelevant: bool,
    pub amount: Option<String>,
    pub tradability: Option<Tradability>,
    // inventory, page and column the tooltip sat against
    pub column: Option<(InventoryType, usize, usize)>,
    // only known when the tooltip was level with its slot
    pub slot: Option<SlotAddress>,
    pub rows: Vec<ChestRow>,
}

pub struct ChestRowLayout {
    pub names: Vec<Rect>,
    pub count: Rect,
    pub whole: Rect,
}

pub struct ChestLayout {
    pub kind: ChestKind,
    pub rows: Vec<ChestRowLayout>,
}

pub fn chest_strips(buffer: &Buffer, layout: &ChestLayout) -> Vec<ChestRowStrips> {
    let strip = |(x0, y0, x1, y1): Rect| text_line(buffer, x0, y0, x1, y1);
    layout
        .rows
        .iter()
        .map(|row| ChestRowStrips {
            names: row.names.iter().map(|x| strip(*x)).collect(),
            count: strip(row.count),
        })
        .collect()
}

// a title a chest can list, lower case
struct Title {
    text: String,
    // without the roman numeral it ends in, if it ends in one
    stem: Option<String>,
    // the digits of the text, or of the stem, and those of the titles that differ by digits alone
    numerals: String,
    rivals: Vec<String>,
}

struct Titles {
    titles: Vec<Title>,
    // for each chest, which of them its contents are
    contents: Vec<Vec<usize>>,
    // the longest roman numeral one ends in
    longest_tier: usize,
}

static TITLES: LazyLock<Titles> = LazyLock::new(|| {
    let mut titles: Vec<Title> = vec![];
    let contents = VARIANTS
        .iter()
        .map(|variant| {
            let mut index = |title: &String| {
                let text = title.to_lowercase();
                titles
                    .iter()
                    .position(|x| x.text == text)
                    .unwrap_or_else(|| {
                        let tier = text
                            .rsplit_once(' ')
                            .filter(|x| x.1.chars().all(|c| "ivx".contains(c)));
                        let stem = tier.map(|x| x.0.trim_end().to_string());
                        let numerals = numerals(stem.as_ref().unwrap_or(&text), false);
                        titles.push(Title {
                            text,
                            stem,
                            numerals,
                            rivals: vec![],
                        });
                        titles.len() - 1
                    })
            };
            variant.contents.iter().map(|x| index(&x.0)).collect()
        })
        .collect();
    let bare = |x: &Title| without_numerals(x.stem.as_ref().unwrap_or(&x.text), false);
    let mut alike: AHashMap<String, Vec<String>> = AHashMap::new();
    for title in &titles {
        alike
            .entry(bare(title))
            .or_default()
            .push(title.numerals.clone());
    }
    for title in &mut titles {
        title.rivals = alike[&bare(title)].clone();
    }
    let tier = |x: &Title| {
        x.stem
            .as_ref()
            .map_or(0, |stem| x.text.len() - stem.len() - 1)
    };
    Titles {
        longest_tier: titles.iter().map(tier).max().unwrap(),
        titles,
        contents,
    }
});

// How far a row's name is from a title that ends in a roman numeral, in edits, and the name without
// what stands for the numeral. The tooltip's body font draws numerals as bare strokes, which do not
// read: "II" came back as "?I", "Il", "I", "?" and "!", glued to the word before it or not. So the
// numeral is taken to be whatever the name ends in, up to the longest numeral's length, and the
// rest is compared with the title's stem. A name that ends in nothing is the numeral short.
fn tiered(read: &str, stem: &str, longest: usize) -> (usize, String) {
    let chars: Vec<char> = read.chars().collect();
    (0..=(longest + 1).min(chars.len()))
        .map(|tail| {
            let (head, tail) = chars.split_at(chars.len() - tail);
            let head = head.iter().collect::<String>().trim_end().to_string();
            let numeral = tail.iter().filter(|c| !c.is_whitespace()).count();
            let missing = if numeral == 0 {
                2
            } else {
                numeral.saturating_sub(longest)
            };
            (levenshtein(&head, stem) + missing, head)
        })
        .min()
        .unwrap()
}

// How alike a row's name and a title are, if the row can be that title. Which numeral a title
// ends in does not read, so every tier of a chest scores the same. Digits do, and the name's have
// to be the title's.
fn row_is(read: &str, title: &Title, all: &Titles) -> Option<f64> {
    if read.len().abs_diff(title.text.len()) * 3 > title.text.len() {
        return None;
    }
    let (alike, numbered) = match &title.stem {
        None => (normalized_levenshtein(read, &title.text), read.to_string()),
        Some(stem) => {
            let (apart, head) = tiered(read, stem, all.longest_tier);
            (
                1.0 - apart as f64 / (stem.chars().count() + 2).max(read.chars().count()) as f64,
                head,
            )
        }
    };
    let digits = || numerals_are(&numerals(&numbered, false), &title.numerals, &title.rivals);
    (alike >= ROW_PASS && digits()).then_some(alike)
}

// The chests these rows are of, each with how well the names agree, and whether the rows are all
// such a chest holds. Every row has to be a content of its own with the right count, but for as
// many rows as a chest has for things that differ by class. A tooltip cannot list all of a chest
// that holds more than it has rows for, or whose rows depend on the class, and its last rows can
// be off the screen: such a chest also fits rows that are only part of it, half a tooltip's worth
// at least where it is its size that keeps it from being whole. Those count when no chest is all
// of the rows. A chest that could be listed whole never fits in part: one row is in many chests.
pub fn chests_listing(rows: &[(String, String)]) -> (Vec<(&'static Variant, f64)>, bool) {
    let all = &*TITLES;
    let rows: Vec<(String, Option<u32>)> = rows
        .iter()
        .map(|(name, count)| {
            let name = name.split("(Bound").next().unwrap().trim().to_lowercase();
            let digits: String = count.chars().filter(char::is_ascii_digit).collect();
            (name, digits.parse().ok())
        })
        .collect();
    // each row against each title, once
    let mut alike: AHashMap<(usize, usize), Option<f64>> = AHashMap::new();
    let (mut whole, mut partly) = (vec![], vec![]);
    for (variant, contents) in VARIANTS.iter().zip(&all.contents) {
        let count = variant.contents.len();
        let mut used = vec![false; count];
        let (mut total, mut matched) = (0.0, 0);
        for (row, (name, amount)) in rows.iter().enumerate() {
            let best = (0..count)
                .filter(|x| !used[*x] && Some(variant.contents[*x].1) == *amount)
                .filter_map(|x| {
                    let score = *alike
                        .entry((row, contents[x]))
                        .or_insert_with(|| row_is(name, &all.titles[contents[x]], all));
                    Some((score?, x))
                })
                .max_by(|a, b| a.0.total_cmp(&b.0));
            if let Some((score, x)) = best {
                used[x] = true;
                total += score;
                matched += 1;
            }
        }
        if matched == 0 || rows.len() - matched > variant.extra {
            continue;
        }
        let fit = (variant, total / matched as f64);
        if matched == count {
            whole.push(fit);
        } else if variant.by_class || (count > ROW_CAP && rows.len() * 2 >= ROW_CAP) {
            partly.push(fit);
        }
    }
    if whole.is_empty() {
        (partly, false)
    } else {
        (whole, true)
    }
}

impl Chest {
    fn with(
        mut self,
        slot: Option<SlotAddress>,
        amount: Option<String>,
        tradability: Option<Tradability>,
    ) -> Chest {
        (self.slot, self.amount, self.tradability) = (slot, amount, tradability);
        self
    }

    pub fn new(variants: Vec<u32>) -> Chest {
        let first = variant(variants[0]);
        let mut icons: Vec<String> = variants
            .iter()
            .filter_map(|id| variant(*id).template())
            .collect();
        icons.sort();
        icons.dedup();
        // chests that list the same things go by several names
        let mut titles: Vec<&str> = vec![];
        for id in &variants {
            let title = variant(*id).title.as_str();
            if !titles.contains(&title) {
                titles.push(title);
            }
        }
        let more = if titles.len() > 2 {
            format!(" and {} more", titles.len() - 2)
        } else {
            String::new()
        };
        Chest {
            title: titles[..titles.len().min(2)].join(" / ") + &more,
            kind: first.kind,
            contents: first
                .contents
                .iter()
                .map(|(item, amount, chest)| ChestContent {
                    item: item.clone(),
                    amount: *amount,
                    chest: *chest,
                })
                .collect(),
            icons,
            irrelevant: variants.iter().all(|id| variant(*id).irrelevant),
            variants,
            amount: None,
            tradability: None,
            column: None,
            slot: None,
            rows: vec![],
        }
    }
}

impl Chest {
    // a brief hover can end before the amount or the bind line was read, which then matches anything
    fn same_contents(&self, other: &Chest) -> bool {
        fn agree<T: PartialEq>(a: &Option<T>, b: &Option<T>) -> bool {
            a.is_none() || b.is_none() || a == b
        }
        self.column == other.column
            && self.variants.iter().any(|x| other.variants.contains(x))
            && agree(&self.tradability, &other.tradability)
            && agree(&self.amount, &other.amount)
    }
}

impl ScannerState {
    // Where this chest goes in the list: over the one last seen in its slot, else over one in the
    // same column that reads the same, else at the end.
    // `resolved`: its slot is the one its hover was tied to, not only the one it is level with
    pub fn store_chest(&mut self, mut chest: Chest, index: Option<usize>, resolved: bool) -> usize {
        self.chests_changed = true;
        let index = index.or_else(|| {
            self.chests.iter().position(|old| {
                (chest.slot.is_some() && old.slot == chest.slot) || old.same_contents(&chest)
            })
        });
        match index {
            Some(index) => {
                // a slot once found is kept while the entry still describes the same chest
                let old = &self.chests[index];
                if old.same_contents(&chest) {
                    // two reads of one chest: it is one of those both can be
                    let both = chest
                        .variants
                        .iter()
                        .copied()
                        .filter(|x| old.variants.contains(x));
                    chest = Chest {
                        rows: chest.rows,
                        column: chest.column,
                        ..Chest::new(both.collect())
                    }
                    .with(chest.slot, chest.amount, chest.tradability);
                    if !resolved {
                        chest.slot = old.slot.or(chest.slot);
                    }
                    chest.amount = chest.amount.or(old.amount.clone());
                    chest.tradability = chest.tradability.or(old.tradability);
                }
                self.chests[index] = chest;
                index
            }
            None => {
                self.chests.push(chest);
                self.chests.len() - 1
            }
        }
    }
}
