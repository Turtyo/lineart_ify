mod image_generation;
mod lineart;

use image_generation::generate_image_grid;
use lineart::Method;
use photon_rs::native::{open_image, save_image};

fn main() {
    let image = open_image("sample_images/sacquet.png").expect("Couldn't open image");
    let other_image = image.clone();
    let lineart_image = lineart::gaussian_blend_dodge(image.clone(), 3);
    let lineart_image_2 = lineart::sobel_blend_dodge(other_image, 3);

    save_image(lineart_image, "result_images/sacquet_01.png").expect("Couldn't save image");
    save_image(lineart_image_2, "result_images/sacquet_02.png").expect("Couldn't save image");

    let min_blur_radius = 1;
    let blur_step = 1;
    let blur_number = 5;
    let min_darken_number = 0;
    let darken_step = 1;
    let darken_number = 4;

    image_generation::generate_all_images(
        "sample_images/sacquet.png",
        min_blur_radius,
        blur_step,
        blur_number,
        min_darken_number,
        darken_step,
        darken_number,
        Method::SobelBlendDodge,
        "multiple_images/",
    )
    .unwrap();

    generate_image_grid(
        min_blur_radius,
        blur_step,
        blur_number,
        min_darken_number,
        darken_step,
        darken_number,
        "multiple_images/sacquet",
    )
    .expect("Couldn't generate the image grid");
}
