use crate::{
    constants::COMBINED_NUMBER_HEIGHT,
    image_utils::{common::Rect, resize::resize_one_config},
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

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "OneIconConfigJs")]
pub struct OneIconConfig {
    pub data: RgbaImage,
    pub name: String,
    // a base template's is in UI units, a scaled template's and a crop's in screen pixels
    pub offset: Rect,
    pub tag: String,
    // overrides the close_enough pass limit, only set for page tabs
    pub required_confidence: Option<f64>,
}

// a rectangle as the config file and the page have it
#[derive(Serialize, Deserialize)]
pub struct WireRect {
    top_left: (f64, f64),
    width: usize,
    height: usize,
}

impl From<&Rect> for WireRect {
    fn from(rect: &Rect) -> Self {
        let (width, height) = rect.pixel_size();
        Self { top_left: rect.top_left, width: width as usize, height: height as usize }
    }
}

// the file's `normalized` is not read: every stored template is
#[derive(Deserialize)]
struct OneIconConfigJs {
    data: Vec<u8>,
    name: String,
    offset: WireRect,
    tag: String,
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
            offset: Rect::ui(w.offset.top_left, width as f64, height as f64),
            tag: w.tag,
            required_confidence: w.required_confidence,
        })
    }
}

// The largest centred part of a template whose scaled size is a whole number of pixels. A template
// scaled to a rounded size is stretched by up to half a pixel against what it has to match, which
// costs match score and moves the position it reports; cut this way it is pixel for pixel.
fn whole_pixel_crop(offset: &Rect, scale: f64) -> Rect {
    let fit = |side: f64| (side * scale).floor() / scale;
    let (width, height) = (fit(offset.width), fit(offset.height));
    Rect::ui(
        ((offset.width - width) / 2.0, (offset.height - height) / 2.0),
        width,
        height,
    )
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
    let base_at = base_icon.offset.top_left;
    // the part of the base template that is used, in its own pixels, and where that sits on screen
    let (crop_top_left, crop_size, scaled_offset) = if is_icon {
        // 64 px art that a slot draws 61 wide, without the rows the number is on
        let art_scale = scale_factor * (61.0 / 64.0);
        let top_left = (0.0, COMBINED_NUMBER_HEIGHT * 64.0 / 61.0);
        let size = (64.0, 64.0 - 22.0 * 64.0 / 61.0);
        let on_screen = Rect::screen(
            (base_at.0 * art_scale, base_at.1 * art_scale),
            size.0 * art_scale,
            size.1 * art_scale,
            resolution,
        );
        (top_left, size, on_screen)
    } else {
        let crop = whole_pixel_crop(&base_icon.offset, scale_factor);
        // the offset says where the template sits, so it moves with the cut
        let on_screen = Rect::ui(base_at, crop.width, crop.height)
            .scaled(resolution)
            .shifted((crop.top_left.0 * scale_factor, crop.top_left.1 * scale_factor));
        (crop.top_left, (crop.width, crop.height), on_screen)
    };
    // a template is whole pixels, and so is anything placed by its offset
    let scaled_offset = scaled_offset.whole_pixels();
    let image = resize_one_config(
        crop_top_left,
        crop_size,
        scaled_offset.pixel_size(),
        resizer,
        base_icon,
    );

    let mut write_guard = COMPUTED_ICONS.write();
    write_guard
        .entry(key.clone())
        .or_insert_with(|| OneIconConfig {
            data: image,
            name: key.0.clone(),
            offset: scaled_offset,
            tag: base_icon.tag.clone(),
            required_confidence: base_icon.required_confidence,
        });

    RwLockReadGuard::map(RwLockWriteGuard::downgrade(write_guard), move |map| {
        &map[&key]
    })
}

// The base templates everything is matched against. The file is written by
// templates/make_msg_pack.py (icons) and scripts/anchor setup/anchors.py (anchors).
pub fn set_config(config: Vec<OneIconConfig>) {
    *BASE_ICONS.write() = config.into_iter().map(|icon| (icon.name.clone(), icon)).collect();
    crate::cropper::slots::FROM_AFAR.write().clear();
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
