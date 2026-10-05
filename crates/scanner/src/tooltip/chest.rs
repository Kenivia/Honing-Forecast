use crate::{
    buffer::Buffer,
    image_utils::common::IntegerRectangle,
    scanner_state::{InventoryType, ScannerState, SlotAddress, Tradability},
    setup::OneIconConfig,
    tooltip::{
        items::{match_rough_title, match_title, near_title},
        title::text_image,
    },
};
use image::RgbaImage;
use serde::{Deserialize, Serialize};

type Rect = (usize, usize, usize, usize); // frame rectangle (x0, y0, x1, y1)

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ChestKind {
    SelectOne,
    Random,
    ObtainAll,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ChestContent {
    // a title from templates/items.json. Only roughly right for a chest inside the chest: its tier can be off
    pub item: String,
    pub amount: u32,
    pub bound: bool,
}

// everything read off one row, known item or not. Debug only
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChestRow {
    pub name_read: String,
    pub count_read: String,
    pub item: Option<String>,
    #[serde(serialize_with = "crate::scan_result::optional_icon_bytes")]
    pub crop: Option<OneIconConfig>,
}

// A chest is whatever its tooltip lists. Its title and icon say nothing reliable: chests with one
// name come with different contents, so the title is only kept for display.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chest {
    pub kind: ChestKind,
    pub contents: Vec<ChestContent>,
    pub amount: Option<String>,
    pub tradability: Option<Tradability>,
    pub last_read_title: String,
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

// one row of a chest as strips for the recogniser, with the crop shown when debugging
#[derive(Debug, Clone)]
pub struct ChestRowStrips {
    pub names: Vec<RgbaImage>,
    pub count: RgbaImage,
    pub crop: Option<OneIconConfig>,
}

pub fn chest_strips(buffer: &Buffer, layout: &ChestLayout, debugging: bool) -> Vec<ChestRowStrips> {
    let strip = |(x0, y0, x1, y1): Rect| text_image(buffer, x0, y0, x1, y1);
    layout
        .rows
        .iter()
        .map(|row| {
            let (x0, y0, x1, y1) = row.whole;
            ChestRowStrips {
                names: row.names.iter().map(|x| strip(*x)).collect(),
                count: strip(row.count),
                crop: debugging.then(|| OneIconConfig {
                    data: buffer.crop(x0, y0, x1, y1.min(buffer.height)),
                    name: String::new(),
                    offset: IntegerRectangle {
                        top_left: (x0 as f64, y0 as f64),
                        width: x1 - x0,
                        height: y1.min(buffer.height) - y0,
                    },
                    tag: String::new(),
                    normalized: true,
                    required_confidence: None,
                }),
            }
        })
        .collect()
}

// The known items of a chest and what every row read as, from each row's name, count and crop.
// None when a row might be a known item that did not read.
pub fn chest_from_texts(
    texts: Vec<(String, String, Option<OneIconConfig>)>,
) -> (Option<Vec<ChestContent>>, Vec<ChestRow>) {
    let mut contents = Some(vec![]);
    let mut rows = vec![];
    for (name_read, count_read, crop) in texts {
        let bound = name_read.contains("(Bound");
        let name = name_read.split("(Bound").next().unwrap().trim();
        let item = match_title(name)
            .or_else(|| match_rough_title(name))
            .and_then(|item| item.title.clone());
        // rows of things we do not know are ignored, so their count is not worth reading
        let count_read = item.as_ref().map(|_| count_read).unwrap_or_default();
        // almost a known title is more likely a misread than something else, so this frame says nothing
        if item.is_none() && near_title(name) {
            contents = None;
        }
        if let Some(item) = &item {
            let digits: String = count_read.chars().filter(char::is_ascii_digit).collect();
            match (digits.parse(), &mut contents) {
                (Ok(amount), Some(contents)) => contents.push(ChestContent {
                    item: item.clone(),
                    amount,
                    bound,
                }),
                _ => contents = None,
            }
        }
        rows.push(ChestRow {
            name_read,
            count_read,
            item,
            crop,
        });
    }
    (contents, rows)
}

impl Chest {
    // a brief hover can end before the amount or the bind line was read, which then matches anything
    fn same_contents(&self, other: &Chest) -> bool {
        fn agree<T: PartialEq>(a: &Option<T>, b: &Option<T>) -> bool {
            a.is_none() || b.is_none() || a == b
        }
        self.column == other.column
            && self.kind == other.kind
            && self.contents == other.contents
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
