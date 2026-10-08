use ahash::AHashMap;

use crate::{
    constants::ALL_PAGE_NUM,
    image_utils::{
        close_enough::{DEFAULT_CONFIDENCE, confidence},
        common::{Rectangle, get_resizer},
        resize::crop_buffer,
    },
    scanner_state::{InventoryType, ScannerState},
    setup::icon_lookup,
};

const TAB_LEAD: f64 = 0.1; // an active tab leads by 0.2, one under the cursor by 0.05

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
            let Some(root) = self.inventory_root(*inv_type) else {
                *self.page_num_infos.get_mut(&inv_type).unwrap() =
                    vec![None; ALL_PAGE_NUM[inv_type].len()];
                continue;
            };

            for (index, (active_name, inactive_name)) in ALL_PAGE_NUM[inv_type].iter().enumerate() {
                let mut score = |name: &String| {
                    let icon = icon_lookup(
                        name,
                        self.screen_info.effective_height,
                        get_resizer(&mut self.resizer),
                    );
                    let seen = confidence(
                        &icon,
                        &mut crop_buffer(
                            icon.offset.use_root(&root),
                            get_resizer(&mut self.resizer),
                            self.buffer,
                            None,
                        ),
                        self.screen_info.brightness.unwrap(),
                    );
                    (seen.unwrap_or(0.0), icon.required_confidence.unwrap_or(DEFAULT_CONFIDENCE))
                };
                // A tab under the cursor lights up and looks a little like both states (0.83 active,
                // 0.78 inactive), so a state has to pass and be clearly the better of the two.
                let ((active, active_limit), (inactive, inactive_limit)) =
                    (score(active_name), score(inactive_name));
                let state = if active > active_limit && active - inactive > TAB_LEAD {
                    Some(true)
                } else if inactive > inactive_limit && inactive - active > TAB_LEAD {
                    Some(false)
                } else {
                    None
                };
                self.page_num_infos.get_mut(&inv_type).unwrap()[index] = state;
            }
        }
    }

    // The page whose tab is lit, when exactly one is. A tab that is neither lit nor dark says
    // nothing: guessing it is the active one put a page's slots on the page whose tab the cursor
    // was over, while the label it brings up hid the lit one.
    pub fn active_page_num(&self) -> AHashMap<InventoryType, Option<usize>> {
        ALL_PAGE_NUM
            .keys()
            .map(|inv_type| {
                let mut lit = self.page_num_infos[inv_type]
                    .iter()
                    .enumerate()
                    .filter(|(_, state)| **state == Some(true));
                let page = lit.next().filter(|_| lit.next().is_none()).map(|x| x.0);
                (*inv_type, page)
            })
            .collect()
    }
}
