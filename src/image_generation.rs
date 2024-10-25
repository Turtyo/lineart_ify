use std::path::Path;

use crate::lineart::{self, Method};
use anyhow::{Context, Result};
use photon_rs::{multiple::blend, native::save_image, PhotonImage};

pub(crate) fn generate_all_images(
    base_image: PhotonImage,
    min_blur_radius: i32,
    blur_step: i32,
    blur_number: u8,
    min_darken_number: u8,
    darken_step: u8,
    darken_number: u8,
    method: Method,
    output_path: impl AsRef<Path>,
) -> Result<()> {
    for blur_index in 0..blur_number {
        let blur_radius = min_blur_radius + (blur_index as i32 * blur_step);
        let original_image = match method {
            Method::GaussianBlendDodge => {
                lineart::gaussian_blend_dodge(base_image.clone(), blur_radius)
            }
            Method::SobelBlendDodge => lineart::sobel_blend_dodge(base_image.clone(), blur_radius),
        };
        let mut image = original_image.clone();
        //blend the image a first time
        for _ in 0..min_darken_number {
            blend(&mut image, &original_image, "multiply")
        }
        for darken_index in 0..(darken_number - 1) {
            let mut save_path = output_path.as_ref().to_owned();
            save_path.push(format!(
                "blur_{}_darken_{}",
                blur_radius,
                min_darken_number + darken_index * darken_step
            ));
            save_path.set_extension("png");
            let save_path = save_path.to_str().with_context(|| {
                format!(
                    "The path to save the image cannot be converted to a string: {:?}",
                    save_path
                )
            })?;
            save_image(image.clone(), save_path)?;
            for _ in 0..darken_step {
                blend(&mut image, &original_image, "multiply")
            }
        }
        let mut save_path = output_path.as_ref().to_owned();
        save_path.push(format!(
            "blur_{}_darken_{}",
            blur_radius,
            min_darken_number + (darken_number -1) * darken_step
        ));
        save_path.set_extension("png");
        
        let save_path = save_path.to_str().with_context(|| {
            format!(
                "The path to save the image cannot be converted to a string: {:?}",
                save_path
            )
        })?;
        
        save_image(image, save_path)?; // save image for last iteration
    }
    Ok(())
}
