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
