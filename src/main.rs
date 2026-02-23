use image_dct::image_to_dct::ImageDct;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    // load image as a RGB ImageBuffer
    let img = image::open("./images/tree-135.webp")
        .expect("failed to open input image")
        .to_rgb8();

    // Create the ImageDct object from ImageBuffer
    let mut image_dct = ImageDct::new(img);

    // Compute the DCT of the image then compute the inverse DCT on the coefficients
    image_dct.compute_dct();
    // Example compression knob: keep only the strongest coefficients.
    // image_dct.retain_strongest_coefficients(image_dct.dct_coefficients().len() / 8);
    image_dct.compute_idct();

    // Reconstruct it back into an RGB ImageBuffer
    let reconstructed_image = image_dct.reconstruct_image();

    // Save the reconstructed image into a PNG
    image::save_buffer(
        "./images/output.png",
        &reconstructed_image,
        image_dct.width(),
        image_dct.height(),
        image::ColorType::Rgb8,
    )
    .expect("failed to save output image");
    let end = Instant::now();
    println!("Finished! -- {:?}", end - start);
}
