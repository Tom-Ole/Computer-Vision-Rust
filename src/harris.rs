use std::io::Cursor;
use image::{imageops::grayscale, io::Reader as ImageReader, DynamicImage, GenericImage, ImageBuffer, Luma, Rgba};

pub fn harris(body: &Vec<u8>) -> Vec<u8> {

    let img = ImageReader::new(Cursor::new(body)).with_guessed_format().unwrap().decode().unwrap();
    
    let gray_scale_img = grayscale(&img);
    let corner_img = harris_corner_detection(&gray_scale_img);

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


fn harris_corner_detection(image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> DynamicImage {

    let width = image.width();
    let height = image.height();

    let k = 0.04;

    let mut max_r = f64::MIN;
    let mut min_r = f64::MAX;
    
    
    let mut response = vec![vec![0.0; height as usize]; width as usize];
    let mut res_image = DynamicImage::new_luma8(image.width(), image.height());



    for x in 1..width-1 {
        for y in 1..height-1 {
            let i_x = ix(image, x, y);
            let i_y = iy(image, x, y);
            let m = vec![
                vec![i_x * i_x, i_x * i_y],
                vec![i_x * i_y ,i_y * i_y]
            ];

            let det_m = det(&m);
            let trace_m = m[0][0] + m[1][1];

            if trace_m != 0.0 {
                let r = det_m - k * trace_m * trace_m;
                response[x as usize][y as usize] = r;
                
                if r > max_r {
                    max_r = r;
                }
                if r < min_r {
                    min_r = r
                }
                
            }
            
        }
    }

    for x in 0..width {
        for y in 0..height {
            let r = response[x as usize][y as usize];
            let r_normalized = if max_r != min_r {
                ((r - min_r) / (max_r - min_r) * 255.0) as u8
            } else {
                0
            };
            res_image.put_pixel(x, y, Rgba([r_normalized, r_normalized, r_normalized, 255]));
        }
    }

    res_image.invert();

    return res_image;   
}



