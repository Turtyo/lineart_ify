mod image_generation;
mod lineart;

use lineart::Method;

fn main() {
    let min_blur_radius = 3;
    let blur_step = 1;
    let blur_number = 5;
    let min_darken_number = 2;
    let darken_step = 1;
    let darken_number = 4;

    image_generation::generate_images_and_grid(
        "sample_images/pj_images/Annabelle Zebuth - Bobo Elite4.jpg",
        min_blur_radius,
        blur_step,
        blur_number,
        min_darken_number,
        darken_step,
        darken_number,
        Method::GaussianBlendDodge,
        "multiple_images/",
    )
    .unwrap();
}
