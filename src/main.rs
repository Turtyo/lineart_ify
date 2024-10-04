use photon_rs::native::{open_image, save_image};

mod lineart;

fn main() {
    let image = open_image("sample_images/sacquet.png").expect("Couldn't open image");
    let lineart_image = lineart::make_lineart(image);
    save_image(lineart_image, "result_images/sacquet.png").expect("Couldn't save image");
}
