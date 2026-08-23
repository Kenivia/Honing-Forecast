use crate::{
    image_utils::{common::get_resizer, downscale::crop_buffer},
    scanner_state::{Rectangle, ScannerState},
};
use ahash::AHashMap;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub static CONFIG: LazyLock<RwLock<AHashMap<(String, usize), OneIconConfig>>> =
    LazyLock::new(|| RwLock::new(AHashMap::new()));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OneIconConfig {
    pub data: Vec<u8>,
    pub name: String,
    pub offset: Rectangle, // naming it like this to distinguish from like actual absolute positions
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncomingNewIcon {
    pub position: Rectangle,
    pub name: String,
    pub tag: String,
}

pub fn icon_lookup(name: &str, resolution: usize) -> MappedRwLockReadGuard<'static, OneIconConfig> {
    let key = (name.to_string(), resolution);

    if let Some(guard) = CONFIG.try_read() {
        if guard.contains_key(&key) {
            return RwLockReadGuard::map(guard, |map| &map[&key]);
        }
    }

    let mut write_guard = CONFIG.write();
    write_guard
        .entry(key.clone())
        .or_insert_with(|| OneIconConfig {
            data: vec![],
            name: String::new(),
            offset: Rectangle {
                top_left: (0.0, 0.0),
                width: 0,
                height: 0,
            },
            tag: String::new(),
        });

    let read_guard = RwLockWriteGuard::downgrade(write_guard);
    RwLockReadGuard::map(read_guard, move |map| &map[&key])
}

impl ScannerState {
    pub fn setup(&mut self) {
        if self.incoming_new_icons.is_some() {
            for incoming in self.incoming_new_icons.clone().unwrap() {
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
            }
            self.incoming_new_icons = None;
        }
    }

    pub fn set_config(&mut self) {
        // assert!(self.config.len() > 0);
        let mut new = AHashMap::with_capacity(self.config.len());
        for i in self.config.iter() {
            new.insert((i.name.clone(), 1440), i.clone());
        }
        *CONFIG.write() = new;
        self.config = vec![]; // no need to pass in and out after setting
    }
}
