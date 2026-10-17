use crate::{
    scanner_state::{InventoryType, Tradability},
    tooltip::chest::ChestKind,
};
use ahash::AHashMap;
use serde::Deserialize;
use std::sync::LazyLock;
use strsim::{levenshtein, normalized_levenshtein};

const TITLE_PASS: f64 = 0.9;
const TITLE_MARGIN: f64 = 0.05; // over the next best

// one row of templates/items.json: an in-game title and the slot icon template it is drawn with
#[derive(Debug, Deserialize)]
pub struct Item {
    pub title: Option<String>,
    pub icon: Option<String>,
    // the material it counts as
    pub label: Option<String>,
    // books only: its gear and honing levels, as its description lines give them
    pub body: Option<String>,
}

// the materials: templates/items.json
pub static ITEMS: LazyLock<Vec<Item>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../../templates/items.json")).unwrap()
});

// One chest of templates/chests.json, which scripts/game_files/chests.py writes from the game's
// own tables: everything that opens to a material, to gold or to silver, and whatever else is
// drawn like one of those.
#[derive(Debug, Deserialize)]
pub struct Variant {
    pub id: u32,
    pub title: String,
    // the art it is drawn with and its rarity; only those that can sit in a slot have them
    pub icon: Option<String>,
    pub rarity: Option<String>,
    pub kind: ChestKind,
    // the item level it asks for, which is then written across its slot
    pub level: Option<u32>,
    // rows its tooltip has besides the contents, for things that differ by class
    #[serde(default)]
    pub extra: usize,
    // it opens to nothing that is counted, and is only there to be told from one that does
    #[serde(default)]
    pub irrelevant: bool,
    // its tooltip lists some of its contents only, depending on the class
    #[serde(default)]
    pub by_class: bool,
    // the ways the items it stands for are bound; none for one that counts for nothing
    #[serde(default)]
    pub binds: Vec<Tradability>,
    // title, amount, and the chest of this table it is
    pub contents: Vec<(String, u32, Option<u32>)>,
    // the same in a fixed order, to compare two chests by
    #[serde(skip)]
    pub sorted: Vec<(String, u32, Option<u32>)>,
}

impl Variant {
    // the name of its slot icon template
    pub fn template(&self) -> Option<String> {
        Some(format!("{}@{}", self.icon.as_ref()?, self.rarity.as_ref()?))
    }
}

pub static VARIANTS: LazyLock<Vec<Variant>> = LazyLock::new(|| {
    let mut variants: Vec<Variant> =
        serde_json::from_str(include_str!("../../../../templates/chests.json")).unwrap();
    for variant in &mut variants {
        variant.sorted = variant.contents.clone();
        variant.sorted.sort();
    }
    variants
});

static BY_TEMPLATE: LazyLock<AHashMap<String, Vec<&'static Variant>>> = LazyLock::new(|| {
    let mut map: AHashMap<String, Vec<&Variant>> = AHashMap::new();
    for variant in VARIANTS.iter() {
        if let Some(template) = variant.template() {
            map.entry(template).or_default().push(variant);
        }
    }
    map
});

pub fn variant(id: u32) -> &'static Variant {
    VARIANTS.iter().find(|x| x.id == id).unwrap()
}

// the chests drawn with this slot icon
pub fn variants_of(icon: &str) -> &'static [&'static Variant] {
    BY_TEMPLATE.get(icon).map_or(&[], |x| x.as_slice())
}

pub fn is_chest_icon(icon: &str) -> bool {
    BY_TEMPLATE.contains_key(icon)
}

// Some chest with this icon has its item level written across the slot. Only those that count:
// with the others in, plain chests of another art passed as this one with the level's rows left out.
pub fn has_level(icon: &str) -> bool {
    variants_of(icon)
        .iter()
        .any(|x| x.level.is_some() && !x.irrelevant)
}

// Whether every one of these chests opens to the same things, so that it does not matter which
// of them a slot holds. Those that open to nothing counted are all the same.
pub fn open_alike(variants: &[&Variant]) -> bool {
    variants.windows(2).all(|pair| {
        pair[0].irrelevant == pair[1].irrelevant
            && (pair[0].irrelevant
                || (pair[0].kind == pair[1].kind && pair[0].sorted == pair[1].sorted))
    })
}

// How these chests are bound, when all of them are bound one way. The game keeps what is bound to
// a character out of roster storage.
pub fn bound_alike<'a>(
    variants: impl IntoIterator<Item = &'a Variant>,
    inventory: InventoryType,
) -> Option<Tradability> {
    let mut binds = variants
        .into_iter()
        .flat_map(|x| &x.binds)
        .filter(|x| inventory != InventoryType::Roster || **x != Tradability::CharBound);
    let first = *binds.next()?;
    binds.all(|x| *x == first).then_some(first)
}

// an icon several materials are drawn with, which only the tooltip tells apart
pub fn shares_icon(icon: &str) -> bool {
    let mut labels = ITEMS
        .iter()
        .filter(|item| item.icon.as_deref() == Some(icon))
        .filter_map(|item| item.label.as_ref());
    labels
        .next()
        .is_some_and(|first| labels.any(|label| label != first))
}

// The book a tooltip's description lines are of: "Honing Lv. 11 (1,645): +10%" for each level,
// and its gear in "Determination of Destiny Weapon". A frame that lost a line gives levels no
// book has, and so nothing. The level is every digit before the bracket: "11" is drawn wide and
// comes back as "1 1".
pub fn item_from_body(lines: &[String]) -> Option<&'static Item> {
    let levels: Vec<u32> = lines
        .iter()
        .filter_map(|line| {
            let level = line.split_once("Honing Lv")?.1.split('(').next()?;
            level
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>()
                .parse()
                .ok()
        })
        .collect();
    let mut gear = ["Weapon", "Armor"]
        .into_iter()
        .filter(|gear| lines.iter().any(|line| line.contains(gear)));
    let gear = gear.next().filter(|_| gear.next().is_none())?;
    let body = format!("{gear} {}-{}", levels.first()?, levels.last()?);
    ITEMS.iter().find(|item| item.body.as_ref() == Some(&body))
}

// A word of a name that is a numeral: one with a digit in it, or for `roman` one of numeral
// letters only.
fn is_numeral(word: &str, roman: bool) -> bool {
    word.chars().any(|c| c.is_ascii_digit()) || (roman && word.chars().all(|c| "ivx?".contains(c)))
}

// The numerals of a name, run together. Of a word with digits only the digits count: the brackets
// of "[15-18]" come back as any of "[(I", and a space before a digit gets lost. A roman numeral
// with a character the recogniser did not know in it is kept as it is.
pub fn numerals(text: &str, roman: bool) -> String {
    let words = text
        .split_whitespace()
        .filter(|word| is_numeral(word, roman));
    words
        .flat_map(|word| {
            let digits = word.chars().any(|c| c.is_ascii_digit());
            word.chars().filter(move |c| !digits || c.is_ascii_digit())
        })
        .collect()
}

// a name without its numerals: names with the same one differ by numerals alone
pub fn without_numerals(text: &str, roman: bool) -> String {
    text.split_whitespace()
        .filter(|word| !is_numeral(word, roman))
        .collect::<Vec<_>>()
        .join(" ")
}

// Whether a read's numerals are a name's, where `rivals` are the numerals of the names that
// differ from it by numerals alone. They have to be closer to its own than to any rival's, with
// no more than a third of them wrong: "[11-14]" read as "(1l-14]" is still nowhere near "[15-18]",
// while a lone digit or a short roman numeral has to read right, a wrong one being another name.
pub fn numerals_are(read: &str, own: &str, rivals: &[String]) -> bool {
    let apart = levenshtein(read, own);
    apart * 3 <= own.chars().count()
        && rivals
            .iter()
            .all(|x| x == own || levenshtein(read, x) > apart)
}

// an OCR'd title to the one of these it clearly is
fn closest<T: Copy>(read: &str, titles: impl Iterator<Item = (&'static str, T)>) -> Option<T> {
    let read = read.to_lowercase();
    let seen = numerals(&read, true);
    // each title with what it is without its numerals, and its numerals
    let titles: Vec<(String, String, String, T)> = titles
        .map(|(title, value)| (title.to_lowercase(), value))
        .map(|(title, value)| {
            (
                without_numerals(&title, true),
                numerals(&title, true),
                title,
                value,
            )
        })
        .collect();
    // one wrong numeral barely changes the similarity ("Level 3" / "Level 4", "Pouch II" / "Pouch
    // III"), so the read's numerals have to be the title's
    let mut scores: Vec<(f64, T)> = titles
        .iter()
        .filter(|(bare, own, _, _)| {
            let rivals: Vec<String> = titles
                .iter()
                .filter(|x| x.0 == *bare)
                .map(|x| x.1.clone())
                .collect();
            numerals_are(&seen, own, &rivals)
        })
        .map(|(_, _, title, value)| (normalized_levenshtein(&read, title), *value))
        .collect();
    scores.sort_by(|a, b| b.0.total_cmp(&a.0));
    let (best, value) = *scores.first()?;
    let second = scores.get(1).map_or(0.0, |x| x.0);
    // "... Pouch II" and "... Pouch III" are closer than the margin, so an exact read always counts
    (best == 1.0 || (best >= TITLE_PASS && best - second >= TITLE_MARGIN)).then_some(value)
}

// an OCR'd title to the known title it is, if it is clearly one of them
pub fn match_title(read: &str) -> Option<&'static Item> {
    closest(
        read,
        ITEMS
            .iter()
            .filter_map(|item| Some((item.title.as_deref()?, item))),
    )
}

// Bars of gold and silver by title, each with the chests of that title. A bar's tooltip has no
// chest panel, so its title is all that says which it is.
static BAR_TITLES: LazyLock<Vec<(&'static str, Vec<u32>)>> = LazyLock::new(|| {
    let mut titles: Vec<(&str, Vec<u32>)> = vec![];
    let bars = VARIANTS.iter().filter(|x| {
        let money = matches!(x.contents.as_slice(), [(item, _, None)] if item == "Gold" || item == "Silver");
        money && x.kind == ChestKind::ObtainAll && x.template().is_some()
    });
    for bar in bars {
        match titles.iter_mut().find(|x| x.0 == bar.title) {
            Some(title) => title.1.push(bar.id),
            None => titles.push((&bar.title, vec![bar.id])),
        }
    }
    titles
});

pub fn match_bar_title(read: &str) -> Option<&'static str> {
    closest(read, BAR_TITLES.iter().map(|x| (x.0, x.0)))
}

pub fn bars_titled(title: &str) -> Vec<u32> {
    BAR_TITLES
        .iter()
        .find(|x| x.0 == title)
        .map_or(vec![], |x| x.1.clone())
}
