use std::{fs, path::Path};

use crate::lineart::{self, Method};
use ab_glyph::FontRef;
use anyhow::{Context, Result};
use image::{ExtendedColorType, ImageBuffer, ImageFormat, Rgba};
use imageproc::drawing::draw_text_mut;
use photon_rs::{
    multiple::blend,
    native::{open_image, save_image},
};

pub(crate) fn generate_all_images(
    base_image_path: impl AsRef<Path>,
    min_blur_radius: i32,
    blur_step: i32,
    blur_number: u8,
    min_darken_number: u8,
    darken_step: u8,
    darken_number: u8,
    method: Method,
    output_dir: impl AsRef<Path>,
) -> Result<()> {
    let mut output_dir_for_images = output_dir.as_ref().to_owned();
    let filename = base_image_path.as_ref().file_stem().with_context(|| {
        format!(
            "No filename found in file path: {:?}",
            base_image_path.as_ref()
        )
    })?;
    output_dir_for_images.push(filename); // add filename without extension, get it from filepath

    // create directory if it doesn't exist
    let directory_exists = output_dir_for_images.try_exists()?;
    if !directory_exists {
        fs::create_dir_all(&output_dir_for_images)?;
    }
    let base_image = open_image(base_image_path)?;
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
            let save_path = build_image_path(
                &output_dir_for_images,
                blur_radius,
                min_darken_number + darken_index * darken_step,
            )?;
            println!("{}", save_path);
            save_image(image.clone(), save_path.as_str())?;
            for _ in 0..darken_step {
                blend(&mut image, &original_image, "multiply")
            }
        }
        let save_path = build_image_path(
            &output_dir_for_images,
            blur_radius,
            min_darken_number + (darken_number - 1) * darken_step,
        )?;

        save_image(image, save_path.as_str())?; // save image for last iteration
    }
    Ok(())
}

fn build_image_path(image_dir: impl AsRef<Path>, blur: i32, darken: u8) -> Result<String> {
    let mut save_path = image_dir.as_ref().to_owned();
    save_path.push(format!("blur_{}_darken_{}", blur, darken));
    save_path.set_extension("png");
    let save_path = save_path.to_str().with_context(|| {
        format!(
            "The path to save the image cannot be converted to a string: {:?}",
            save_path
        )
    })?;
    Ok(save_path.to_string())
}

pub(crate) fn generate_image_grid(
    min_blur_radius: i32,
    blur_step: i32,
    blur_number: u8,
    min_darken_number: u8,
    darken_step: u8,
    darken_number: u8,
    input_dir: impl AsRef<Path>,
) -> Result<()> {
    let right_padding_mult: f32 = 1.2;
    let down_padding_mult: f32 = 1.1;
    let top_padding_mult: f32 = 0.6;
    let left_padding_mult: f32 = 1.3;

    //load a first image to get the dimensions and extrapolate the size of the final image
    let first_image_path = build_image_path(&input_dir, min_blur_radius, min_darken_number)?;
    let first_image = open_image(first_image_path.as_str())?;
    let first_width = first_image.get_width();
    let first_height = first_image.get_height();
    let left_padding = (first_width as f32) * left_padding_mult;
    let top_padding = (first_height as f32) * top_padding_mult;
    let total_width =
        (first_width as f32 * right_padding_mult) * (darken_number as f32) + left_padding; // darken by rows
    let total_height =
        (first_height as f32 * down_padding_mult) * (blur_number as f32) + top_padding; // blur by columns
    let total_width = total_width as u32;
    let total_height = total_height as u32;

    // data for the text
    let pixel_to_repeat: Rgba<u8> = Rgba([255, 255, 255, 255]);
    let mut canvas = ImageBuffer::from_pixel(total_width, total_height, pixel_to_repeat);
    let font = FontRef::try_from_slice(include_bytes!("../fonts/Exo2-Light.otf"))?;
    let text_color = Rgba([0_u8, 0_u8, 0_u8, 255_u8]);
    let scale = (first_width as f32) / 3_f32;

    // constant positions for the text
    let blur_text_x = (left_padding / 2_f32) as i32;
    let darken_text_position_y = top_padding as i32 - (first_height as f32 / 3_f32) as i32;

    for blur_index in 0..blur_number {
        let blur_radius = min_blur_radius + (blur_index as i32 * blur_step);
        let image_y =
            ((first_height as f32 * down_padding_mult) * (blur_index as f32) + top_padding) as i64;
        if blur_index == 0 {
            draw_text_mut(
                &mut canvas,
                text_color,
                0,
                image_y as i32,
                scale,
                &font,
                "Blur",
            );

            let darken_text_position_x = (first_width as f32 / 3_f32) as i32;
            draw_text_mut(
                &mut canvas,
                text_color,
                darken_text_position_x,
                darken_text_position_y,
                scale,
                &font,
                "Darken",
            );
        }
        let blur_text_y = image_y as i32 + (first_height as f32 / 2_f32) as i32;

        draw_text_mut(
            &mut canvas,
            text_color,
            blur_text_x,
            blur_text_y,
            scale,
            &font,
            format!("{}", blur_radius).as_str(),
        );

        for darken_index in 0..darken_number {
            let darken = min_darken_number + darken_index * darken_step;
            let fetch_path = build_image_path(&input_dir, blur_radius, darken)?;
            let image = image::ImageReader::open(fetch_path)?.decode()?;
            let image_x =
                (first_width as f32 * right_padding_mult) * (darken_index as f32) + left_padding;
            image::imageops::overlay(&mut canvas, &image, image_x as i64, image_y);

            if blur_index == 0 {
                let darken_text_position_x = image_x as i32 + (first_width as f32 / 2_f32) as i32;
                draw_text_mut(
                    &mut canvas,
                    text_color,
                    darken_text_position_x,
                    darken_text_position_y,
                    scale,
                    &font,
                    format!("{}", darken).as_str(),
                );
            }
        }
    }

    let mut canvas_dir_out = input_dir.as_ref().to_owned();
    canvas_dir_out.push("summary");
    canvas_dir_out.set_extension("png");

    image::save_buffer_with_format(
        canvas_dir_out,
        &canvas,
        total_width,
        total_height,
        ExtendedColorType::Rgba8,
        ImageFormat::Png,
    )?;
    Ok(())
}
