use std::{f32::consts::PI, io::Cursor};

// use crate::gausian_blur::apply_gausian_filter;

use image::{imageops::grayscale, io::Reader as ImageReader, DynamicImage, GenericImage, GenericImageView, Rgba};


pub fn canny(body: &Vec<u8>, gausian_strength: &f32, threshold: &f32) -> Vec<u8> {

    let img = ImageReader::new(Cursor::new(body)).with_guessed_format().unwrap().decode().unwrap();

    let gray_scale_img = grayscale(&img);
    //let gausian_image = apply_gausian_filter(gray_scale_img.into(), gausian_strength).unwrap(); // My implementation
    let gausian_image = image::imageops::blur(&gray_scale_img, *gausian_strength);

    let (gradient_mag, gradient_dir) = sobel_operator(&gausian_image.into());
    let suppressed_image = non_maximum_suppression(gradient_mag, &gradient_dir);
    let mut threshold_image = double_threshold(suppressed_image, *threshold);
    threshold_image = threshold_image.brighten(10);

    let mut image_bytes = Vec::new();
    threshold_image.write_to(&mut Cursor::new(&mut image_bytes), image::ImageFormat::Png).unwrap(); 
    
    return image_bytes;
}

fn sobel_operator(image: &DynamicImage) -> (DynamicImage, DynamicImage) {
    let width = image.width() as i32;
    let height = image.height() as i32;

    // Sobel operator coefficients
    let sobel_x = vec![
        vec![-1, 0, 1],
        vec![-2, 0, 2],
        vec![-1, 0, 1],
    ];

    let sobel_y = vec![
        vec![-1, -2, -1],
        vec![0, 0, 0],
        vec![1, 2, 1],
    ];

    let mut gradient_magnitude = DynamicImage::new_luma8(image.width(), image.height());
    let mut gradient_direction = DynamicImage::new_luma8(image.width(), image.height());

    let mut max_gradient = 0;

    // Compute gradient magnitudes
    for x in 1..width-1 {
        for y in 1..height-1 {
            let mut gx = 0.0;
            let mut gy = 0.0;

            for i in -1..=1 {
                for j in -1..=1 {
                    let pixel = image.get_pixel((x + i) as u32, (y + j) as u32).0[0] as i32;
                    gx += (pixel * sobel_x[(i + 1) as usize][(j + 1) as usize]) as f32;
                    gy += (pixel * sobel_y[(i + 1) as usize][(j + 1) as usize]) as f32;
                }
            }

            let g = f32::sqrt(gx * gx + gy * gy) as i32;
            max_gradient = max_gradient.max(g);
            gradient_magnitude.put_pixel(x as u32, y as u32, Rgba([g as u8, g as u8, g as u8, 255]));

            // Compute gradient direction
            let angle = gy.atan2(gx);
            let angle_deg = (angle * (180.0 / PI)).round() as i32;
            
            let normalized_angle = match angle_deg {
                0..=22 => 0,   // Horizontal
                23..=68 => 45,   // 45-degree diagonal
                69..=112 => 90, // vetical-degree diagonal
                113..=158 => 135, // 135-degree diagonal
                _ => 0,         // Vertical
            };

            gradient_direction.put_pixel(x as u32, y as u32, Rgba([normalized_angle as u8, normalized_angle as u8, normalized_angle as u8, 255]));
            gradient_magnitude.put_pixel(x as u32, y as u32, Rgba([g as u8, g as u8, g as u8, 255]));
        }
    }

    // // Normalize gradient magnitudes
    // for x in 1..width-1 {
    //     for y in 1..height-1 {
    //         let pixel = gradient_magnitude.get_pixel(x as u32, y as u32).0[0] as f32;
    //         let normalized_pixel = (pixel / max_gradient as f32 * 255.0).min(255.0) as u8;
    //         gradient_magnitude.put_pixel(x as u32, y as u32, Rgba([normalized_pixel, normalized_pixel, normalized_pixel, 255]));
    //     }
    // }

    (gradient_magnitude, gradient_direction)
}

fn non_maximum_suppression(grad_mag: DynamicImage, grad_dir: &DynamicImage) -> DynamicImage {

    let width = grad_mag.width();
    let height = grad_mag.height();

    let mut suppresed_image = DynamicImage::new_luma8(width, height);

    //FIXME: Get the edges of the image too.
    for x in 1..width-1 {
        for y in 1..height-1 {
            let mag = grad_mag.get_pixel(x, y).0[0];
            let direction = grad_dir.get_pixel(x, y).0[0];

            let (neigh1, neigh2) = match direction {
                0 => ((x + 1, y), (x - 1, y)),           // Horizontal
                45 => ((x + 1, y - 1), (x - 1, y + 1)),  // 45-degree diagonal
                90 => ((x, y + 1), (x, y - 1)),          // Vertical
                135 => ((x - 1, y - 1), (x + 1, y + 1)), // 135-degree diagonal
                _ => continue,
            };
            
            let neigh1_val = grad_mag.get_pixel(neigh1.0, neigh1.1).0[0];
            let neigh2_val = grad_mag.get_pixel(neigh2.0, neigh2.1).0[0];

            let pixel_value = if mag >= neigh1_val && mag >= neigh2_val {
                mag
            } else {
                0
            };

            suppresed_image.put_pixel(x, y, Rgba([pixel_value, pixel_value, pixel_value, 255]));
            
        }
    }

    return suppresed_image;
}

fn double_threshold(image: DynamicImage, threshold: f32) -> DynamicImage {

    let mut double_threshold_image = DynamicImage::new_luma8(image.width(), image.height());

    for x in 0..image.width() {
        for y in 0..image.height() {
            let value = image.get_pixel(x, y).0[0];
            if (value as f32) < threshold * 255.0 {
                double_threshold_image.put_pixel(x, y, Rgba([0,0,0,255]));
            } else {
                double_threshold_image.put_pixel(x, y, Rgba([value,value,value,255]));
            }
        }
    }

    return double_threshold_image;
}
