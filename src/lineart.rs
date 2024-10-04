use photon_rs::{
    channels::invert, conv::gaussian_blur, monochrome::desaturate, multiple::blend, PhotonImage,
};

pub(crate) fn make_lineart(mut image: PhotonImage) -> PhotonImage {
    desaturate(&mut image);
    let mut blend_layer = image.clone();
    invert(&mut blend_layer);
    gaussian_blur(&mut blend_layer, 3);
    blend(&mut image, &blend_layer, "dodge");
    image
}
