use std::sync::LazyLock;

use ahash::AHashMap;

use crate::{
    image_utils::common::FloatRectangle,
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
) -> AHashMap<SlotAddress, FloatRectangle> {
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

            let pos = FloatRectangle {
                top_left: (start_x + col as f64 * x_step, start_y + row as f64 * y_step),
                width,
                height,
            };

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
const ROSTER_GRID_LEFT: f64 = 15.1; // the roster window is narrower, so its grid sits closer to its left edge

// every window shares the same grid relative to its own origin, apart from where it starts
fn standard_grid(
    inventory_type: InventoryType,
    page_num: usize,
    left: f64,
    num_rows: usize,
    num_cols: usize,
) -> AHashMap<SlotAddress, FloatRectangle> {
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

pub static ALL_SLOT_ADDRESSS: LazyLock<AHashMap<SlotAddress, FloatRectangle>> =
    LazyLock::new(|| {
        let mut map = AHashMap::new();

        map.extend(standard_grid(CharInventory, 0, GRID_LEFT, 10, 10));
        map.extend(standard_grid(CharInventory, 1, GRID_LEFT, 5, 10));
        for page_num in 0..4 {
            map.extend(standard_grid(
                InventoryType::CharStorage,
                page_num,
                ROSTER_GRID_LEFT,
                10,
                10,
            ));
        }
        for page_num in 0..2 {
            map.extend(standard_grid(
                InventoryType::Roster,
                page_num,
                ROSTER_GRID_LEFT,
                10,
                6,
            ));
        }

        map
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
    Ui(FloatRectangle),
    // relative to another anchor's root, which must be found first
    Relative(AnchorType, FloatRectangle),
}

pub struct AnchorVariant {
    pub name: &'static str,
    pub bound: Bound,
    // moves the root when a variant shows the same layout somewhere else
    pub root_shift: (f64, f64),
    // from scripts/brightness/calibrate_anchor.py
    pub brightness: [f64; 3],
}

pub struct AnchorSpec {
    pub anchor_type: AnchorType,
    pub variants: Vec<AnchorVariant>,
    // while this anchor is found, these are cleared and not searched
    pub forbids: Vec<AnchorType>,
    // window origin of each inventory this anchor locates, relative to the anchor root
    pub inventories: Vec<(InventoryType, (f64, f64))>,
}

// where a template is expected, with some slack on every side
fn around(top_left: (f64, f64), width: f64, height: f64, slack: f64) -> FloatRectangle {
    FloatRectangle {
        top_left: (top_left.0 - slack, top_left.1 - slack),
        width: width + slack * 2.0,
        height: height + slack * 2.0,
    }
}

const SORT_BUTTON: &str = "Char Inventory Anchor 1";
const SORT_BUTTON_CURVE: [f64; 3] = [5.1915080096e-03, 0.7677018683, -40.0652762];
const INVENTORY_BOTTOM: &str = "Char Inventory Anchor 2";
const INVENTORY_BOTTOM_CURVE: [f64; 3] = [3.3969098428e-03, 1.0041463049, -24.1746495];
const STORAGE_SLACK: f64 = 2.0;
const WINDOW_SLACK: f64 = 4.0;
const MOVE_ALL_CURVE: [f64; 3] = [4.1848223556e-03, 1.0029699644, -35.7747665];

// one window of the storage layout: the sort button plus one other piece of its chrome,
// both looked for around where the storage anchor says the window is
fn storage_window(
    anchor_type: AnchorType,
    inventory_type: InventoryType,
    origin: (f64, f64),
    other_name: &'static str,
    other_offset: FloatRectangle,
    other_brightness: [f64; 3],
) -> AnchorSpec {
    let relative = |offset: (f64, f64), width: f64, height: f64| {
        Bound::Relative(
            AnchorType::Storage,
            around(
                (origin.0 + offset.0, origin.1 + offset.1),
                width,
                height,
                WINDOW_SLACK,
            ),
        )
    };
    AnchorSpec {
        anchor_type,
        variants: vec![
            AnchorVariant {
                name: SORT_BUTTON,
                bound: relative((20.0, 60.0), 38.0, 38.0),
                root_shift: (0.0, 0.0),
                brightness: SORT_BUTTON_CURVE,
            },
            AnchorVariant {
                name: other_name,
                bound: relative(
                    other_offset.top_left,
                    other_offset.width,
                    other_offset.height,
                ),
                root_shift: (0.0, 0.0),
                brightness: other_brightness,
            },
        ],
        forbids: vec![],
        inventories: vec![(inventory_type, (0.0, 0.0))],
    }
}

// checked in this order
pub static ANCHORS: LazyLock<Vec<AnchorSpec>> = LazyLock::new(|| {
    vec![
        // only says the storage layout is open and roughly where, the windows have their own anchors
        AnchorSpec {
            anchor_type: AnchorType::Storage,
            variants: vec![
                AnchorVariant {
                    name: "Storage anchor pet",
                    bound: Bound::Ui(around((655.0, 1280.0), 120.0, 50.0, STORAGE_SLACK)),
                    root_shift: (0.0, 0.0),
                    brightness: [5.7877893407e-03, 0.5554276781, -36.8317787],
                },
                AnchorVariant {
                    name: "Storage anchor npc",
                    bound: Bound::Ui(around((1215.0, 1100.0), 120.0, 50.0, STORAGE_SLACK)),
                    root_shift: (-6.4, -142.45),
                    brightness: [5.8147377428e-03, 0.5543038280, -36.7024555],
                },
            ],
            forbids: vec![AnchorType::CharInventory],
            inventories: vec![],
        },
        storage_window(
            AnchorType::StorageRoster,
            InventoryType::Roster,
            (332.6, 290.7),
            "Roster move all duplicates",
            FloatRectangle {
                top_left: (91.4, 809.3),
                width: 264.0,
                height: 36.0,
            },
            MOVE_ALL_CURVE,
        ),
        storage_window(
            AnchorType::StorageCharStorage,
            InventoryType::CharStorage,
            (778.7, 290.7),
            "Char storage move all duplicates",
            FloatRectangle {
                top_left: (233.3, 809.3),
                width: 264.0,
                height: 36.0,
            },
            MOVE_ALL_CURVE,
        ),
        storage_window(
            AnchorType::StorageInventory,
            InventoryType::CharInventory,
            (1504.6, 290.7),
            // greyed out here, so not the same template as the char inventory one
            "Storage inventory anchor 2",
            FloatRectangle {
                top_left: (22.4, 806.3),
                width: 80.0,
                height: 42.0,
            },
            [-5.3541878546e-04, 1.8218981042, -18.4227396],
        ),
        AnchorSpec {
            anchor_type: AnchorType::CharInventory,
            variants: vec![
                AnchorVariant {
                    name: SORT_BUTTON,
                    bound: Bound::Frame,
                    root_shift: (0.0, 0.0),
                    brightness: SORT_BUTTON_CURVE,
                },
                AnchorVariant {
                    name: INVENTORY_BOTTOM,
                    bound: Bound::Frame,
                    root_shift: (0.0, 0.0),
                    brightness: INVENTORY_BOTTOM_CURVE,
                },
            ],
            forbids: vec![],
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

pub const NUMBER_OFFSET: FloatRectangle = FloatRectangle {
    top_left: (0.0, -NUMBER_HEIGHT - NUMBER_BOTTOM_MARGIN),
    width: 61.0,
    height: NUMBER_HEIGHT,
};
