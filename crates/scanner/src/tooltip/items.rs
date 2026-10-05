use serde::Deserialize;
use std::sync::LazyLock;
use strsim::normalized_levenshtein;

const TITLE_PASS: f64 = 0.9;
const TITLE_MARGIN: f64 = 0.05; // over the next best

// one row of templates/items.json: an in-game title and the slot icon template it is drawn with
#[derive(Debug, Deserialize)]
pub struct Item {
    pub title: Option<String>,
    pub icon: Option<String>,
}

pub static ITEMS: LazyLock<Vec<Item>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../../templates/items.json")).unwrap()
});

fn digits(text: &str) -> String {
    text.chars().filter(char::is_ascii_digit).collect()
}

// an OCR'd title to the known title it is, if it is clearly one of them
pub fn match_title(read: &str) -> Option<&'static Item> {
    let read = read.to_lowercase();
    // one wrong digit barely changes the similarity ("Level 3" / "Level 4"), so digits have to match exactly
    let mut scores: Vec<(f64, &Item)> = ITEMS
        .iter()
        .filter_map(|item| Some((item.title.as_ref()?.to_lowercase(), item)))
        .filter(|(title, _)| digits(title) == digits(&read))
        .map(|(title, item)| (normalized_levenshtein(&read, &title), item))
        .collect();
    scores.sort_by(|a, b| b.0.total_cmp(&a.0));
    let (best, item) = *scores.first()?;
    let second = scores.get(1).map_or(0.0, |x| x.0);
    // "... Pouch II" and "... Pouch III" are closer than the margin, so an exact read always counts
    (best == 1.0 || (best >= TITLE_PASS && best - second >= TITLE_MARGIN)).then_some(item)
}
