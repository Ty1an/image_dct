# Image DCT

Simple Rust implementation for getting DCT coefficients of an image and reconstructing it.

Dependent on [image](https://crates.io/crates/image), [rustdct](https://crates.io/crates/rustdct) crates.

Repository: [Ty1an/image_dct](https://github.com/Ty1an/image_dct)

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
    // Optional: keep only strongest coefficients for lossy reconstruction
    // image_dct.retain_strongest_coefficients(image_dct.dct_coefficients().len() / 8);
    image_dct.compute_idct();

    // Reconstruct it back into an RGB ImageBuffer
    let reconstructed_image = image_dct.reconstruct_image();

    // Save the reconstructed image into a PNG
    image::save_buffer(
        "./output.png",
        &reconstructed_image,
        image_dct.width(),
        image_dct.height(),
        image::ColorType::Rgb8,
    )
    .unwrap();
}
```

## Useful APIs

- `dct_coefficients() -> &[f32]`: inspect frequency-domain data
- `zero_coefficients_below(threshold)`: prune small coefficients
- `retain_strongest_coefficients(keep)`: keep top-K coefficients (simple compression)
- `mse_luma()`: luma reconstruction error after inverse transform
