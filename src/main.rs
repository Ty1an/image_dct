use image_dct::image_to_dct::ImageDct;
use std::time::Instant;


fn main() {
    let start = Instant::now();
    let img = image::open("./images/tree-135.webp").unwrap().to_rgb8();

    let mut image_dct = ImageDct::new(img);
    image_dct.compute_dct();
    image_dct.compute_idct();
    let reconstructed_image = image_dct.reconstructe_image();

    image::save_buffer("./images/output.png", &reconstructed_image, image_dct.width() as u32, image_dct.height(), image::ColorType::Rgb8).unwrap();
    let end = Instant::now();
    println!("Finished! -- {:?}", end - start);
}

