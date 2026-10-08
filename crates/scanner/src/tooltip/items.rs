use serde::Deserialize;
use std::sync::LazyLock;
use strsim::normalized_levenshtein;

const TITLE_PASS: f64 = 0.9;
const TITLE_MARGIN: f64 = 0.05; // over the next best
const TITLE_NEAR: f64 = 0.85;

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

const CHESTS: &str = include_str!("../../../../templates/chest.json");

// Materials, then the chests that sit in slots, then the chests only ever listed inside another
// chest (templates/inner_chests.json), which are here for their titles alone.
pub static ITEMS: LazyLock<Vec<Item>> = LazyLock::new(|| {
    let mut items: Vec<Item> =
        serde_json::from_str(include_str!("../../../../templates/items.json")).unwrap();
    items.extend(serde_json::from_str::<Vec<Item>>(CHESTS).unwrap());
    let inner: Vec<Item> =
        serde_json::from_str(include_str!("../../../../templates/inner_chests.json")).unwrap();
    // a title in both tables would never be clear of itself
    for item in inner {
        if !items.iter().any(|old| old.title == item.title) {
            items.push(item);
        }
    }
    items
});

static CHEST_ICONS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let chests: Vec<Item> = serde_json::from_str(CHESTS).unwrap();
    chests.into_iter().filter_map(|chest| chest.icon).collect()
});

pub fn is_chest_icon(icon: &str) -> bool {
    CHEST_ICONS.iter().any(|x| x == icon)
}

// an icon several materials are drawn with, which only the tooltip tells apart
pub fn shares_icon(icon: &str) -> bool {
    let mut labels = ITEMS
        .iter()
        .filter(|item| item.icon.as_deref() == Some(icon))
        .filter_map(|item| item.label.as_ref());
    labels.next().is_some_and(|first| labels.any(|label| label != first))
}

// The book a tooltip's description lines are of: "Honing Lv. 11 (1,645): +10%" for each level,
// and its gear in "Determination of Destiny Weapon". A frame that lost a line gives levels no
// book has, and so nothing.
pub fn item_from_body(lines: &[String]) -> Option<&'static Item> {
    let levels: Vec<u32> = lines
        .iter()
        .filter_map(|line| {
            let digits = line.split_once("Honing Lv")?.1.chars().skip_while(|c| !c.is_ascii_digit());
            digits.take_while(char::is_ascii_digit).collect::<String>().parse().ok()
        })
        .collect();
    let mut gear = ["Weapon", "Armor"]
        .into_iter()
        .filter(|gear| lines.iter().any(|line| line.contains(gear)));
    let gear = gear.next().filter(|_| gear.next().is_none())?;
    let body = format!("{gear} {}-{}", levels.first()?, levels.last()?);
    ITEMS.iter().find(|item| item.body.as_ref() == Some(&body))
}

// Digits and roman numerals, which is all that tells some titles apart. A numeral with a character
// the recogniser did not know in it is kept as it is, so it matches nothing. Of a word with digits
// in it only the digits count: the brackets of "[15-18]" come back as any of "[(I".
fn numerals(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|word| {
            if word.chars().any(|c| c.is_ascii_digit()) {
                Some(word.chars().filter(char::is_ascii_digit).collect())
            } else {
                word.chars().all(|c| "ivx?".contains(c)).then(|| word.to_string())
            }
        })
        .collect()
}

// an OCR'd title to the known title it is, if it is clearly one of them
pub fn match_title(read: &str) -> Option<&'static Item> {
    let read = read.to_lowercase();
    // one wrong numeral barely changes the similarity ("Level 3" / "Level 4", "Pouch II" / "Pouch III"), so they have to match exactly
    let mut scores: Vec<(f64, &Item)> = ITEMS
        .iter()
        .filter_map(|item| Some((item.title.as_ref()?.to_lowercase(), item)))
        .filter(|(title, _)| numerals(title) == numerals(&read))
        .map(|(title, item)| (normalized_levenshtein(&read, &title), item))
        .collect();
    scores.sort_by(|a, b| b.0.total_cmp(&a.0));
    let (best, item) = *scores.first()?;
    let second = scores.get(1).map_or(0.0, |x| x.0);
    // "... Pouch II" and "... Pouch III" are closer than the margin, so an exact read always counts
    (best == 1.0 || (best >= TITLE_PASS && best - second >= TITLE_MARGIN)).then_some(item)
}

// close to a known title without being accepted as one
pub fn near_title(read: &str) -> bool {
    let read = read.to_lowercase();
    match_title(&read).is_none()
        && ITEMS
            .iter()
            .filter_map(|item| item.title.as_ref())
            .any(|title| normalized_levenshtein(&read, &title.to_lowercase()) >= TITLE_NEAR)
}

// A read without the roman numeral it ends in. The tooltip's body font draws them as bare strokes,
// which come back as any of these characters.
fn without_tier(text: &str) -> &str {
    match text.rsplit_once(' ') {
        Some((rest, last)) if last.chars().all(|c| "ivxl!|?1".contains(c)) => rest,
        _ => text,
    }
}

// For a chest listed inside another chest: the numbered title this read is closest to, whatever
// the number. Which tier it is can come out wrong.
pub fn match_rough_title(read: &str) -> Option<&'static Item> {
    let read = read.to_lowercase();
    ITEMS
        .iter()
        .filter_map(|item| Some((item.title.as_ref()?.to_lowercase(), item)))
        .filter(|(title, _)| {
            without_tier(title) != title
                && normalized_levenshtein(without_tier(&read), without_tier(title)) >= TITLE_PASS
        })
        .map(|(title, item)| (normalized_levenshtein(&read, &title), item))
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|x| x.1)
}
