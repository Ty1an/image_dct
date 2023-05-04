use image::{Rgb, RgbImage};
use rustdct::DctPlanner;


#[derive(Debug, Clone, Copy)]
pub struct YCbCr {
    pub pixel: [u8; 3],
}

impl YCbCr {
    pub fn new(pixel: &Rgb<u8>) -> YCbCr {
        let r = pixel[0] as f64;
        let g = pixel[1] as f64;
        let b = pixel[2] as f64;
    
        let y = (0.299 * r + 0.587 * g + 0.114 * b).round() as u8;
        let cb = (-0.169 * r - 0.331 * g + 0.5 * b + 128.0).round() as u8;
        let cr = (0.5 * r - 0.419 * g - 0.081 * b + 128.0).round() as u8;

        YCbCr { pixel: [y, cb, cr] }
    }


    pub fn get_cb(&self) -> u8 {
        self.pixel[1]
    }

    pub fn get_cr(&self) -> u8 {
        self.pixel[2]
    }

}


pub struct  ImageDct {
    pub image: RgbImage,
    pub grayscale_vec: Vec<f32>,
    pub ycbcr_vec: Vec<YCbCr>,
    pub dct_coefficents : Vec<f32>,
    pub reconstructed_image_vec: Vec<f32>,
    dct_planner: DctPlanner<f32>
}

impl ImageDct {
    pub fn new(image: RgbImage) -> ImageDct {
        
        
        let (width, height) = &image.dimensions();
        let mut grayscale_vec = vec![0.0; (width * height) as usize];
        let mut ycbcr_vec: Vec<YCbCr> = vec![YCbCr { pixel: [0, 0, 0] }; (width * height) as usize];
        // Convert the image to a 2D array of grayscale values
        for (x, y, pixel) in image.enumerate_pixels() {
            let ycbcr = YCbCr::new(&pixel);
            ycbcr_vec[(y * width + x) as usize] = ycbcr;
            grayscale_vec[(y * width + x) as usize] = ycbcr.pixel[0] as f32;
        }

        ImageDct {
            image: image,
            grayscale_vec: grayscale_vec,
            ycbcr_vec: ycbcr_vec,
            dct_coefficents: vec![0.0; (width * height) as usize],
            reconstructed_image_vec: vec![0.0; (width * height) as usize],
            dct_planner: DctPlanner::new()
        }
    }

    pub fn compute_dct(&mut self) {
        let dct = self.dct_planner.plan_dct2(self.grayscale_vec.len());
        self.dct_coefficents = self.grayscale_vec.clone();
        dct.process_dct2(&mut self.dct_coefficents);
        let normalization_factor = (2.0 / self.grayscale_vec.len() as f32).sqrt();
        for i in 0..self.dct_coefficents.len() {
            self.dct_coefficents[i] *= normalization_factor;
        }
    }

    pub fn compute_idct(&mut self) {
        let idct = self.dct_planner.plan_dct3(self.grayscale_vec.len());
        self.reconstructed_image_vec = self.dct_coefficents.clone();
        idct.process_dct3(&mut self.reconstructed_image_vec);
        let normalization_factor = (2.0 / self.grayscale_vec.len() as f32).sqrt();
        for i in 0..self.reconstructed_image_vec.len() {
            self.reconstructed_image_vec[i] *= normalization_factor;
        }
    }

    pub fn reconstructe_image(&mut self) -> RgbImage {
        let mut img_buffer = RgbImage::new(self.width(), self.height());
        for (x, y, pixel) in img_buffer.enumerate_pixels_mut() {
            let index = (y * self.width() + x) as usize;
            let value = self.reconstructed_image_vec[index];
            let r = (value + 1.402 * (self.ycbcr_vec[index].get_cr() as f32 - 128.0)).round() as u8;
            let g = (value - 0.34414 * (self.ycbcr_vec[index].get_cb() as f32 - 128.0) - 0.71414 * (self.ycbcr_vec[index].get_cr() as f32 - 128.0)).round() as u8;
            let b = (value + 1.772 * (self.ycbcr_vec[index].get_cb() as f32 - 128.0)).round() as u8;
            let rgb = Rgb([r, g, b]);
            *pixel = rgb;
        }
        img_buffer
    }


    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height() 
    }
}
