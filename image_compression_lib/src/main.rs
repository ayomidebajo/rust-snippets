use image::{
    imageops::colorops::{grayscale, invert},
    GenericImageView, ImageBuffer, Pixel, Rgb, RgbImage,
};
use ndarray;

pub fn preprocess_image(image: impl GenericImageView) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    //  Create a square matrix with side length that's a power of 2.

    let image = grayscale(&image);

    let (width, height) = image.dimensions();

    // Find the largest dimensions of the image.
    let max_dimension = width.max(height);

    // Find the smallest power of 2 that's larger than the largest dimension.
    let dimension = max_dimension.next_power_of_two();

    // let new_image = ImageBuffer::new(width, height);

    let rgb_new = Rgb([0, 0, 0]);

    // Pad the image with white pixels so that it's a square with side length `dimension`.
    let padded_image =
        RgbImage::from_pixel(dimension, dimension, rgb_new);

    padded_image

    // for x in 0..width {
    //     for y in 0..height {
    //         padded_image.put_pixel(x, y, image.get_pixel(x, y).to_rgb());
    //     }

    // }
}

fn main() {
    println!("Hello, world!");

    let rgb_image = RgbImage::new(100, 100);

    let new_image = preprocess_image(rgb_image);

    println!("just checking {:?}", new_image);

}
