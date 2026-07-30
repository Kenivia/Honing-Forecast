use std::sync::LazyLock;

use ahash::AHashMap;

use crate::scanner_state::{
    InventoryType::{self, CharInventory},
    ScaledPosition, SlotAddress,
};

pub static ANCHORS_LOOKUP: LazyLock<
    AHashMap<InventoryType, Vec<(String, Option<ScaledPosition>)>>,
> = LazyLock::new(|| {
    AHashMap::from([(
        InventoryType::CharInventory,
        vec![
            ("Sort button".to_string(), None),
            ("Dismantle button".to_string(), None),
        ],
    )])
});

//TODO generate this properly
pub static ALL_SLOT_ADDRESSS: LazyLock<AHashMap<SlotAddress, ScaledPosition>> =
    LazyLock::new(|| {
        AHashMap::from([(
            SlotAddress {
                inventory_type: CharInventory,
                page_num: 0,
                pos_in_inv: (0, 0),
            },
            ScaledPosition {
                top_left: (0.0, 0.0),
                width: 0,
                height: 0,
            },
        )])
    });

//                                                             true   , false
pub static ALL_PAGE_NUM: LazyLock<AHashMap<InventoryType, Vec<(String, String)>>> =
    LazyLock::new(|| {
        AHashMap::from([(
            InventoryType::CharInventory,
            vec![(
                "Char inventory page 1 active".to_string(),
                "Char inventory page 1 inactive".to_string(),
            )],
        )])
    });


pub const NUMBER_OFFSET: ScaledPosition = ScaledPosition {
    top_left: (0.0, -6.7),
    width: 6,
    height: 7,
};
pub const TARGET_RESOLUTION: (usize, usize) = (1280, 720);
