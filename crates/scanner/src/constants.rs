use std::sync::LazyLock;

use ahash::AHashMap;

use crate::{
    image_utils::common::FloatRectangle,
    scanner_state::{
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
pub const NUMBER_HEIGHT: f64 = 22.0;
pub const ICON_WIDTH: f64 = 61.0;
pub const ICON_HEIGHT: f64 = 61.0 - NUMBER_HEIGHT;

pub static ALL_SLOT_ADDRESSS: LazyLock<AHashMap<SlotAddress, FloatRectangle>> =
    LazyLock::new(|| {
        let mut map = AHashMap::new();

        map.extend(generate_slot_grid(
            CharInventory,
            0,
            (16.0 + MARGIN, 105.33333 + MARGIN + NUMBER_HEIGHT),
            ICON_WIDTH,
            ICON_HEIGHT,
            2.7777777 + MARGIN * 2.0,
            2.7777777 + MARGIN * 2.0 + NUMBER_HEIGHT,
            10,
            10,
        ));

        map.extend(generate_slot_grid(
            CharInventory,
            1,
            (16.0 + MARGIN, 105.33333 + MARGIN + NUMBER_HEIGHT),
            ICON_WIDTH,
            ICON_HEIGHT,
            2.7777777 + MARGIN * 2.0,
            2.7777777 + MARGIN * 2.0 + NUMBER_HEIGHT,
            5,
            10,
        ));

        map.extend(generate_slot_grid(
            InventoryType::CharStorage,
            1,
            (16.0 + MARGIN, 105.33333 + MARGIN + NUMBER_HEIGHT),
            ICON_WIDTH,
            ICON_HEIGHT,
            2.7777777 + MARGIN * 2.0,
            2.7777777 + MARGIN * 2.0 + NUMBER_HEIGHT,
            10,
            10,
        ));

        map.extend(generate_slot_grid(
            InventoryType::CharStorage,
            2,
            (16.0 + MARGIN, 105.33333 + MARGIN + NUMBER_HEIGHT),
            ICON_WIDTH,
            ICON_HEIGHT,
            2.7777777 + MARGIN * 2.0,
            2.7777777 + MARGIN * 2.0 + NUMBER_HEIGHT,
            10,
            10,
        ));

        map.extend(generate_slot_grid(
            InventoryType::CharStorage,
            3,
            (16.0 + MARGIN, 105.33333 + MARGIN + NUMBER_HEIGHT),
            ICON_WIDTH,
            ICON_HEIGHT,
            2.7777777 + MARGIN * 2.0,
            2.7777777 + MARGIN * 2.0 + NUMBER_HEIGHT,
            10,
            10,
        ));

        map.extend(generate_slot_grid(
            InventoryType::CharStorage,
            4,
            (16.0 + MARGIN, 105.33333 + MARGIN + NUMBER_HEIGHT),
            ICON_WIDTH,
            ICON_HEIGHT,
            2.7777777 + MARGIN * 2.0,
            2.7777777 + MARGIN * 2.0 + NUMBER_HEIGHT,
            10,
            10,
        ));

        map.extend(generate_slot_grid(
            InventoryType::Roster,
            1,
            (16.0 + MARGIN, 105.33333 + MARGIN + NUMBER_HEIGHT),
            ICON_WIDTH,
            ICON_HEIGHT,
            2.7777777 + MARGIN * 2.0,
            2.7777777 + MARGIN * 2.0 + NUMBER_HEIGHT,
            6,
            10,
        ));
        map.extend(generate_slot_grid(
            InventoryType::Roster,
            2,
            (16.0 + MARGIN, 105.33333 + MARGIN + NUMBER_HEIGHT),
            ICON_WIDTH,
            ICON_HEIGHT,
            2.7777777 + MARGIN * 2.0,
            2.7777777 + MARGIN * 2.0 + NUMBER_HEIGHT,
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

pub static ANCHORS_LOOKUP: LazyLock<
    AHashMap<InventoryType, Vec<(String, Option<FloatRectangle>)>>,
> = LazyLock::new(|| {
    let mut map = AHashMap::from([(
        InventoryType::CharInventory,
        vec![
            ("Char Inventory Anchor 1".to_string(), None),
            ("Char Inventory Anchor 2".to_string(), None),
        ],
    )]);
    map.extend(AHashMap::from([(
        InventoryType::CharStorage,
        vec![
            ("Char Inventory Anchor 1".to_string(), None),
            ("Char Inventory Anchor 2".to_string(), None),
        ],
    )]));
    map.extend(AHashMap::from([(
        InventoryType::Roster,
        vec![
            ("Char Inventory Anchor 1".to_string(), None),
            ("Char Inventory Anchor 2".to_string(), None),
        ],
    )]));
    map
});
pub const NUMBER_OFFSET: FloatRectangle = FloatRectangle {
    top_left: (0.0, -NUMBER_HEIGHT),
    width: 61.0,
    height: NUMBER_HEIGHT,
};
