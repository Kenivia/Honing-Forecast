use crate::ocr::{jobs::Line, text::text_line};
use crate::{
    buffer::Buffer,
    scanner_state::{InventoryType, ScannerState, SlotAddress, Tradability},
    tooltip::{
        items::{VARIANTS, Variant, variant},
    },
};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use strsim::normalized_levenshtein;

type Rect = (usize, usize, usize, usize); // frame rectangle (x0, y0, x1, y1)

const ROW_PASS: f64 = 0.8; // how alike a row's name and a content's title have to be

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
    // the chest of templates/chests.json it is, when it is one
    pub chest: Option<u32>,
}

// what one row read as
#[derive(Debug, Serialize, Clone)]
pub struct ChestRow {
    pub name_read: String,
    pub count_read: String,
}

// A chest is one of templates/chests.json, found by what its tooltip lists: its title is not read,
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

// the text lines of one row of a chest
#[derive(Debug, Clone)]
pub struct ChestRowStrips {
    pub names: Vec<Line>,
    pub count: Line,
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

// every title a chest can list, lower case, and for each chest which of them its contents are
static TITLES: LazyLock<(Vec<String>, Vec<Vec<usize>>)> = LazyLock::new(|| {
    let mut titles: Vec<String> = vec![];
    let contents = VARIANTS
        .iter()
        .map(|variant| {
            let mut index = |title: &String| {
                let title = title.to_lowercase();
                titles.iter().position(|x| *x == title).unwrap_or_else(|| {
                    titles.push(title);
                    titles.len() - 1
                })
            };
            variant.contents.iter().map(|x| index(&x.0)).collect()
        })
        .collect();
    (titles, contents)
});

// A name without the roman numeral it ends in, and whether it ends in one. The tooltip's body font
// draws them as bare strokes, which come back as any of these characters and in any number, glued
// to a bracket before them or not: "II" was read as "?I", "Il", "I" and "?".
fn without_tier(text: &str) -> (String, bool) {
    let text = text.replace(')', ") ");
    let text = text.trim_end();
    match text.rsplit_once(' ') {
        // a number is a number: "Level 1" is not "Level 2"
        Some((rest, last))
            if last.chars().all(|c| "ivxl!|?1".contains(c)) && !last.chars().all(|c| c.is_ascii_digit()) =>
        {
            (rest.trim_end().to_string(), true)
        }
        _ => (text.to_string(), false),
    }
}

// Whether two names have the same digits, which are all that tells some titles apart. Spaces come
// and go inside "[11-14]", and a bracket that reads as a 1 is one digit too many.
fn same_digits(read: &str, title: &str) -> bool {
    let digits = |text: &str| text.chars().filter(char::is_ascii_digit).collect::<String>();
    let (read_digits, title_digits) = (digits(read), digits(title));
    let fewer = |bracket: char| read.matches(bracket).count() < title.matches(bracket).count();
    read_digits == title_digits
        || (fewer(']') && read_digits.strip_suffix('1') == Some(&title_digits))
        || (fewer('[') && read_digits.strip_prefix('1') == Some(&title_digits))
}

// How alike a row's name and a title are, if the row can be that title. A numeral at the end only
// has to be there: which one it is does not read, so every tier of a chest scores the same.
fn row_is(read: &str, title: &str) -> Option<f64> {
    if read.len().abs_diff(title.len()) * 3 > title.len() {
        return None;
    }
    let ((read, read_tier), (title, title_tier)) = (without_tier(read), without_tier(title));
    let alike = normalized_levenshtein(&read, &title);
    (read_tier == title_tier && alike >= ROW_PASS && same_digits(&read, &title)).then_some(alike)
}

// The chests whose contents are these rows, each with how well the names agree. Every content has
// to be a row of its own with the right count; rows left over are allowed where a chest has some
// that differ by class.
pub fn chests_listing(rows: &[(String, String)]) -> Vec<(&'static Variant, f64)> {
    let (titles, contents) = &*TITLES;
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
    let mut out = vec![];
    for (variant, contents) in VARIANTS.iter().zip(contents) {
        let count = variant.contents.len();
        if rows.len() < count || rows.len() > count + variant.extra {
            continue;
        }
        let mut used = vec![false; rows.len()];
        let mut total = 0.0;
        let fits = variant.contents.iter().zip(contents).all(|((_, amount, _), title)| {
            let best = (0..rows.len())
                .filter(|row| !used[*row] && rows[*row].1 == Some(*amount))
                .filter_map(|row| {
                    let score = *alike
                        .entry((row, *title))
                        .or_insert_with(|| row_is(&rows[row].0, &titles[*title]));
                    Some((score?, row))
                })
                .max_by(|a, b| a.0.total_cmp(&b.0));
            best.is_some_and(|(score, row)| {
                used[row] = true;
                total += score;
                true
            })
        });
        if fits && count > 0 {
            out.push((variant, total / count as f64));
        }
    }
    out
}

impl Chest {
    fn with(mut self, slot: Option<SlotAddress>, amount: Option<String>, tradability: Option<Tradability>) -> Chest {
        (self.slot, self.amount, self.tradability) = (slot, amount, tradability);
        self
    }

    pub fn new(variants: Vec<u32>) -> Chest {
        let first = variant(variants[0]);
        let mut icons: Vec<String> = variants.iter().filter_map(|id| variant(*id).template()).collect();
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
        let more = if titles.len() > 2 { format!(" and {} more", titles.len() - 2) } else { String::new() };
        Chest {
            title: titles[..titles.len().min(2)].join(" / ") + &more,
            kind: first.kind,
            contents: first
                .contents
                .iter()
                .map(|(item, amount, chest)| ChestContent { item: item.clone(), amount: *amount, chest: *chest })
                .collect(),
            icons,
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
    pub fn store_chest(&mut self, mut chest: Chest, index: Option<usize>) -> usize {
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
                    let both = chest.variants.iter().copied().filter(|x| old.variants.contains(x));
                    chest = Chest { rows: chest.rows, column: chest.column, ..Chest::new(both.collect()) }
                        .with(chest.slot, chest.amount, chest.tradability);
                    chest.slot = old.slot.or(chest.slot);
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
