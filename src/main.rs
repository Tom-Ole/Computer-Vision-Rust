/*
################################################################################
# Reference: https://en.wikipedia.org/wiki/Feature_(computer_vision)#Detectors #
################################################################################
*/

mod canny;
mod gausian_blur;
mod harris;
mod sobel;
mod shi;

use std::{
    env, fs, io::{BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, path::PrefixComponent, str, sync::{mpsc, Arc, Mutex}, thread, time::Instant
};

use base64::{engine::general_purpose::STANDARD, Engine};
use canny::canny;
use serde_json::{json, Value};
use shi::shi;

use crate::{harris::harris, sobel::sobel};
#[derive(Clone)]
struct InputValues {
    sigma: f32,
    threshold: f32,
}

#[derive(Clone)]
struct results {
    canny: String,
    sobel: String,
    harris: String,
    shi: String,
}
#[derive(Clone)]
struct computerVison {
    image: Vec<u8>,
    sigma: f32,
    threshold: f32,
    results: results
}

impl computerVison {
    fn canny(mut self) -> String {
        let res = canny(&self.image, &self.sigma, &self.threshold);
        let encode = STANDARD.encode(&res); 
        self.results.canny = encode.clone();
        return encode;
    }
    fn sobel(mut self) -> String {
        let res = sobel(&self.image, &self.sigma);
        let encode = STANDARD.encode(&res); 
        self.results.sobel = encode.clone();
        return encode;
    }
    fn harris(mut self) -> String {
        let res = harris(&self.image);
        let encode = STANDARD.encode(&res); 
        self.results.harris = encode.clone();
        return encode;
    }
    fn shi(mut self) -> String {
        let res = shi(&self.image, &self.threshold);
        let encode = STANDARD.encode(&res); 
        self.results.shi = encode.clone();
        return encode;    
    }
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let mut port = String::from("8080");

    for i in 0..args.len() {
        if args[i] == "port" && i + 1 < args.len() {
            port = args[i+1].clone();
            break;
        }
    }


    let mut cV = computerVison {
        image: vec![0],
        sigma: 1.0,
        threshold: 0.3,
        results: results {
            canny: String::new(),
            sobel: String::new(),
            harris: String::new(),
            shi: String::new()
        }
    };

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    println!("Server running on 127.0.0.1:{}", port);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, cV.clone());
    }
}

fn handle_connection(mut stream: TcpStream, mut cV: computerVison) {
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

    let content_length = headers
        .iter()
        .filter_map(|header| {
            if header.to_lowercase().starts_with("content-length") {
                header
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| v.trim().parse::<usize>().ok())
            } else {
                None
            }
        })
        .next()
        .unwrap_or(0);

    let mut body = vec![0; content_length];
    buf_reader.read_exact(&mut body).unwrap();

    let (canny_tx, canny_rx) = mpsc::channel();
    let (sobel_tx, sobel_rx) = mpsc::channel();
    let (harris_tx, harris_rx) = mpsc::channel();
    let (shi_tx, shi_rx) = mpsc::channel();
    cV.image = body.clone();
    
    let response = match request_line.as_str() {
        "GET / HTTP/1.1" => {
            let contents = fs::read_to_string("src/client/index.html").unwrap();
            response_200(contents)
        }
        "GET /app.js HTTP/1.1" => {
            let contents = fs::read_to_string("src/client/app.js").unwrap();
            response_200(contents)
        }
        "GET /style.css HTTP/1.1" => {
            let contents = fs::read_to_string("src/client/style.css").unwrap();
            response_200(contents)
        }
        "POST /setSigma HTTP/1.1" => {
            let t = str::from_utf8(&body).unwrap().parse::<f32>().unwrap();
            cV.sigma = t;

            response_200("Ok".to_string())
        }
        "POST /setThreshold HTTP/1.1" => {
            let t = str::from_utf8(&body).unwrap().parse::<f32>().unwrap();
            cV.threshold = t;

            response_200("Ok".to_string())
        }
        "POST /canny HTTP/1.1" => {
            println!("Start Processing canny");
            let now = Instant::now();
            println!("Elapsed time: {:.2?}", now.elapsed());
            response_json(json!({
                "data": {
                    "base64": cV.canny()
                }
            }))
        }
        "POST /sobel HTTP/1.1" => {
            println!("Start Processing sobel");
            let now = Instant::now();
            println!("Elapsed time: {:.2?}", now.elapsed());
            response_json(json!({
                "data": {
                    "base64": cV.sobel()
                }
            }))
        }
        "POST /harris HTTP/1.1" => {
            println!("Start Processing Harris");
            let now = Instant::now();
            println!("Elapsed time: {:.2?}", now.elapsed());
            response_json(json!({
                "data": {
                    "base64": cV.harris()
                }
            }))
        }
        "POST /shi HTTP/1.1" => {
            println!("Start Processing Shi");
            let now = Instant::now();
            println!("Elapsed time: {:.2?}", now.elapsed());
            response_json(json!({
                "data": {
                    "base64": cV.shi()
                }
            }))
        }
        "POST /all HTTP/1.1" => {

            let body_mut = Arc::new(Mutex::new(body));
            println!("Start Processing All");
            let now = Instant::now();
            let body_clone_1 = Arc::clone(&body_mut);
            let values_clone_1 = cV.clone();
            let thread_1 = thread::spawn(move || {
                let canny_image = canny(
                    &body_clone_1.lock().unwrap(),
                    &values_clone_1.sigma,
                    &values_clone_1.threshold,
                );
                canny_tx.send(canny_image).unwrap();
            });
            
            let body_clone_2 = Arc::clone(&body_mut);
            let values_clone_2 = cV.clone();
            let thread_2 = thread::spawn(move || {
                let sobel_image = sobel(&body_clone_2.lock().unwrap(), &values_clone_2.sigma);
                sobel_tx.send(sobel_image).unwrap();
            });
            
            let body_clone_3 = Arc::clone(&body_mut);
            let thread_3 = thread::spawn(move || {
                let harris_image = harris(&body_clone_3.lock().unwrap());
                harris_tx.send(harris_image).unwrap();
            });
            
            let body_clone_4 = Arc::clone(&body_mut);
            let values_clone_4 = cV.clone();
            let thread_4 = thread::spawn(move || {
                let shi_image = shi(&body_clone_4.lock().unwrap(), &values_clone_4.threshold);
                shi_tx.send(shi_image).unwrap();
            });

            thread_1.join().unwrap();
            thread_2.join().unwrap();
            thread_3.join().unwrap();
            thread_4.join().unwrap();

            println!("Elapsed time: {:.2?}", now.elapsed());
            response_json(json!({
                "data": {
                    "canny": STANDARD.encode(&canny_rx.recv().unwrap()),
                    "sobel": STANDARD.encode(&sobel_rx.recv().unwrap()),
                    "harris": STANDARD.encode(&harris_rx.recv().unwrap()),
                    "shi": STANDARD.encode(&shi_rx.recv().unwrap())
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

