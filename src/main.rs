use photon_rs::native::{open_image, save_image};
mod lineart;

fn main() {
    let image = open_image("sample_images/sacquet.png").expect("Couldn't open image");
    let other_image = image.clone();
    let lineart_image = lineart::gaussian_blend_dodge(image);
    let lineart_image_2 = lineart::sobel_blend_dodge(other_image);
    save_image(lineart_image, "result_images/sacquet_1.png").expect("Couldn't save image");
    save_image(lineart_image_2, "result_images/sacquet_2.png").expect("Couldn't save image");
}
