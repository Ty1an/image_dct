use image::{Rgb, RgbImage};
use rustdct::DctPlanner;

#[derive(Clone, Copy, Debug, Default)]
pub struct YCbCr {
    pub pixel: [u8; 3],
}

impl YCbCr {
    // Convert the RGB pixel to YCbCr
    pub fn new(pixel: &Rgb<u8>) -> YCbCr {
        let r = f32::from(pixel[0]);
        let g = f32::from(pixel[1]);
        let b = f32::from(pixel[2]);

        let y = clamp_u8(0.299 * r + 0.587 * g + 0.114 * b);
        let cb = clamp_u8(-0.169 * r - 0.331 * g + 0.5 * b + 128.0);
        let cr = clamp_u8(0.5 * r - 0.419 * g - 0.081 * b + 128.0);

        YCbCr { pixel: [y, cb, cr] }
    }

    pub fn y(&self) -> u8 {
        self.pixel[0]
    }

    pub fn cb(&self) -> u8 {
        self.pixel[1]
    }

    pub fn cr(&self) -> u8 {
        self.pixel[2]
    }

    pub fn get_cb(&self) -> u8 {
        self.cb()
    }

    pub fn get_cr(&self) -> u8 {
        self.cr()
    }
}

#[inline]
fn clamp_u8(value: f32) -> u8 {
    value.clamp(0.0, 255.0).round() as u8
}

pub struct ImageDct {
    pub image: RgbImage,
    pub grayscale_vec: Vec<f32>,
    pub ycbcr_vec: Vec<YCbCr>,
    pub dct_coefficents: Vec<f32>,
    pub reconstructed_image_vec: Vec<f32>,
    dct_planner: DctPlanner<f32>,
}

impl ImageDct {
    pub fn new(image: RgbImage) -> ImageDct {
        let (width, height) = image.dimensions();
        let len = (width * height) as usize;
        let mut grayscale_vec = Vec::with_capacity(len);
        let mut ycbcr_vec = Vec::with_capacity(len);

        // Convert once into luma for DCT and store chroma data for reconstruction.
        for pixel in image.pixels() {
            let ycbcr = YCbCr::new(pixel);
            ycbcr_vec.push(ycbcr);
            grayscale_vec.push(f32::from(ycbcr.y()));
        }

        ImageDct {
            image,
            grayscale_vec,
            ycbcr_vec,
            dct_coefficents: vec![0.0; len],
            reconstructed_image_vec: vec![0.0; len],
            dct_planner: DctPlanner::new(),
        }
    }

    pub fn compute_dct(&mut self) {
        self.prepare_working_buffers();
        if self.grayscale_vec.is_empty() {
            return;
        }

        let dct = self.dct_planner.plan_dct2(self.grayscale_vec.len());
        self.dct_coefficents.copy_from_slice(&self.grayscale_vec);
        dct.process_dct2(&mut self.dct_coefficents);
        let normalization_factor = (2.0 / self.grayscale_vec.len() as f32).sqrt();
        for coeff in &mut self.dct_coefficents {
            *coeff *= normalization_factor;
        }
    }

    pub fn compute_idct(&mut self) {
        self.prepare_working_buffers();
        if self.dct_coefficents.is_empty() {
            return;
        }

        let idct = self.dct_planner.plan_dct3(self.grayscale_vec.len());
        self.reconstructed_image_vec
            .copy_from_slice(&self.dct_coefficents);
        idct.process_dct3(&mut self.reconstructed_image_vec);
        let normalization_factor = (2.0 / self.grayscale_vec.len() as f32).sqrt();
        for coeff in &mut self.reconstructed_image_vec {
            *coeff *= normalization_factor;
        }
    }

    pub fn reconstruct_image(&self) -> RgbImage {
        let mut img_buffer = RgbImage::new(self.width(), self.height());
        for (index, pixel) in img_buffer.pixels_mut().enumerate() {
            let value = self
                .reconstructed_image_vec
                .get(index)
                .copied()
                .unwrap_or(0.0);
            let chroma = self.ycbcr_vec.get(index).copied().unwrap_or_default();

            let cr = f32::from(chroma.cr()) - 128.0;
            let cb = f32::from(chroma.cb()) - 128.0;

            let r = clamp_u8(value + 1.402 * cr);
            let g = clamp_u8(value - 0.34414 * cb - 0.71414 * cr);
            let b = clamp_u8(value + 1.772 * cb);

            *pixel = Rgb([r, g, b]);
        }
        img_buffer
    }

    #[deprecated(note = "Use reconstruct_image instead")]
    pub fn reconstructe_image(&self) -> RgbImage {
        self.reconstruct_image()
    }

    pub fn dct_coefficients(&self) -> &[f32] {
        &self.dct_coefficents
    }

    pub fn nonzero_coefficient_count(&self) -> usize {
        self.dct_coefficents
            .iter()
            .filter(|coeff| **coeff != 0.0)
            .count()
    }

    pub fn zero_coefficients_below(&mut self, threshold: f32) -> usize {
        let threshold = threshold.abs();
        let mut zeroed = 0;
        for coeff in &mut self.dct_coefficents {
            if coeff.abs() < threshold {
                if *coeff != 0.0 {
                    zeroed += 1;
                }
                *coeff = 0.0;
            }
        }
        zeroed
    }

    pub fn retain_strongest_coefficients(&mut self, keep: usize) -> usize {
        let len = self.dct_coefficents.len();
        if keep >= len {
            return self.nonzero_coefficient_count();
        }
        if keep == 0 {
            self.dct_coefficents.fill(0.0);
            return 0;
        }

        let mut indices: Vec<usize> = (0..len).collect();
        indices.sort_unstable_by(|lhs, rhs| {
            self.dct_coefficents[*rhs]
                .abs()
                .total_cmp(&self.dct_coefficents[*lhs].abs())
        });

        let mut keep_mask = vec![false; len];
        for index in indices.into_iter().take(keep) {
            keep_mask[index] = true;
        }

        for (index, coeff) in self.dct_coefficents.iter_mut().enumerate() {
            if !keep_mask[index] {
                *coeff = 0.0;
            }
        }

        self.nonzero_coefficient_count()
    }

    pub fn mse_luma(&self) -> Option<f32> {
        if self.grayscale_vec.len() != self.reconstructed_image_vec.len()
            || self.grayscale_vec.is_empty()
        {
            return None;
        }

        let mut sum = 0.0;
        for (original, reconstructed) in self
            .grayscale_vec
            .iter()
            .zip(self.reconstructed_image_vec.iter())
        {
            let delta = original - reconstructed;
            sum += delta * delta;
        }

        Some(sum / self.grayscale_vec.len() as f32)
    }

    fn prepare_working_buffers(&mut self) {
        let len = self.grayscale_vec.len();
        if self.dct_coefficents.len() != len {
            self.dct_coefficents.resize(len, 0.0);
        }
        if self.reconstructed_image_vec.len() != len {
            self.reconstructed_image_vec.resize(len, 0.0);
        }
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }
}

#[cfg(test)]
mod tests {
    use super::ImageDct;
    use image::{Rgb, RgbImage};

    fn build_test_image(width: u32, height: u32) -> RgbImage {
        let mut img = RgbImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let r = ((x * 17 + y * 3) % 256) as u8;
                let g = ((x * 11 + y * 7) % 256) as u8;
                let b = ((x * 5 + y * 13) % 256) as u8;
                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        img
    }

    fn rgb_mse(lhs: &RgbImage, rhs: &RgbImage) -> f32 {
        assert_eq!(lhs.dimensions(), rhs.dimensions());
        let mut error = 0.0f32;
        for (left, right) in lhs.pixels().zip(rhs.pixels()) {
            for channel in 0..3 {
                let delta = f32::from(left[channel]) - f32::from(right[channel]);
                error += delta * delta;
            }
        }
        error / (lhs.width() * lhs.height() * 3) as f32
    }

    #[test]
    fn roundtrip_reconstruction_stays_close() {
        let original = build_test_image(32, 32);
        let mut image_dct = ImageDct::new(original.clone());
        image_dct.compute_dct();
        image_dct.compute_idct();
        let reconstructed = image_dct.reconstruct_image();

        let mse = rgb_mse(&original, &reconstructed);
        assert!(mse < 6.0, "MSE too high: {mse}");
    }

    #[test]
    fn strongest_coefficient_filter_reduces_signal() {
        let original = build_test_image(16, 16);
        let mut image_dct = ImageDct::new(original);
        image_dct.compute_dct();

        let before = image_dct.nonzero_coefficient_count();
        let after = image_dct.retain_strongest_coefficients(32);

        assert!(before >= after);
        assert!(after <= 32);
    }
}
