use crate::{
    buffer::Buffer,
    constants::{ICON_HEIGHT, ICON_MARGIN, ICON_WIDTH},
    image_utils::{
        common::{FloatRectangle, IntegerRectangle, Rectangle, get_resizer},
        downscale::crop_buffer,
    },
    scanner_state::ScannerState,
};
use ahash::AHashMap;
use fast_image_resize::Resizer;
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

    let base_icon_guard = BASE_ICONS.read();
    let base_icon = base_icon_guard.get(&key.0).unwrap();
    let position = FloatRectangle {
        top_left: (ICON_MARGIN.left, ICON_MARGIN.top),
        width: ICON_WIDTH,
        height: ICON_HEIGHT,
    }
    .scaled(resolution as f64 / 1440.0);
    let mut write_guard = COMPUTED_ICONS.write();
    write_guard
        .entry(key.clone())
        .or_insert_with(|| OneIconConfig {
            data: crop_buffer(
                position,
                resizer,
                Buffer {
                    pointer: Some(base_icon.data.as_ptr() as usize),
                    width: base_icon.offset.width,
                    height: base_icon.offset.height,
                    size: base_icon.data.len(),
                },
            )
            .into_vec(),
            name: key.0.clone(),
            offset: position.to_rounded(),
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
