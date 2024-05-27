use std::io::Cursor;

use image::{imageops::grayscale, io::Reader as ImageReader, DynamicImage, GenericImage, ImageBuffer, Luma, Rgba};

pub fn shi(body: &Vec<u8>, threshold: f32) -> Vec<u8> {

    let img = ImageReader::new(Cursor::new(body)).with_guessed_format().unwrap().decode().unwrap();

    let gray_scale_img = grayscale(&img);
    let corner_img = shi_corner_detection(&gray_scale_img, threshold as f64);

    let mut image_bytes = Vec::new();
    corner_img.write_to(&mut Cursor::new(&mut image_bytes), image::ImageFormat::Png).unwrap(); 

    return image_bytes;
}

fn ix(image: &ImageBuffer<Luma<u8>, Vec<u8>>, x: u32, y: u32) -> f64 {
    let x1 = if x > 0 { x - 1 } else { x };
    let x2 = if x < image.width() - 1 { x + 1 } else { x };
    return (image.get_pixel(x2, y).0[0] as f64 - image.get_pixel(x1, y).0[0] as f64) / 2.0;
}
fn iy(image: &ImageBuffer<Luma<u8>, Vec<u8>>, x: u32, y: u32) -> f64 {
    let y1 = if y > 0 { y - 1 } else { y };
    let y2 = if y < image.height() - 1 { y + 1 } else { y };
    return (image.get_pixel(x, y2).0[0] as f64 - image.get_pixel(x, y1).0[0] as f64) / 2.0;
}

fn det(m: &Vec<Vec<f64>>) -> f64 {
    return m[0][0] * m[1][1] - m[1][0] * m[0][1];
}

fn trace(m: &Vec<Vec<f64>>) -> f64 {
    return m[0][0] + m[1][1];
}

fn shi_corner_detection(image: &ImageBuffer<Luma<u8>, Vec<u8>>, threshold: f64) -> DynamicImage {
    let width = image.width();
    let height = image.height();

    let mut shi_image = DynamicImage::new_luma8(width, height);

    let window_size = 3;
    let offset = window_size / 2;
    let k: f64 = 0.04;

    for y in offset..height-offset {
        for x in offset..width-offset {

            let mut sum_ix2 = 0.0;
            let mut sum_iy2 = 0.0;
            let mut sum_ixiy = 0.0;

            for j in 0..window_size {
                for i in 0..window_size {
                    let ix = ix(image, x + i as u32 - offset, y + j as u32 - offset);
                    let iy = iy(image, x + i as u32 - offset, y + j as u32 - offset);
                    sum_ix2 += ix * ix;
                    sum_iy2 += iy * iy;
                    sum_ixiy += ix * iy;
                }
            }

            let a = vec![vec![sum_ix2, sum_ixiy], vec![sum_ixiy, sum_iy2]];
            let det_a = det(&a);
            let trace_a = trace(&a);
            let mc = det_a - k * trace_a * trace_a;

            if mc > threshold {
                shi_image.put_pixel(x, y, Rgba([255,255,255,255]));
            }

        }
    }

    return shi_image;
}