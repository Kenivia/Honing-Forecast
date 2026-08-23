use std::sync::LazyLock;

use ahash::AHashMap;

use crate::scanner_state::{
    InventoryType::{self, CharInventory},
    Rectangle, SlotAddress,
};

fn generate_slot_grid(
    inventory_type: InventoryType,
    page_num: usize,
    top_left: (f64, f64),
    width: usize,
    height: usize,
    h_gap: f64,
    v_gap: f64,
    num_rows: usize,
    num_cols: usize,
) -> AHashMap<SlotAddress, Rectangle> {
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

            let pos = Rectangle {
                top_left: (start_x + col as f64 * x_step, start_y + row as f64 * y_step),
                width,
                height,
            };

            map.insert(slot, pos);
        }
    }

    map
}

pub static ALL_SLOT_ADDRESSS: LazyLock<AHashMap<SlotAddress, Rectangle>> = LazyLock::new(|| {
    let mut map = AHashMap::new();

    map.extend(generate_slot_grid(
        CharInventory,
        0,
        (11.0, 64.0),
        28,
        19,
        6.8888888,     // h_gap
        10.0 + 5.8889, // v_gap
        10,            // num_rows
        10,            // num_cols
    ));

    map.extend(generate_slot_grid(
        CharInventory,
        1,
        (11.0, 64.0),
        28,
        19,
        6.8888888,     // h_gap
        10.0 + 5.8889, // v_gap
        5,
        10,
    ));

    map.extend(generate_slot_grid(
        InventoryType::CharStorage,
        1,
        (11.0, 64.0),
        28,
        19,
        6.8888888,     // h_gap
        10.0 + 5.8889, // v_gap
        10,
        10,
    ));

    map.extend(generate_slot_grid(
        InventoryType::CharStorage,
        2,
        (11.0, 64.0),
        28,
        19,
        6.8888888,     // h_gap
        10.0 + 5.8889, // v_gap
        10,
        10,
    ));

    map.extend(generate_slot_grid(
        InventoryType::CharStorage,
        3,
        (11.0, 64.0),
        28,
        19,
        6.8888888,     // h_gap
        10.0 + 5.8889, // v_gap
        10,
        10,
    ));

    map.extend(generate_slot_grid(
        InventoryType::CharStorage,
        4,
        (11.0, 64.0),
        28,
        19,
        6.8888888,     // h_gap
        10.0 + 5.8889, // v_gap
        10,
        10,
    ));

    map.extend(generate_slot_grid(
        InventoryType::Roster,
        1,
        (11.0, 64.0),
        28,
        19,
        6.8888888,     // h_gap
        10.0 + 5.8889, // v_gap
        6,
        10,
    ));
    map.extend(generate_slot_grid(
        InventoryType::Roster,
        2,
        (11.0, 64.0),
        28,
        19,
        6.8888888,     // h_gap
        10.0 + 5.8889, // v_gap
        6,
        10,
    ));

    map
});

//                                                             true   , false
pub static ALL_PAGE_NUM: LazyLock<AHashMap<InventoryType, Vec<(String, String)>>> =
    LazyLock::new(|| {
        let mut map = AHashMap::from([(
            InventoryType::CharInventory,
            vec![
                (
                    "Char page 1 active".to_string(),
                    "Char page 1 inactive".to_string(),
                ),
                (
                    "Char page 2 active".to_string(),
                    "Char page 2 inactive".to_string(),
                ),
            ],
        )]);
        map.extend(AHashMap::from([(
            InventoryType::CharStorage,
            vec![
                (
                    "Char page 1 active".to_string(),
                    "Char page 1 inactive".to_string(),
                ),
                (
                    "Char page 1 active".to_string(),
                    "Char page 1 inactive".to_string(),
                ),
                (
                    "Char page 1 active".to_string(),
                    "Char page 1 inactive".to_string(),
                ),
                (
                    "Char page 1 active".to_string(),
                    "Char page 1 inactive".to_string(),
                ),
            ],
        )]));
        map.extend(AHashMap::from([(
            InventoryType::Roster,
            vec![
                (
                    "Char page 1 active".to_string(),
                    "Char page 1 inactive".to_string(),
                ),
                (
                    "Char page 1 active".to_string(),
                    "Char page 1 inactive".to_string(),
                ),
            ],
        )]));
        map
    });

pub static ANCHORS_LOOKUP: LazyLock<AHashMap<InventoryType, Vec<(String, Option<Rectangle>)>>> =
    LazyLock::new(|| {
        let mut map = AHashMap::from([(
            InventoryType::CharInventory,
            vec![
                ("Char Inventory anchor 1 ".to_string(), None),
                ("Char Inventory anchor 2".to_string(), None),
            ],
        )]);
        map.extend(AHashMap::from([(
            InventoryType::CharStorage,
            vec![
                ("Char Inventory anchor 1 ".to_string(), None),
                ("Char Inventory anchor 2".to_string(), None),
            ],
        )]));
        map.extend(AHashMap::from([(
            InventoryType::Roster,
            vec![
                ("Char Inventory anchor 1 ".to_string(), None),
                ("Char Inventory anchor 2".to_string(), None),
            ],
        )]));
        map
    });
pub const NUMBER_OFFSET: Rectangle = Rectangle {
    top_left: (0.0, -8.0),
    width: 28,
    height: 9,
};
pub const TARGET_RESOLUTION: (usize, usize) = (1280, 720);
