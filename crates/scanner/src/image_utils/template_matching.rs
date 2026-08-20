use super::common::{config_to_rgba, image_to_rgba};
use crate::{scanner_state::ScaledPosition, setup::OneIconConfig};
use fast_image_resize::images::Image;
use hf_core::my_dbg;
use image::{GrayImage, imageops::grayscale};
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use rustfft::{Fft, FftPlanner, num_complex::Complex32};
use std::sync::Arc;

struct Fft2D {
    rows: usize,
    cols: usize,
    row_fft: Arc<dyn RealToComplex<f32>>,
    row_ifft: Arc<dyn ComplexToReal<f32>>,
    col_fft: Arc<dyn Fft<f32>>,
    col_ifft: Arc<dyn Fft<f32>>,
}

impl Fft2D {
    fn new(rows: usize, cols: usize) -> Self {
        let mut real_planner = RealFftPlanner::<f32>::new();
        let row_fft = real_planner.plan_fft_forward(cols);
        let row_ifft = real_planner.plan_fft_inverse(cols);

        let mut complex_planner = FftPlanner::<f32>::new();
        let col_fft = complex_planner.plan_fft_forward(rows);
        let col_ifft = complex_planner.plan_fft_inverse(rows);

        Self {
            rows,
            cols,
            row_fft,
            row_ifft,
            col_fft,
            col_ifft,
        }
    }

    fn complex_cols(&self) -> usize {
        self.cols / 2 + 1
    }

    fn forward(&self, data: &mut [f32]) -> Vec<Complex32> {
        let cc = self.complex_cols();
        let mut spectrum = vec![Complex32::new(0.0, 0.0); self.rows * cc];
        let mut scratch = self.row_fft.make_scratch_vec();

        for y in 0..self.rows {
            let row = &mut data[y * self.cols..(y + 1) * self.cols];
            let out = &mut spectrum[y * cc..(y + 1) * cc];
            self.row_fft
                .process_with_scratch(row, out, &mut scratch)
                .expect("row fft failed");
        }

        let mut col_buf = vec![Complex32::new(0.0, 0.0); self.rows];
        for x in 0..cc {
            for y in 0..self.rows {
                col_buf[y] = spectrum[y * cc + x];
            }
            self.col_fft.process(&mut col_buf);
            for y in 0..self.rows {
                spectrum[y * cc + x] = col_buf[y];
            }
        }

        spectrum
    }

    fn inverse(&self, spectrum: &mut [Complex32]) -> Vec<f32> {
        let cc = self.complex_cols();

        let mut col_buf = vec![Complex32::new(0.0, 0.0); self.rows];
        for x in 0..cc {
            for y in 0..self.rows {
                col_buf[y] = spectrum[y * cc + x];
            }
            self.col_ifft.process(&mut col_buf);
            for y in 0..self.rows {
                spectrum[y * cc + x] = col_buf[y];
            }
        }

        let nyquist_even = self.cols % 2 == 0;
        for y in 0..self.rows {
            spectrum[y * cc].im = 0.0;
            if nyquist_even {
                spectrum[y * cc + cc - 1].im = 0.0;
            }
        }

        let mut out = vec![0f32; self.rows * self.cols];
        let mut scratch = self.row_ifft.make_scratch_vec();
        for y in 0..self.rows {
            let inp = &mut spectrum[y * cc..(y + 1) * cc];
            let row_out = &mut out[y * self.cols..(y + 1) * self.cols];
            self.row_ifft
                .process_with_scratch(inp, row_out, &mut scratch)
                .expect("row ifft failed");
        }

        let norm = (self.rows * self.cols) as f32;
        for v in out.iter_mut() {
            *v /= norm;
        }

        out
    }
}

fn next_fast_len(n: usize) -> usize {
    let mut candidate = n.max(1);
    loop {
        let mut m = candidate;
        for p in [2usize, 3, 5, 7] {
            while m % p == 0 {
                m /= p;
            }
        }
        if m == 1 {
            return candidate;
        }
        candidate += 1;
    }
}

fn integral_images(pixels: &[f32], width: usize, height: usize) -> (Vec<f64>, Vec<f64>) {
    let w1 = width + 1;
    let h1 = height + 1;
    let mut sum = vec![0f64; w1 * h1];
    let mut sum_sq = vec![0f64; w1 * h1];

    for y in 0..height {
        let mut row_sum = 0f64;
        let mut row_sum_sq = 0f64;
        for x in 0..width {
            let v = pixels[y * width + x] as f64;
            row_sum += v;
            row_sum_sq += v * v;
            let idx = (y + 1) * w1 + (x + 1);
            sum[idx] = sum[idx - w1] + row_sum;
            sum_sq[idx] = sum_sq[idx - w1] + row_sum_sq;
        }
    }

    (sum, sum_sq)
}

fn box_sum(table: &[f64], w1: usize, x0: usize, y0: usize, x1: usize, y1: usize) -> f64 {
    table[y1 * w1 + x1] - table[y0 * w1 + x1] - table[y1 * w1 + x0] + table[y0 * w1 + x0]
}

fn ncc_score(
    correlation: &[f32],
    cols: usize,
    sum_table: &[f64],
    sum_sq_table: &[f64],
    w1: usize,
    t_n: f64,
    t_ss: f64,
    tw: usize,
    th: usize,
    x: usize,
    y: usize,
) -> f64 {
    let numerator = correlation[y * cols + x] as f64;
    let s1 = box_sum(sum_table, w1, x, y, x + tw, y + th);
    let s2 = box_sum(sum_sq_table, w1, x, y, x + tw, y + th);
    let var_f_sum = (s2 - s1 * s1 / t_n).max(0.0);
    let denom = (var_f_sum * t_ss).sqrt();
    if denom > 1e-6 { numerator / denom } else { 0.0 }
}

fn parabolic_offset(left: f64, center: f64, right: f64) -> f64 {
    let denom = left - 2.0 * center + right;
    if denom.abs() < 1e-9 {
        0.0
    } else {
        (0.5 * (left - right) / denom).clamp(-0.5, 0.5)
    }
}

fn parabolic_peak_value(left: f64, center: f64, right: f64) -> f64 {
    let denom = left - 2.0 * center + right;
    if denom.abs() < 1e-9 {
        center
    } else {
        center - (right - left).powi(2) / (8.0 * denom)
    }
}

pub fn template_match(
    template: &OneIconConfig,
    observed: Image,
) -> Option<(ScaledPosition, f64, f64)> {
    let template_img = config_to_rgba(template);
    let observed_img = image_to_rgba(observed);

    let template_gray: GrayImage = grayscale(&template_img);
    let observed_gray: GrayImage = grayscale(&observed_img);

    let tw = template_gray.width() as usize;
    let th = template_gray.height() as usize;
    let ow = observed_gray.width() as usize;
    let oh = observed_gray.height() as usize;

    if tw == 0 || th == 0 || ow < tw || oh < th {
        return None;
    }

    let t_pixels: Vec<f32> = template_gray.as_raw().iter().map(|&p| p as f32).collect();
    let f_pixels: Vec<f32> = observed_gray.as_raw().iter().map(|&p| p as f32).collect();

    let t_n = (tw * th) as f64;
    let t_mean = t_pixels.iter().map(|&v| v as f64).sum::<f64>() / t_n;
    let t_ss: f64 = t_pixels
        .iter()
        .map(|&v| {
            let d = v as f64 - t_mean;
            d * d
        })
        .sum();

    if t_ss <= 1e-6 {
        return None;
    }

    let rows = next_fast_len(oh + th - 1);
    let cols = next_fast_len(ow + tw - 1);
    let fft2d = Fft2D::new(rows, cols);

    let mut f_padded = vec![0f32; rows * cols];
    for y in 0..oh {
        let src = &f_pixels[y * ow..(y + 1) * ow];
        let dst = y * cols;
        f_padded[dst..dst + ow].copy_from_slice(src);
    }

    let mut g_padded = vec![0f32; rows * cols];
    for y in 0..th {
        for x in 0..tw {
            g_padded[y * cols + x] = (t_pixels[y * tw + x] as f64 - t_mean) as f32;
        }
    }

    let f_spectrum = fft2d.forward(&mut f_padded);
    let mut g_spectrum = fft2d.forward(&mut g_padded);

    for (f_val, g_val) in f_spectrum.iter().zip(g_spectrum.iter_mut()) {
        *g_val = f_val * g_val.conj();
    }

    let correlation = fft2d.inverse(&mut g_spectrum);

    let (sum_table, sum_sq_table) = integral_images(&f_pixels, ow, oh);
    let w1 = ow + 1;

    let valid_h = oh - th + 1;
    let valid_w = ow - tw + 1;

    let mut best_score = f64::MIN;
    let mut best_x = 0usize;
    let mut best_y = 0usize;

    for y in 0..valid_h {
        for x in 0..valid_w {
            let score = ncc_score(
                &correlation,
                cols,
                &sum_table,
                &sum_sq_table,
                w1,
                t_n,
                t_ss,
                tw,
                th,
                x,
                y,
            );
            if score > best_score {
                best_score = score;
                best_x = x;
                best_y = y;
            }
        }
    }

    let s1 = box_sum(&sum_table, w1, best_x, best_y, best_x + tw, best_y + th);
    let best_mean_f = s1 / t_n;

    let mut dx = 0.0;
    let mut x_peak_value = best_score;
    if best_x > 0 && best_x + 1 < valid_w {
        let left = ncc_score(
            &correlation,
            cols,
            &sum_table,
            &sum_sq_table,
            w1,
            t_n,
            t_ss,
            tw,
            th,
            best_x - 1,
            best_y,
        );
        let right = ncc_score(
            &correlation,
            cols,
            &sum_table,
            &sum_sq_table,
            w1,
            t_n,
            t_ss,
            tw,
            th,
            best_x + 1,
            best_y,
        );
        dx = parabolic_offset(left, best_score, right);
        x_peak_value = parabolic_peak_value(left, best_score, right);
    }

    let mut dy = 0.0;
    let mut y_peak_value = best_score;
    if best_y > 0 && best_y + 1 < valid_h {
        let up = ncc_score(
            &correlation,
            cols,
            &sum_table,
            &sum_sq_table,
            w1,
            t_n,
            t_ss,
            tw,
            th,
            best_x,
            best_y - 1,
        );
        let down = ncc_score(
            &correlation,
            cols,
            &sum_table,
            &sum_sq_table,
            w1,
            t_n,
            t_ss,
            tw,
            th,
            best_x,
            best_y + 1,
        );
        dy = parabolic_offset(up, best_score, down);
        y_peak_value = parabolic_peak_value(up, best_score, down);
    }
    let corrected_score = x_peak_value + y_peak_value - best_score;

    Some((
        ScaledPosition {
            top_left: (best_x as f64 + dx, best_y as f64 + dy),
            width: template.offset.width,
            height: template.offset.height,
        },
        corrected_score.clamp(-1.0, 1.0),
        best_mean_f,
    ))
}
