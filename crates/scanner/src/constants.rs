use std::sync::LazyLock;

use ahash::AHashMap;

use crate::{
    image_utils::common::Rect,
    scanner_state::{
        AnchorType,
        InventoryType::{self, CharInventory},
        SlotAddress,
    },
};

fn generate_slot_grid(
    inventory_type: InventoryType,
    page_num: usize,
    top_left: (f64, f64),
    width: f64,
    height: f64,
    h_gap: f64,
    v_gap: f64,
    num_rows: usize,
    num_cols: usize,
) -> AHashMap<SlotAddress, Rect> {
    let mut map = AHashMap::with_capacity(num_rows * num_cols);

    let (start_x, start_y) = top_left;
    let x_step = width as f64 + h_gap;
    let y_step = height as f64 + v_gap;

    for row in 0..num_rows {
        for col in 0..num_cols {
            let slot = SlotAddress {
                inventory_type,
                page_num,
                pos_in_inv: (row, col),
            };

            let pos = Rect::ui(
                (start_x + col as f64 * x_step, start_y + row as f64 * y_step),
                width,
                height,
            );

            map.insert(slot, pos);
        }
    }

    map
}

pub const MARGIN: f64 = 3.0;
pub const NUMBER_TOP_MARGIN: f64 = 4.0;
pub const NUMBER_HEIGHT: f64 = 18.0;
pub const NUMBER_BOTTOM_MARGIN: f64 = 0.0;
pub const COMBINED_NUMBER_HEIGHT: f64 = NUMBER_TOP_MARGIN + NUMBER_HEIGHT + NUMBER_BOTTOM_MARGIN;
pub const ICON_WIDTH: f64 = 61.0;
pub const ICON_HEIGHT: f64 = 61.0 - COMBINED_NUMBER_HEIGHT;

const GRID_LEFT: f64 = 16.0;
// the two storage windows are narrower than the inventory, and they differ from each other; both
// measured on 1440p stills, where nothing is rounded
const CHAR_STORAGE_GRID_LEFT: f64 = 15.27;
const ROSTER_ONLY_GRID_LEFT: f64 = 14.70;

// every window shares the same grid relative to its own origin, apart from where it starts
fn standard_grid(
    inventory_type: InventoryType,
    page_num: usize,
    left: f64,
    num_rows: usize,
    num_cols: usize,
) -> AHashMap<SlotAddress, Rect> {
    generate_slot_grid(
        inventory_type,
        page_num,
        (left + MARGIN, 105.25 + MARGIN + COMBINED_NUMBER_HEIGHT),
        ICON_WIDTH,
        ICON_HEIGHT,
        2.75 + MARGIN * 2.0,
        2.75 + MARGIN * 2.0 + COMBINED_NUMBER_HEIGHT,
        num_rows,
        num_cols,
    )
}

pub static ALL_SLOT_ADDRESSS: LazyLock<AHashMap<SlotAddress, Rect>> =
    LazyLock::new(|| {
        let mut map = AHashMap::new();

        map.extend(standard_grid(CharInventory, 0, GRID_LEFT, 10, 10));
        map.extend(standard_grid(CharInventory, 1, GRID_LEFT, 5, 10));
        for page_num in 0..4 {
            map.extend(standard_grid(
                InventoryType::CharStorage,
                page_num,
                CHAR_STORAGE_GRID_LEFT,
                10,
                10,
            ));
        }
        for page_num in 0..2 {
            map.extend(standard_grid(
                InventoryType::Roster,
                page_num,
                ROSTER_ONLY_GRID_LEFT,
                10,
                6,
            ));
        }

        map
    });

// window, page, row, column: the order slots are looked at
pub static SLOT_ORDER: LazyLock<Vec<SlotAddress>> = LazyLock::new(|| {
    let mut all: Vec<SlotAddress> = ALL_SLOT_ADDRESSS.keys().copied().collect();
    all.sort_by_key(|x| (x.inventory_type as u8, x.page_num, x.pos_in_inv));
    all
});

fn page_names(prefix: &str, num_pages: usize) -> Vec<(String, String)> {
    (1..=num_pages)
        .map(|page| {
            (
                format!("{prefix} page {page} active"),
                format!("{prefix} page {page} inactive"),
            )
        })
        .collect()
}

// page tab offsets are relative to the window origin              active , inactive
pub static ALL_PAGE_NUM: LazyLock<AHashMap<InventoryType, Vec<(String, String)>>> =
    LazyLock::new(|| {
        AHashMap::from([
            (InventoryType::CharInventory, page_names("Char", 2)),
            (InventoryType::CharStorage, page_names("Char storage", 4)),
            (InventoryType::Roster, page_names("Roster", 2)),
        ])
    });

#[derive(Clone, Copy)]
pub enum Bound {
    // the game can sit anywhere in the capture, so search all of it
    Frame,
    // 1440p UI space
    Ui(Rect),
    // 1440p UI space, moved along with the storage layout
    Storage(Rect),
}

pub struct AnchorVariant {
    pub name: &'static str,
    pub bound: Bound,
    // Setting from the matched patch's mean, fitted to the captures in scripts/brightness/inputs*
    // (1080p). None for thin icons: the game draws them anew at each resolution, and their mean
    // comes out 9 settings apart between 1080p and 1440p.
    pub brightness: Option<[f64; 3]>,
}

pub struct AnchorSpec {
    pub anchor_type: AnchorType,
    pub variants: Vec<AnchorVariant>,
    // window origin of each inventory this anchor locates, relative to the anchor root
    pub inventories: Vec<(InventoryType, (f64, f64))>,
}

// where a template is expected, with some slack on every side
fn around(top_left: (f64, f64), width: f64, height: f64, slack: f64) -> Rect {
    Rect::ui(
        (top_left.0 - slack, top_left.1 - slack),
        width + slack * 2.0,
        height + slack * 2.0,
    )
}

const SORT_BUTTON: &str = "Char Inventory Anchor 1";
const SORT_BUTTON_CURVE: [f64; 3] = [5.0512534143e-03, 0.7490040658, -36.4018862];
// the inventory's other icons up there are its filters, and the chosen one lights up
const SEARCH_BUTTON: &str = "Inventory search button";
const SEARCH_BUTTON_CURVE: [f64; 3] = [3.6937751239e-03, 1.2577411327, -24.5035260];
const STORAGE_SLACK: f64 = 2.0;
pub const WINDOW_SLACK: f64 = 6.0;
// The storage layout is three windows and the Storage button, always in the same place: one for
// the pet menu, and shifted by this for the storage NPC. It is open while this many of its seven
// anchors are found, so the cursor or a tooltip covering a few of them changes nothing.
pub const STORAGE_SHIFTS: [(f64, f64); 2] = [(0.0, 0.0), (-6.4, -142.45)];
pub const STORAGE_MIN: usize = 3;

// one window of the storage layout: the sort button at its top left and the icons at its top right
fn storage_window(
    anchor_type: AnchorType,
    inventory_type: InventoryType,
    origin: (f64, f64),
    other_name: &'static str,
    other_offset: Rect,
    other_brightness: Option<[f64; 3]>,
) -> AnchorSpec {
    let relative = |offset: (f64, f64), width: f64, height: f64| {
        Bound::Storage(around(
            (origin.0 + offset.0, origin.1 + offset.1),
            width,
            height,
            WINDOW_SLACK,
        ))
    };
    AnchorSpec {
        anchor_type,
        variants: vec![
            AnchorVariant {
                name: SORT_BUTTON,
                bound: relative((20.0, 60.0), 38.0, 38.0),
                brightness: Some(SORT_BUTTON_CURVE),
            },
            AnchorVariant {
                name: other_name,
                bound: relative(
                    other_offset.top_left,
                    other_offset.width,
                    other_offset.height,
                ),
                brightness: other_brightness,
            },
        ],
        inventories: vec![(inventory_type, (0.0, 0.0))],
    }
}

// every one but the last belongs to the storage layout
pub static ANCHORS: LazyLock<Vec<AnchorSpec>> = LazyLock::new(|| {
    vec![
        // the highlighted Storage button, which sits elsewhere for the NPC but does not move with the windows
        AnchorSpec {
            anchor_type: AnchorType::Storage,
            variants: vec![
                AnchorVariant {
                    name: "Storage anchor pet",
                    bound: Bound::Ui(around((655.0, 1280.0), 120.0, 50.0, STORAGE_SLACK)),
                    // no captures of the pet menu across settings, so this one is from the model
                    brightness: Some([5.7877893407e-03, 0.5554276781, -36.8317787]),
                },
                AnchorVariant {
                    name: "Storage anchor npc",
                    bound: Bound::Ui(around((1215.0, 1100.0), 120.0, 50.0, STORAGE_SLACK)),
                    brightness: Some([5.4345973311e-03, 0.5303418622, -30.5668870]),
                },
            ],
            inventories: vec![],
        },
        storage_window(
            AnchorType::StorageRoster,
            InventoryType::Roster,
            (332.6, 290.7),
            "Roster top right icons",
            Rect::ui((359.4, 63.3), 76.0, 36.0),
            None,
        ),
        storage_window(
            AnchorType::StorageCharStorage,
            InventoryType::CharStorage,
            (778.7, 290.7),
            "Char storage top right icons",
            Rect::ui((509.3, 63.3), 208.0, 36.0),
            None,
        ),
        storage_window(
            AnchorType::StorageInventory,
            InventoryType::CharInventory,
            (1504.6, 290.7),
            SEARCH_BUTTON,
            Rect::ui((662.0, 56.0), 48.0, 48.0),
            Some(SEARCH_BUTTON_CURVE),
        ),
        AnchorSpec {
            anchor_type: AnchorType::CharInventory,
            variants: vec![
                AnchorVariant {
                    name: SORT_BUTTON,
                    bound: Bound::Frame,
                    brightness: Some(SORT_BUTTON_CURVE),
                },
                AnchorVariant {
                    name: SEARCH_BUTTON,
                    bound: Bound::Frame,
                    brightness: Some(SEARCH_BUTTON_CURVE),
                },
            ],
            inventories: vec![(InventoryType::CharInventory, (0.0, 0.0))],
        },
    ]
});

pub fn anchor_spec(anchor_type: AnchorType) -> &'static AnchorSpec {
    ANCHORS
        .iter()
        .find(|spec| spec.anchor_type == anchor_type)
        .unwrap()
}

pub const NUMBER_OFFSET: Rect =
    Rect::ui((0.0, -NUMBER_HEIGHT - NUMBER_BOTTOM_MARGIN), 61.0, NUMBER_HEIGHT);
