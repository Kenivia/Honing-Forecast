use std::sync::OnceLock;

use ahash::AHashMap;
use fast_image_resize::images::Image;
use serde::{Deserialize, Serialize};

use crate::scanner_state::{ScaledPosition, ScannerState};

pub static CONFIG: OnceLock<AHashMap<String, OneIconConfig>> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OneIconConfig {
    pub data: Vec<u8>,
    pub name: String,
    pub position: ScaledPosition,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncomingNewIcon {
    pub position: ScaledPosition,
    pub name: String,
    pub tag: String,
}
impl ScannerState {
    pub fn setup(&mut self) {
        if self.incoming_new_icon.is_some() {
            let incoming: IncomingNewIcon = self.incoming_new_icon.clone().unwrap();
            let input: Vec<ScaledPosition> = vec![incoming.position];
            let data: Vec<u8> = self
                .downscale(&input)
                .into_iter()
                .flat_map(Image::into_vec)
                .collect();

            self.config.insert(
                0,
                OneIconConfig {
                    data,
                    name: incoming.name,
                    position: incoming.position,
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
        CONFIG.set(new).expect("alr set config")
    }
}
