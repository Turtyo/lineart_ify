use photon_rs::native::{open_image, save_image};
mod image_generation;
mod lineart;

fn main() {
    let image = open_image("sample_images/sacquet.png").expect("Couldn't open image");
    let other_image = image.clone();
    let lineart_image = lineart::gaussian_blend_dodge(image);
    let lineart_image_2 = lineart::sobel_blend_dodge(other_image);
    // for i in 1..=25 {
    //     let image =
    //         lineart::change_grayscale_range_midpoint(lineart_image_2.clone(), 10 * i).unwrap();
    //     let path = if i + 2 < 10 {
    //         format!("result_images/sacquet_0{}.png", i + 2)
    //     } else {
    //         format!("result_images/sacquet_{}.png", i + 2)
    //     };
    //     save_image(image, path.as_str()).expect("Couldn't save image");
    // }
    for i in 1..5 {
        let mut image_1 = lineart_image.clone();
        let mut image_2 = lineart_image_2.clone();
        lineart::blend_multiply_repeat(&mut image_1, i);
        lineart::blend_multiply_repeat(&mut image_2, i);
        let path_1 = if i + 2 < 10 {
            format!("result_images/sacquet_1_darken_0{}.png", i + 2)
        } else {
            format!("result_images/sacquet_1_darken_{}.png", i + 2)
        };
        let path_2 = if i + 2 < 10 {
            format!("result_images/sacquet_2_darken_0{}.png", i + 2)
        } else {
            format!("result_images/sacquet_2_darken_{}.png", i + 2)
        };
        save_image(image_1, path_1.as_str()).expect("Couldn't save image");
        save_image(image_2, path_2.as_str()).expect("Couldn't save image");
    }
    save_image(lineart_image, "result_images/sacquet_01.png").expect("Couldn't save image");
    save_image(lineart_image_2, "result_images/sacquet_02.png").expect("Couldn't save image");
}
