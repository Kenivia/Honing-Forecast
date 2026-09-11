use crate::{
    image_utils::resize::resize_one_config,
    setup::{OCR_ENGINE, OneIconConfig},
};
use fast_image_resize::{Resizer, images::Image};
use image::{GrayImage, Luma, RgbaImage, imageops::grayscale};
use imageproc::region_labelling::{Connectivity, connected_components};
use ocrs::ImageSource;
use rten_imageproc::{PointF, RotatedRect, Vec2};
use std::collections::{HashMap, HashSet};

fn colorfulness_mask(icon: &mut OneIconConfig, threshold: u8) {
    let mut img = RgbaImage::from_raw(
        icon.offset.width as u32,
        icon.offset.height as u32,
        std::mem::take(&mut icon.data),
    )
    .expect("data does not match declared dimensions");

    for px in img.pixels_mut() {
        let max = px.0[0].max(px.0[1]).max(px.0[2]);
        let min = px.0[0].min(px.0[1]).min(px.0[2]);
        let saturation = if max == 0 {
            0.0
        } else {
            (max - min) as f32 / max as f32
        };
        if saturation * 255.0 > threshold as f32 {
            px.0[0] = 0;
            px.0[1] = 0;
            px.0[2] = 0;
        }
    }

    icon.data = img.into_raw();
}

fn luminosity_threshold(icon: &mut OneIconConfig, threshold: u8) {
    let mut img = RgbaImage::from_raw(
        icon.offset.width as u32,
        icon.offset.height as u32,
        std::mem::take(&mut icon.data),
    )
    .expect("data does not match declared dimensions");
    let gray = grayscale(&img);

    for (px, &Luma([luma])) in img.pixels_mut().zip(gray.pixels()) {
        let value = if luma >= threshold { luma } else { 0 };
        px.0[0] = value;
        px.0[1] = value;
        px.0[2] = value;
    }

    icon.data = img.into_raw();
}

/// Zeroes out the white region reachable from the image border (4-connectivity),
/// leaving enclosed white holes untouched.
fn background_flood_fill(icon: &mut OneIconConfig, tolerance: u8) {
    let (w, h) = (icon.offset.width as u32, icon.offset.height as u32);
    let mut img = RgbaImage::from_raw(w, h, std::mem::take(&mut icon.data))
        .expect("data does not match declared dimensions");

    let mask: GrayImage = GrayImage::from_fn(w, h, |x, y| {
        let p = img.get_pixel(x, y);
        let is_white =
            255 - p[0] <= tolerance && 255 - p[1] <= tolerance && 255 - p[2] <= tolerance;
        Luma([if is_white { 255 } else { 0 }])
    });
    let labels = connected_components(&mask, Connectivity::Four, Luma([0u8]));

    let mut border_labels: HashSet<u32> = HashSet::new();
    for x in 0..w {
        border_labels.insert(labels.get_pixel(x, 0)[0]);
        border_labels.insert(labels.get_pixel(x, h - 1)[0]);
    }
    for y in 0..h {
        border_labels.insert(labels.get_pixel(0, y)[0]);
        border_labels.insert(labels.get_pixel(w - 1, y)[0]);
    }
    border_labels.remove(&0);

    for (x, y, px) in img.enumerate_pixels_mut() {
        if border_labels.contains(&labels.get_pixel(x, y)[0]) {
            px.0[0] = 0;
            px.0[1] = 0;
            px.0[2] = 0;
        }
    }

    icon.data = img.into_raw();
}

/// Blackens connected clusters of pixels above `brightness_threshold` luma
/// whose size is at or below `size_threshold`.
fn speck_removal(icon: &mut OneIconConfig, brightness_threshold: u8, size_threshold: usize) {
    let (w, h) = (icon.offset.width as u32, icon.offset.height as u32);
    let mut img = RgbaImage::from_raw(w, h, std::mem::take(&mut icon.data))
        .expect("data does not match declared dimensions");
    let gray = grayscale(&img);

    let mask: GrayImage = GrayImage::from_fn(w, h, |x, y| {
        Luma([if gray.get_pixel(x, y)[0] > brightness_threshold {
            255
        } else {
            0
        }])
    });
    let labels = connected_components(&mask, Connectivity::Four, Luma([0u8]));

    let mut sizes: HashMap<u32, usize> = HashMap::new();
    for &Luma([label]) in labels.pixels() {
        if label != 0 {
            *sizes.entry(label).or_insert(0) += 1;
        }
    }

    for (x, y, px) in img.enumerate_pixels_mut() {
        let label = labels.get_pixel(x, y)[0];
        if label != 0 && sizes[&label] <= size_threshold {
            px.0[0] = 0;
            px.0[1] = 0;
            px.0[2] = 0;
        }
    }

    icon.data = img.into_raw();
}

pub fn pre_process(mut icon: OneIconConfig, resizer: &mut Resizer) -> OneIconConfig {
    colorfulness_mask(&mut icon, 40);
    luminosity_threshold(&mut icon, 100);
    background_flood_fill(&mut icon, 100);
    speck_removal(&mut icon, 75, 2);
    let (scaled_image, scaled_offset) =
        resize_one_config(None, 64.0 / icon.offset.height as f64, resizer, &icon);
    OneIconConfig {
        data: scaled_image.into_vec(),
        name: icon.name,
        offset: scaled_offset,
        tag: icon.tag,
    }
}

pub fn get_number(scaled_image: OneIconConfig) -> String {
    let read = OCR_ENGINE.read(); // need to have this line for some reason
    let engine = read.as_ref().unwrap();
    let width = scaled_image.offset.width as u32;
    let height = scaled_image.offset.height as u32;
    return engine
        .recognize_text(
            &engine
                .prepare_input(
                    ImageSource::from_bytes(&scaled_image.data, (width, height)).unwrap(),
                )
                .unwrap(),
            &vec![vec![RotatedRect::new(
                PointF::from_yx(height as f32 / 2.0, width as f32 / 2.0),
                Vec2::from_yx(1., 0.),
                width as f32,
                height as f32,
            )]],
        )
        .iter()
        .flatten()
        .map(|x| {
            let y = x.as_ref();
            if y.is_some() {
                return y.unwrap().to_string();
            } else {
                return "".to_string();
            }
        })
        .reduce(|prev, new| prev + &new)
        .unwrap_or("".to_string());
}
