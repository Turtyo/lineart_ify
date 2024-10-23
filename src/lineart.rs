use anyhow::{Context, Result};
use photon_rs::{
    channels::invert,
    conv::{gaussian_blur, sobel_horizontal, sobel_vertical},
    monochrome::desaturate,
    multiple::blend,
    PhotonImage,
};

pub(crate) fn gaussian_blend_dodge(mut image: PhotonImage) -> PhotonImage {
    desaturate(&mut image);
    let mut blend_layer = image.clone();
    invert(&mut blend_layer);
    gaussian_blur(&mut blend_layer, 3);
    blend(&mut image, &blend_layer, "dodge");
    image
}

fn calculate_global_sobel(image: PhotonImage) -> Result<PhotonImage> {
    let mut sobel_y = image.clone();
    let mut sobel_x = image;
    sobel_horizontal(&mut sobel_x);
    sobel_vertical(&mut sobel_y);

    let sob_x_values = sobel_x.get_raw_pixels();
    let sob_y_values = sobel_y.get_raw_pixels();

    let width = sobel_x.get_width();
    let height = sobel_x.get_height();

    let mut sob_xy_values = vec![];

    for i in 0..(sob_x_values.len()) {
        let kx = *(sob_x_values.get(i).with_context(||
            format!("No available value in the sobel X component at index {} when there should be a value at this index", i),
        )?) as u32;
        let ky = *(sob_y_values.get(i).with_context(||
            format!("No available value in the sobel Y component at index {} when there should be a value at this index", i),
        )?) as u32;
        let kxy_2 = kx * kx + ky * ky; // u8 * u8 is u16 and we sum two so we need u32
        sob_xy_values.push((kxy_2 as f64).sqrt() as u8);
    }

    let image_sobel = PhotonImage::new(sob_xy_values, width, height);

    Ok(image_sobel)
}

pub(crate) fn sobel_blend_dodge(image: PhotonImage) -> PhotonImage {
    let mut sobel = calculate_global_sobel(image).unwrap();
    desaturate(&mut sobel);
    let mut base_layer = sobel.clone();
    invert(&mut base_layer);
    gaussian_blur(&mut sobel, 3);
    blend(&mut base_layer, &sobel, "dodge");
    base_layer
}

/// Changes the midpoint of the grayscale from 122 to the new midpooint
/// Tends to darken the dark lines if midpoint is more than 127
/// Tends to erase the dark lines if midpoint is less than 127
/// What it does is assign the range 0-127 to 0-new_midpoint and 127-255 to new_midpoint-255
/// This is done by a linear correlation
/// Only the RGB values are changed, the alpha is left untouched
/// Note that you can give this function a non-greyscale value, but this function assumes R=G=B and will take the value of the red channel for all the other colour channels
pub(crate) fn change_grayscale_range_midpoint(
    image: PhotonImage,
    new_midpoint: u8,
) -> Result<PhotonImage> {
    fn new_low_range(x_value: u8, new_midpoint: u8) -> u8 {
        // we make a linear correlation y = ax + b
        // using (x1, y1) = (0,0) and (x2, y2) = (127, new_midpoint)
        // classic formula gives:
        // y = y1 + (y2 - y1)/(x2 - x1)*(x - x1)
        assert!(x_value <= new_midpoint);
        let x1 = 0;
        let y1 = 0;
        let x2 = 127;
        let y2 = new_midpoint;
        let a = ((y2 - y1) as f32) / ((x2 - x1) as f32);
        y1 + ((a * ((x2 - x1) as f32)) as u8)
    }
    fn new_high_range(x_value: u8, new_midpoint: u8) -> u8 {
        // we make a linear correlation y = ax + b
        // using (x1, y1) = (127, new_midpoint) and (x2, y2) = (255, 255)
        // classic formula gives:
        // y = y1 + (y2 - y1)/(x2 - x1)*(x - x1)
        assert!(x_value >= new_midpoint);
        let x1 = 127;
        let y1 = new_midpoint;
        let x2 = 255;
        let y2 = 255;
        let a = ((y2 - y1) as f32) / ((x2 - x1) as f32);
        y1 + ((a * ((x2 - x1) as f32)) as u8)
    }
    let width = image.get_width();
    let height = image.get_height();

    let raw_pixels = image.get_raw_pixels();
    let mut new_pixels = Vec::with_capacity(raw_pixels.len());
    // cannot just image.get_raw_pixels().chunks_exact() because of temporary value dropped while borrowed
    let mut pixel_iter = raw_pixels.chunks_exact(4);
    // using chunks of 4 because of RGBA format
    for pixel in pixel_iter.by_ref() {
        let grey_value = *pixel
            .first()
            .context("The current pixel doesn't have any value inside")?;
        let alpha = *pixel
            .get(3)
            .context("The current pixel doesn't have an alpha value")?;
        let new_gray = if grey_value <= new_midpoint {
            new_low_range(grey_value, new_midpoint)
        } else {
            new_high_range(grey_value, new_midpoint)
        };
        // keep the same alpha
        let new_pixel = vec![new_gray, new_gray, new_gray, alpha];
        new_pixels.extend(new_pixel);
    }
    // just in case the size was actually not divisible
    new_pixels.extend(pixel_iter.remainder().to_vec());

    Ok(PhotonImage::new(new_pixels, width, height))
}

/// Repeteadly blend an image with itself with the a multiply blend
/// This will darken the lines
pub(crate) fn blend_multiply_repeat(image: &mut PhotonImage, repeat_number: u8) {
    let original_image = image.clone();
    // we could do it faster by using a base 2 decomposition of repeat number (a fast exponentiation but with the images)
    for _ in 0..repeat_number {
        blend(image, &original_image, "multiply")
    }
}
