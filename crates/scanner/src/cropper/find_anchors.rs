use crate::{
    scanner_state::{InventoryType, ScannerState},
    setup::CONFIG,
};

impl ScannerState {
    pub fn validate_anchors(&mut self, inv_type: InventoryType) {
        if self.anchors.contains_key(&inv_type)
            && !self.images_close_enough(
                &CONFIG.get().unwrap()[&self.anchors[&inv_type].name],
                self.anchors[&inv_type].position,
            )
        {
            self.anchors.remove(&inv_type);
        }

        // TODO how reliable is using inv_type as an index? maybe just use usize
        if !self.anchors.contains_key(&inv_type) {
            if let Some((variant, (position, icon_id))) = ALL_ANCHOR_TEMPLATES[inv_type as usize]
                .iter()
                .enumerate()
                .find_map(|(variant, &icon_id)| {
                    self.template_match(
                        self.access_icon_id(icon_id),
                        Some(ANCHOR_BOUNDS[inv_type as usize]),
                    )
                    .map(|position| (variant, (position, icon_id)))
                })
            {
                self.anchors.insert(
                    inv_type,
                    OneAnchorInfo {
                        variant,
                        position,
                        id: icon_id,
                    },
                );
            }
        }
    }
}
