use std::{f32::consts::PI, io::Cursor};

use image::{imageops::grayscale, io::Reader as ImageReader, DynamicImage, GenericImage, GenericImageView, Rgba};

pub fn sobel(body: &Vec<u8>, sigma: &f32) -> Vec<u8> {

    let image = ImageReader::new(Cursor::new(body)).with_guessed_format().unwrap().decode().unwrap();

    let gray_scale_img = grayscale(&image);
    //let gausian_image = apply_gausian_filter(gray_scale_img.into(), gausian_strength).unwrap(); // My implementation
    let gausian_image = image::imageops::blur(&gray_scale_img, *sigma);


    let operators = vec![
        vec![
        vec![-1, 0, 1],
        vec![-2, 0, 2],
        vec![-1, 0, 1],
    ],
    vec![
        vec![-1, -2, -1],
        vec![0, 0, 0],
        vec![1, 2, 1],
    ]
    ];

    let (gradient_mag, _) = sobel_operator(&gausian_image.into(), operators);

    let mut image_bytes = Vec::new();
    gradient_mag.write_to(&mut Cursor::new(&mut image_bytes), image::ImageFormat::Png).unwrap(); 

    return image_bytes;
}

pub fn sobel_operator(image: &DynamicImage, operators: Vec<Vec<Vec<i32>>>) -> (DynamicImage, DynamicImage) {
    let width = image.width() as i32;
    let height = image.height() as i32;

    // Sobel operator coefficients
    let sobel_x = operators.get(0).unwrap();
    let sobel_y = operators.get(1).unwrap();

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