use crate::{
    constants::NUMBER_HEIGHT,
    image_utils::{
        common::{FloatRectangle, IntegerRectangle, Rectangle, get_resizer},
        downscale::{crop_buffer, resize_one_config},
    },
    scanner_state::ScannerState,
};
use ahash::AHashMap;
use fast_image_resize::Resizer;
use hf_core::my_dbg;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub static BASE_ICONS: LazyLock<RwLock<AHashMap<String, OneIconConfig>>> =
    LazyLock::new(|| RwLock::new(AHashMap::new()));

pub static COMPUTED_ICONS: LazyLock<RwLock<AHashMap<(String, u32), OneIconConfig>>> =
    LazyLock::new(|| RwLock::new(AHashMap::new()));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OneIconConfig {
    pub data: Vec<u8>,
    pub name: String,
    pub offset: IntegerRectangle, // naming it like this to distinguish from like actual absolute positions
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncomingNewIcon {
    pub position: FloatRectangle,
    pub name: String,
    pub tag: String,
}

pub fn icon_lookup(
    name: &str,
    resolution: u32,
    resizer: &mut Resizer,
) -> MappedRwLockReadGuard<'static, OneIconConfig> {
    let key = (name.to_string(), resolution);

    let guard = COMPUTED_ICONS.read();
    if guard.contains_key(&key) {
        return RwLockReadGuard::map(guard, |map| &map[&key]);
    }
    drop(guard);

    let base_icon_guard = BASE_ICONS.read();
    // my_dbg!(&resolution);
    let base_icon = base_icon_guard.get(&key.0).expect(&key.0);
    let scale_factor = resolution as f64 / 1440.0;
    let (image, scaled_offset) = resize_one_config(
        if base_icon.tag == "Icon" {
            Some(FloatRectangle {
                top_left: (0.0, NUMBER_HEIGHT * 64.0 / 61.0),
                width: 64.0,
                height: 64.0 - 22.0 * 64.0 / 61.0,
            })
        } else {
            None
        },
        scale_factor
            * if base_icon.tag == "Icon" {
                61.0 / 64.0
            } else {
                1.0
            },
        resizer,
        base_icon,
        base_icon.offset,
    );

    let mut write_guard = COMPUTED_ICONS.write();
    write_guard
        .entry(key.clone())
        .or_insert_with(|| OneIconConfig {
            data: image.into_vec(),
            name: key.0.clone(),
            offset: scaled_offset,
            tag: base_icon.tag.clone(),
        });

    RwLockReadGuard::map(RwLockWriteGuard::downgrade(write_guard), move |map| {
        &map[&key]
    })
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
                        offset: incoming.position.to_rounded(),
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
            new.insert(i.name.clone(), i.clone());
        }
        *BASE_ICONS.write() = new;
        self.config = vec![]; // no need to pass in and out after setting
    }
}
