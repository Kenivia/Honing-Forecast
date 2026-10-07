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
    // chest.json only: the titles it is expected to hold
    pub contents: Option<Vec<String>>,
}

// chests come from their own file, templates/chest.json
pub static ITEMS: LazyLock<Vec<Item>> = LazyLock::new(|| {
    let mut items: Vec<Item> =
        serde_json::from_str(include_str!("../../../../templates/items.json")).unwrap();
    items.extend(
        serde_json::from_str::<Vec<Item>>(include_str!("../../../../templates/chest.json"))
            .unwrap(),
    );
    items
});

pub fn is_chest_icon(icon: &str) -> bool {
    ITEMS
        .iter()
        .any(|item| item.contents.is_some() && item.icon.as_deref() == Some(icon))
}

// what the chests drawn with this icon are expected to hold
pub fn expected_contents(icon: &str) -> Vec<&'static String> {
    ITEMS
        .iter()
        .filter(|item| item.icon.as_deref() == Some(icon))
        .flat_map(|item| item.contents.iter().flatten())
        .collect()
}

// Digits and roman numerals, which is all that tells some titles apart. A numeral with a character
// the recogniser did not know in it is kept as it is, so it matches nothing.
fn numerals(text: &str) -> Vec<&str> {
    text.split_whitespace()
        .filter(|word| word.chars().all(|c| c.is_ascii_digit() || "ivx?".contains(c)))
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
