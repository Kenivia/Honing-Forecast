use ahash::AHashMap;
use hf_core::my_dbg;

use crate::{
    constants::ALL_PAGE_NUM,
    scanner_state::{InventoryType, ScannerState},
    setup::icon_lookup,
};

impl ScannerState {
    pub fn initialize_page_num_infos(&mut self) {
        self.page_num_infos = AHashMap::new();
        for inv_type in ALL_PAGE_NUM.keys() {
            self.page_num_infos
                .insert(*inv_type, vec![None; ALL_PAGE_NUM[inv_type].len()]);
        }
    }

    pub fn update_page_status(&mut self) {
        for inv_type in ALL_PAGE_NUM.keys() {
            if !self.anchors[inv_type].is_found() {
                *self.page_num_infos.get_mut(&inv_type).unwrap() =
                    vec![None; self.page_num_infos[inv_type].len()];
                continue;
            }

            for (index, (active_name, inactive_name)) in ALL_PAGE_NUM[inv_type].iter().enumerate() {
                let check = |name: &String| {
                    let icon = icon_lookup(name);
                    self.images_close_enough(
                        icon,
                        self.downscale(icon.offset + self.anchors[inv_type].position_root.unwrap()),
                    )
                    .is_some()
                };

                let matched = check(&active_name) || check(&inactive_name);
                self.page_num_infos.get_mut(&inv_type).unwrap()[index] = matched.then_some(true);
                my_dbg!( inv_type, index, matched);
            }
        }
    }
    pub fn active_page_num(&self) -> AHashMap<InventoryType, Option<usize>> {
        let mut out: AHashMap<InventoryType, Option<usize>> = AHashMap::new();
        for inv_type in ALL_PAGE_NUM.keys() {
            let infos: Option<&Vec<Option<bool>>> = self.page_num_infos.get(inv_type);
            if infos.is_none() {
                out.insert(*inv_type, None);
                continue;
            }
            if let Some(idx) = infos.unwrap().iter().position(|state| *state == Some(true)) {
                out.insert(*inv_type, Some(idx));
                continue;
            }

            let mut unsure_iter = infos
                .unwrap()
                .iter()
                .enumerate()
                .filter(|(_, state)| state.is_none());

            let first_unsure: Option<(usize, &Option<bool>)> = unsure_iter.next();
            if first_unsure.is_none() || unsure_iter.next().is_some() {
                // More than one unsure
                out.insert(*inv_type, None);
                continue;
            }

            out.insert(*inv_type, Some(first_unsure.unwrap().0));
        }
        out
    }
}
