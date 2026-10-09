use crate::{
    constants::COMBINED_NUMBER_HEIGHT,
    image_utils::{
        brightness::normalize_brightness,
        common::{FloatRectangle, IntegerRectangle, Rectangle, get_resizer},
        resize::{crop_buffer, resize_one_config},
    },
    scanner_state::ScannerState,
};
use ahash::AHashMap;
use fast_image_resize::Resizer;
use ocrs::{OcrEngine, OcrEngineParams};
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use rten::Model;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub static OCR_ENGINE: LazyLock<RwLock<Option<OcrEngine>>> = LazyLock::new(|| RwLock::new(None));
// the same model, kept to what a slot's count can be
pub static OCR_NUMBER_ENGINE: LazyLock<RwLock<Option<OcrEngine>>> =
    LazyLock::new(|| RwLock::new(None));
const NUMBER_CHARS: &str = "0123456789.+-";

pub static BASE_ICONS: LazyLock<RwLock<AHashMap<String, OneIconConfig>>> =
    LazyLock::new(|| RwLock::new(AHashMap::new()));

pub static COMPUTED_ICONS: LazyLock<RwLock<AHashMap<(String, u32), OneIconConfig>>> =
    LazyLock::new(|| RwLock::new(AHashMap::new()));

use image::RgbaImage;
use serde::Serializer;

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "OneIconConfigJs")]
pub struct OneIconConfig {
    pub data: RgbaImage,
    pub name: String,
    pub offset: IntegerRectangle,
    pub tag: String,
    pub normalized: bool,
    // overrides the close_enough pass limit, only set for page tabs
    pub required_confidence: Option<f64>,
}

#[derive(Deserialize)]
struct OneIconConfigJs {
    data: Vec<u8>,
    name: String,
    offset: IntegerRectangle,
    tag: String,
    normalized: bool,
    #[serde(default)]
    required_confidence: Option<f64>,
}

impl TryFrom<OneIconConfigJs> for OneIconConfig {
    type Error = String;

    fn try_from(w: OneIconConfigJs) -> Result<Self, Self::Error> {
        let (width, height) = (w.offset.width as u32, w.offset.height as u32);
        let data = RgbaImage::from_raw(width, height, w.data)
            .ok_or_else(|| format!("'{}': data len doesn't match {width}x{height}", w.name))?;
        Ok(Self {
            data,
            name: w.name,
            offset: w.offset,
            tag: w.tag,
            normalized: w.normalized,
            required_confidence: w.required_confidence,
        })
    }
}

impl Serialize for OneIconConfig {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("OneIconConfig", 6)?;
        s.serialize_field("data", self.data.as_raw())?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("offset", &self.offset)?;
        s.serialize_field("tag", &self.tag)?;
        s.serialize_field("normalized", &self.normalized)?;
        s.serialize_field("required_confidence", &self.required_confidence)?;
        s.end()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncomingNewIcon {
    pub position: FloatRectangle,
    pub name: String,
    pub tag: String,
    pub brightness: f64,
}

// The largest centred part of a template whose scaled size is a whole number of pixels. A template
// scaled to a rounded size is stretched by up to half a pixel against what it has to match, which
// costs match score and moves the position it reports; cut this way it is pixel for pixel.
fn whole_pixel_crop(offset: &IntegerRectangle, scale: f64) -> FloatRectangle {
    let fit = |side: usize| (side as f64 * scale).floor() / scale;
    let (width, height) = (fit(offset.width), fit(offset.height));
    FloatRectangle {
        top_left: (
            (offset.width as f64 - width) / 2.0,
            (offset.height as f64 - height) / 2.0,
        ),
        width,
        height,
    }
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
    let base_icon = base_icon_guard.get(&key.0).expect(&key.0);
    let scale_factor = resolution as f64 / 1440.0;
    let is_icon = base_icon.tag == "Icon";
    let crop = if is_icon {
        FloatRectangle {
            top_left: (0.0, COMBINED_NUMBER_HEIGHT * 64.0 / 61.0),
            width: 64.0,
            height: 64.0 - 22.0 * 64.0 / 61.0,
        }
    } else {
        whole_pixel_crop(&base_icon.offset, scale_factor)
    };
    let (image, mut scaled_offset) = resize_one_config(
        Some(crop),
        scale_factor * if is_icon { 61.0 / 64.0 } else { 1.0 },
        resizer,
        base_icon,
    );
    // the offset says where the template sits, so it moves with the cut
    if !is_icon {
        scaled_offset.top_left = (
            scaled_offset.top_left.0 + crop.top_left.0 * scale_factor,
            scaled_offset.top_left.1 + crop.top_left.1 * scale_factor,
        );
    }

    let mut write_guard = COMPUTED_ICONS.write();
    write_guard
        .entry(key.clone())
        .or_insert_with(|| OneIconConfig {
            data: RgbaImage::from_raw(
                scaled_offset.width as u32,
                scaled_offset.height as u32,
                image.into_vec(),
            )
            .unwrap(),
            name: key.0.clone(),
            offset: scaled_offset,
            tag: base_icon.tag.clone(),
            normalized: true,
            required_confidence: base_icon.required_confidence,
        });

    RwLockReadGuard::map(RwLockWriteGuard::downgrade(write_guard), move |map| {
        &map[&key]
    })
}

impl ScannerState {
    pub fn setup(&mut self) {
        if self.incoming_new_icons.is_some() {
            for incoming in self.incoming_new_icons.clone().unwrap() {
                let mut observed = crop_buffer(
                    incoming.position,
                    get_resizer(&mut self.resizer),
                    self.buffer,
                    None,
                );
                normalize_brightness(&mut observed, 70.0);
                self.config.insert(
                    0,
                    OneIconConfig {
                        data: observed.data,
                        name: incoming.name,
                        offset: incoming.position.to_rounded(),
                        tag: incoming.tag,
                        normalized: true,
                        required_confidence: None,
                    },
                );
            }
            self.incoming_new_icons = None;
        }
    }

    pub fn set_config(&mut self) {
        let mut new = AHashMap::with_capacity(self.config.len());
        for i in self.config.iter() {
            new.insert(i.name.clone(), i.clone());
        }
        *BASE_ICONS.write() = new;
        crate::cropper::slots::FROM_AFAR.write().clear();
        self.config = vec![]; // no need to pass in and out after setting
    }

    pub fn set_ocr_engine(&mut self) {
        load_ocr_engine(self.model.take().unwrap());
    }
}

pub fn load_ocr_engine(model: Vec<u8>) {
    // the alphabet is fixed per engine, so the model is loaded once for each
    let engine = |allowed_chars: Option<String>| {
        OcrEngine::new(OcrEngineParams {
            recognition_model: Some(Model::load(model.clone()).expect("model load failed")),
            allowed_chars,
            ..Default::default()
        })
        .expect("model load failed")
    };
    *OCR_ENGINE.write() = Some(engine(None));
    *OCR_NUMBER_ENGINE.write() = Some(engine(Some(NUMBER_CHARS.to_string())));
}
