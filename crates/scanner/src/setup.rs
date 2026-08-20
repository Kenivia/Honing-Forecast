use crate::{
    image_utils::{common::get_resizer, downscale::crop_buffer},
    scanner_state::{ScaledPosition, ScannerState},
};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub static CONFIG: OnceLock<AHashMap<String, OneIconConfig>> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OneIconConfig {
    pub data: Vec<u8>,
    pub name: String,
    pub offset: ScaledPosition, // naming it like this to distinguish from like actual absolute positions
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncomingNewIcon {
    pub position: ScaledPosition,
    pub name: String,
    pub tag: String,
}

pub fn icon_lookup(name: &String) -> &OneIconConfig {
    // TODO DOWNSCALE HERE ?
    CONFIG.get().unwrap().get(name).unwrap()
}
impl ScannerState {
    pub fn setup(&mut self) {
        if self.incoming_new_icon.is_some() {
            let incoming: IncomingNewIcon = self.incoming_new_icon.clone().unwrap();
            let data: Vec<u8> = crop_buffer(
                incoming.position,
                get_resizer(&mut self.resizer),
                self.buffer,
            )
            .into_vec();

            self.config.insert(
                0,
                OneIconConfig {
                    data,
                    name: incoming.name,
                    offset: incoming.position,
                    tag: incoming.tag,
                },
            );
            self.incoming_new_icon = None;
        }
    }

    pub fn set_config(&mut self) {
        // assert!(self.config.len() > 0);
        let mut new = AHashMap::with_capacity(self.config.len());
        for i in self.config.iter() {
            new.insert(i.name.clone(), i.clone());
        }
        CONFIG.set(new).expect("alr set config");
        self.config = vec![]; // no need to pass in and out after setting 
    }
}
