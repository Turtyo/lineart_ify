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

    let mut sob_x_values = sobel_x.get_raw_pixels();
    let mut sob_y_values = sobel_y.get_raw_pixels();

    let width = sobel_x.get_width();
    let height = sobel_x.get_height();

    let mut sob_xy_values = vec![];

    for _ in 0..(sob_x_values.len()) {
        let kx = sob_x_values.pop().context(
            "No more values available in the sobel X component when there should be some left",
        )? as u32;
        let ky = sob_y_values.pop().context(
            "No more values available in the sobel Y component when there should be some left",
        )? as u32;
        let kxy_2 = kx * kx + ky * ky; // u8 * u8 is u16 and we sum two so we need u32
        sob_xy_values.push((kxy_2 as f64).sqrt() as u8);
    }
    //revert the array since we've been putting at the start values we take from the end of the coefficients list
    sob_xy_values.reverse();
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
