use std::f32::consts::PI;

use image::{DynamicImage, GenericImage, GenericImageView, Rgba};

#[no_mangle]
pub fn apply_gausian_filter(image: DynamicImage, sigma: f32) -> Result<DynamicImage, String> {
    let width = image.width() as i32;
    let height = image.height() as i32;

    let mask_size: usize = 5;
    if mask_size % 2 == 0 {
        return Err("Mask must be an odd number".into());
    }
    let k: i32 = ((mask_size - 1) / 2) as i32;

    let mut kernel = vec![vec![0.0; mask_size]; mask_size];
    let mut sum = 0.0;

    // Precompute Gaussian kernel
    for i in -k..=k {
        for j in -k..=k {
            let value = (1.0 / (2.0 * PI * sigma * sigma)) * (-(i * i + j * j) as f32 / (2.0 * sigma * sigma)).exp();
            kernel[(i + k) as usize][(j + k) as usize] = value;
            sum += value;
        }
    }

    // Normalize kernel
    for i in 0..mask_size {
        for j in 0..mask_size {
            kernel[i][j] /= sum;
        }
    }

    let mut filtered_image = DynamicImage::new_rgba8(width as u32, height as u32);

    for x in 0..width {
        for y in 0..height {
            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;
            let mut a = 0.0;

            for i in -k..=k {
                for j in -k..=k {
                    let nx = x + i;
                    let ny = y + j;
                    if nx >= 0 && nx < width && ny >= 0 && ny < height {
                        let pixel = image.get_pixel(nx as u32, ny as u32);
                        let weight = kernel[(i + k) as usize][(j + k) as usize];
                        r += pixel[0] as f32 * weight;
                        g += pixel[1] as f32 * weight;
                        b += pixel[2] as f32 * weight;
                        a += pixel[3] as f32 * weight;
                    }
                }
            }

            filtered_image.put_pixel(x as u32, y as u32, Rgba([
                r.min(255.0) as u8,
                g.min(255.0) as u8,
                b.min(255.0) as u8,
                a.min(255.0) as u8,
            ]));
        }
    }

    Ok(filtered_image)
}