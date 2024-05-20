use std::{
    f32::consts::{FRAC_PI_4, PI}, fs, io::{BufRead, BufReader, Cursor, Read, Write}, net::{TcpListener, TcpStream}, os::windows
};

use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use image::{codecs::jpeg::PixelDensity, imageops::grayscale, io::Reader as ImageReader, DynamicImage, GenericImage, GenericImageView, Pixel, Rgba};

fn main() {
    const PORT: usize = 8080;
    let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT)).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut request_line = "".to_string();
    buf_reader.read_line(&mut request_line).unwrap();

    request_line = request_line.replace("\r\n", "");

    let mut header_line = String::new();
    let mut headers = Vec::new();

    loop {
        header_line.clear();
        buf_reader.read_line(&mut header_line).unwrap();
        if header_line == "\r\n" {
            break;
        }
        headers.push(header_line.clone());
    }

    let content_length = headers.iter()
        .filter_map(|header| {
            if header.to_lowercase().starts_with("content-length") {
                header.split_whitespace().nth(1).and_then(|v| v.trim().parse::<usize>().ok())
            } else {
                None
            }
        })
        .next()
        .unwrap_or(0);
    
    let mut body = vec![0; content_length];
    buf_reader.read_exact(&mut body).unwrap();

    let response = match request_line.as_str() {
        "GET / HTTP/1.1" => {
            let contents = fs::read_to_string("src/html/index.html").unwrap();
            response_200(contents)
        }
        "GET /app.js HTTP/1.1" => {
            let contents = fs::read_to_string("src/html/app.js").unwrap();
            response_200(contents)
        }
        "GET /test HTTP/1.1" => {
            response_json(json!({
                "data": ["test", 12, {"a": "hello", "b": "world"}]
            }))
        }
        "POST /loadImage HTTP/1.1" => {
                let new_image = canny_edge(&body);
                response_json(json!({
                    "data": {
                        "bytes": new_image.clone(),
                        "base64": STANDARD.encode(&new_image)
                    }
                }))
        }
        _ => {
            println!("Request line: {}", request_line);
            response_404()
        }
    };

    stream.write_all(response.as_bytes()).unwrap();
}

fn response_200(contents: String) -> String {
    let status_line = "HTTP/1.1 200 OK";
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    return response;
}

fn response_404() -> String {
    let status_line = "HTTP/1.1 404 Not Found";
    let contents = "<h1>404</h1>";
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    return response;
}

fn response_json(data: Value) -> String {
    let status_line = "HTTP/1.1 202 Ok";
    let contents = data.to_string();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    return response;
}

fn canny_edge(body: &Vec<u8>) -> Vec<u8> {

    let mut img = ImageReader::new(Cursor::new(body)).with_guessed_format().unwrap().decode().unwrap();
    let width = img.width();
    let height = img.height();

    // let pix = image::Rgba([240,100,190,255]);

    // for x in 0..width {
    //     for y in 0..height {
    //         if x % 2 == 0 && y % 2 != 0 {            
    //             let mut pixel = img.get_pixel(x, y);
    //             pixel.blend(&pix);
    //             img.put_pixel(x, y, pixel);
    //         }
    //     }
        
    // }

    let gray_scale_img = grayscale(&img);
    let gausian_image = apply_gausian_filter(gray_scale_img.into(), 1.0).unwrap();

    let (gradient_mag, gradient_dir) = sobel_operator(&gausian_image);

    let mut image_bytes = Vec::new();
    gradient_mag.write_to(&mut Cursor::new(&mut image_bytes), image::ImageFormat::Png).unwrap(); 
    return image_bytes;
}


fn apply_gausian_filter(image: DynamicImage, sigma: f32) -> Result<DynamicImage, String> {
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
            let x = j as f32;
            let y = i as f32;
            let value = (1.0 / (2.0 * PI * sigma * sigma)) * (-(x * x + y * y) / (2.0 * sigma * sigma)).exp();
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

// edge detection operator
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
            let angle_deg = (angle * 180.0 / PI).round() as i32;
            let normalized_angle = match angle_deg {
                -22..=22 => 0,   // Horizontal
                23..=67 => 45,   // 45-degree diagonal
                -67..=-23 => 135, // 135-degree diagonal
                _ => 90,         // Vertical
            };
            gradient_direction.put_pixel(x as u32, y as u32, Rgba([normalized_angle as u8, normalized_angle as u8, normalized_angle as u8, 255]));
        }
    }

    // Normalize gradient magnitudes
    for x in 1..width-1 {
        for y in 1..height-1 {
            let pixel = gradient_magnitude.get_pixel(x as u32, y as u32).0[0] as f32;
            let normalized_pixel = (pixel / max_gradient as f32 * 255.0).min(255.0) as u8;
            gradient_magnitude.put_pixel(x as u32, y as u32, Rgba([normalized_pixel, normalized_pixel, normalized_pixel, 255]));
        }
    }

    (gradient_magnitude, gradient_direction)
}
