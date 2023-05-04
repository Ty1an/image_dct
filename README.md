# Image DCT

Simple Rust implementation for getting the DCT coefficients of an image.

Dependent on [image](https://crates.io/crates/image), [rustdct](https://crates.io/crates/rustdct) crates.

# Install
```bash
cargo add image_dct
```

## Usage
```rust
use image_dct::image_to_dct::ImageDct;

fn main() {
    // load image as a RGB ImageBuffer
    let img = image::open("image.png").unwrap().to_rgb8();

    // Create the ImageDct object from ImageBuffer
    let mut image_dct = ImageDct::new(img);

    // Compute the DCT of the image then compute the inverse DCT on the coefficients
    image_dct.compute_dct();
    image_dct.compute_idct();

    // Reconstruct it back into an RGB ImageBuffer
    let reconstructed_image = image_dct.reconstructe_image();

    // Save the reconstructed image into a PNG
    image::save_buffer(
        "./output.png",
        &reconstructed_image,
        image_dct.width() as u32,
        image_dct.height(),
        image::ColorType::Rgb8,
    )
    .unwrap();
}
```
