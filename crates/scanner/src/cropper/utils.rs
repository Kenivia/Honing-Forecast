use crate::scanner_state::{InventoryType, ScaledPosition, ScannerState, SlotAddress};


impl ScannerState {
    pub fn active_page_num(&self, inv_type: InventoryType) -> Option<usize> {
        if self.anchors.contains_key(&inv_type) {
            Some(self.anchors[&inv_type].variant)
        } else {
            None
        }
    }

    // given the anchor found, find where this icon should be
    pub fn anchored_position(&self, slot_address: SlotAddress) -> ScaledPosition {
        assert!(self.anchors.contains_key(&slot_address.inventory_type));
        ScaledPosition {
            top_left: (0.0, 0.0),
            width: 0,
            height: 0,
        }
        //
    }
}
