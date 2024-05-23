
/*
################################################################################
# Reference: https://en.wikipedia.org/wiki/Feature_(computer_vision)#Detectors #
################################################################################
*/

mod gausian_blur;
mod canny;
mod sobel;
mod harris;

use std::{
    str, fs, io::{BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, time::Instant
};

use base64::{engine::general_purpose::STANDARD, Engine};
use canny::canny;
use serde_json::{json, Value};

use crate::{harris::harris, sobel::sobel};

struct InputValues {
    sigma: f32,
    threshold: f32,
}

fn main() {
    let mut values = InputValues  {
        sigma: 1.0,
        threshold: 0.3,
    };

    const PORT: usize = 8080;
    let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT)).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, &mut values);
    }
}


fn handle_connection(mut stream: TcpStream, values: &mut InputValues) {



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
        "GET /style.css HTTP/1.1" => {
            let contents = fs::read_to_string("src/html/style.css").unwrap();
            response_200(contents)
        }
        "POST /setSigma HTTP/1.1" => {
            let t = str::from_utf8(&body).unwrap().parse::<f32>().unwrap(); 
            values.sigma = t;
            
            response_200("Ok".to_string())
        }
        "POST /setThreshold HTTP/1.1" => {
            let t = str::from_utf8(&body).unwrap().parse::<f32>().unwrap(); 
            values.threshold = t;
            
            response_200("Ok".to_string())
        }
        "POST /canny HTTP/1.1" => {
            println!("Start Processing canny");
            let now = Instant::now();
                let new_image = canny(&body, &values.sigma, &values.threshold);
            println!("Elapsed time: {:.2?}", now.elapsed());
                response_json(json!({
                    "data": {
                        "base64": STANDARD.encode(&new_image)
                    }
                }))
        }
        "POST /sobel HTTP/1.1" => {
            println!("Start Processing sobel");
            let now = Instant::now();
                let new_image = sobel(&body, &values.sigma);
            println!("Elapsed time: {:.2?}", now.elapsed());
                response_json(json!({
                    "data": {
                        "base64": STANDARD.encode(&new_image)
                    }
                }))
        }
        "POST /harris HTTP/1.1" => {
            println!("Start Processing Harris");
            let now = Instant::now();
                let new_image = harris(&body);
            println!("Elapsed time: {:.2?}", now.elapsed());
                response_json(json!({
                    "data": {
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

