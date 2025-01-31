mod image_generation;
mod lineart;

use std::{
    ffi::OsStr,
    fs::{self, DirEntry},
    path::PathBuf,
};

use lineart::Method;

use clap::Parser;

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
struct Input {
    /// The path to the input image. Mutually exclusive with `input_directory`
    #[arg(long, short = 'i')]
    input_image: Option<PathBuf>,
    /// The path to the input directory where the images are, this does not work recursively. Mutually exclusive with `input_image`
    #[arg(long, short = 'd')]
    input_directory: Option<PathBuf>,
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[clap(flatten)]
    input: Input,
    /// The directory to output the images (if it doesn't exist, it will be created, recursively)
    /// The actual path where the image will be is `output_dir`/image_name/
    /// With the name of the image being extracted from the `input_image` path
    #[arg(long, short, default_value_t = String::from("./multiple_images"), verbatim_doc_comment)]
    output_dir: String,
    /// The x size of the output image, this is used together with the `target_size_y`
    /// We resize the image to keep the same image ratio and to get an area equals to target_size_x * target_size_y
    /// It means that the actual output image might not have the exact target_size_x if the image ratio of the input is not the same as the target_size ratio
    #[arg(long, short = 'x', default_value_t = 500, verbatim_doc_comment)]
    target_size_x: u32,
    /// The y size of the output image, this is used together with the `target_size_x`
    /// We resize the image to keep the same image ratio and to get an area equals to target_size_x * target_size_y
    /// It means that the actual output image might not have the exact target_size_y if the image ratio of the input is not the same as the target_size ratio
    #[arg(long, short = 'y', default_value_t = 600, verbatim_doc_comment)]
    target_size_y: u32,
    /// The smallest blur radius that will be used by either the Gaussian blur. Note that both the Gaussian and the Sobel methods use a Gaussian blur
    /// This can be used for both methods
    #[arg(long, default_value_t = 3)]
    min_blur_radius: i32,
    /// How much to change the blur radius between each image
    #[arg(long, default_value_t = 1)]
    blur_step: i32,
    /// How many different images should be made by varying the blur radius
    /// For the image i (between 0 and `blur_number`-1), the blur radius will be `min_blur_radius` + i * `blur_step`
    #[arg(long, default_value_t = 5, verbatim_doc_comment)]
    blur_number: u8,
    /// The lowest amount of darken rounds that must be used. Darken is done by blending the image with itself each round, which darkens the lines
    #[arg(long, default_value_t = 2)]
    min_darken_number: u8,
    /// How much to increase the number of darken rounds for each new image when changing the darken
    #[arg(long, default_value_t = 1)]
    darken_step: u8,
    /// How many different images should be made by varying the darken
    /// For the image i (between 0 and `darken_number`-1), the number of darken rounds will be `min_darken_number` + i * `darken_step`
    #[arg(long, default_value_t = 4)]
    darken_number: u8,
    /// The method to use when generating the lineart. Depending on your image, one method can work better than the other.
    #[arg(value_enum, long, short = 'm', default_value_t = Method::Gaussian)]
    method: Method,
}

fn main() {
    let cli = Cli::parse();
    let target_size = (cli.target_size_x, cli.target_size_y);
    let min_blur_radius = cli.min_blur_radius;
    let blur_step = cli.blur_step;
    let blur_number = cli.blur_number;
    let min_darken_number = cli.min_darken_number;
    let darken_step = cli.darken_step;
    let darken_number = cli.darken_number;
    let method = cli.method;
    let output_dir = PathBuf::from(cli.output_dir);

    if let Some(input_image) = cli.input.input_image {
        image_generation::generate_images_and_grid(
            input_image,
            target_size,
            min_blur_radius,
            blur_step,
            blur_number,
            min_darken_number,
            darken_step,
            darken_number,
            method,
            output_dir,
        )
        .unwrap();
    } else if let Some(input_directory) = cli.input.input_directory {
        for path in fs::read_dir(input_directory).unwrap() {
            if check_file_type_is_image(&path) {
                // we can unwrap since check_file_type_is_image returns false when we can't unwrap
                let input_image = path.unwrap().path();
                match image_generation::generate_images_and_grid(
                    input_image,
                    target_size,
                    min_blur_radius,
                    blur_step,
                    blur_number,
                    min_darken_number,
                    darken_step,
                    darken_number,
                    method,
                    &output_dir,
                ) {
                    Ok(_) => continue,
                    Err(e) => println!("{:?}", e),
                }
            }
        }
    } else {
        println!("No input image or input directory has been supplied, please use --help to see options, exiting.")
    }
}

fn check_file_type_is_image(path: &Result<DirEntry, std::io::Error>) -> bool {
    if let Ok(dir_entry) = path {
        let path = dir_entry.path();
        if path.is_file() {
            return matches!(
                path.extension().and_then(OsStr::to_str),
                Some("png") | Some("jpg") | Some("jpeg")
            );
        }
    }
    false
}
