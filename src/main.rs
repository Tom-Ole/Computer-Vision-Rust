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
    env, fs, io::{BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, str, sync::{mpsc, Arc, Mutex}, thread, time::Instant
};
use base64::{engine::general_purpose::STANDARD, Engine};
use canny::canny;
use shi::shi;
use serde_json::{json, Value};

use crate::{harris::harris, sobel::sobel};


#[derive(Clone, Debug)]
struct Results {
    canny: String,
    sobel: String,
    harris: String,
    shi: String,
}
#[derive(Clone, Debug)]
struct ComputerVison {
    image: Vec<u8>,
    sigma: f32,
    threshold: f32,
    results: Results
}

impl ComputerVison {
    fn canny(&mut self) -> String {
        let res = canny(&self.image, &self.sigma, &self.threshold);
        let encode = STANDARD.encode(&res); 
        self.results.canny = encode.clone();
        return encode;
    }
    fn sobel(&mut self) -> String {
        let res = sobel(&self.image, &self.sigma);
        let encode = STANDARD.encode(&res); 
        self.results.sobel = encode.clone();
        return encode;
    }
    fn harris(&mut self) -> String {
        let res = harris(&self.image);
        let encode = STANDARD.encode(&res); 
        self.results.harris = encode.clone();
        return encode;
    }
    fn shi(&mut self) -> String {
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

    let cv = Arc::new(Mutex::new(ComputerVison {
        image: vec![0],
        sigma: 1.0,
        threshold: 0.3,
        results: Results {
            canny: String::new(),
            sobel: String::new(),
            harris: String::new(),
            shi: String::new()
        }
    }));


    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    println!("Server running on 127.0.0.1:{}", port);

    for stream in listener.incoming() {
        print!("Hello");
        let stream = stream.unwrap();
        handle_connection(stream, cv.clone());
    }
}

fn handle_connection(mut stream: TcpStream, cv: Arc<Mutex<ComputerVison>>) {

    
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
            let mut cv = cv.lock().unwrap();
            cv.sigma = t;

            response_200("Ok".to_string())
        }
        "POST /setThreshold HTTP/1.1" => {
            let t = str::from_utf8(&body).unwrap().parse::<f32>().unwrap();
            let mut cv = cv.lock().unwrap();
            cv.threshold = t;

            response_200("Ok".to_string())
        }
        "POST /canny HTTP/1.1" => {
            println!("Start Processing canny");
            let now = Instant::now();
            println!("Elapsed time: {:.2?}", now.elapsed());
            let base64_image = {
                let mut cv = cv.lock().unwrap();
                cv.canny()
            };
            response_json(json!({
                "data": {
                    "base64": base64_image
                }
            }))
        }
        "POST /sobel HTTP/1.1" => {
            println!("Start Processing sobel");
            let now = Instant::now();
            println!("Elapsed time: {:.2?}", now.elapsed());
            let base64_image = {
                let mut cv = cv.lock().unwrap();
                 cv.sobel() 
                };
            response_json(json!({
                "data": {
                    "base64": base64_image
                }
            }))
        }
        "POST /harris HTTP/1.1" => {
            println!("Start Processing Harris");
            let now = Instant::now();
            println!("Elapsed time: {:.2?}", now.elapsed());
            let base64_image = {
                let mut cv = cv.lock().unwrap();
                cv.harris() 
            };
            response_json(json!({
                "data": {
                    "base64": base64_image
                }
            }))
        }
        "POST /shi HTTP/1.1" => {
            println!("Start Processing Shi");
            let now = Instant::now();
            println!("Elapsed time: {:.2?}", now.elapsed());
            let base64_image = {
                let mut cv = cv.lock().unwrap();
                cv.shi()
            };
            response_json(json!({
                "data": {
                    "base64": base64_image
                }
            }))
        }
        "POST /all HTTP/1.1" => {
            let (canny_tx, canny_rx) = mpsc::channel();
            let (sobel_tx, sobel_rx) = mpsc::channel();
            let (harris_tx, harris_rx) = mpsc::channel();
            let (shi_tx, shi_rx) = mpsc::channel();

            let cv_arc = Arc::clone(&cv);

            println!("Start Processing All");
            let now = Instant::now();

            let canny_handle = {
                let cv_clone = Arc::clone(&cv_arc);
                thread::spawn(move || {
                   let mut cv = cv_clone.lock().unwrap();
                   let canny_image = cv.canny(); 
                   canny_tx.send(canny_image).unwrap();
                })
            };
            let sobel_handle = {
                let cv_clone = Arc::clone(&cv_arc);
                thread::spawn(move || {
                   let mut cv = cv_clone.lock().unwrap();
                   let sobel_image = cv.sobel(); 
                   sobel_tx.send(sobel_image).unwrap();
                })
            };
            let harris_handle = {
                let cv_clone = Arc::clone(&cv_arc);
                thread::spawn(move || {
                   let mut cv = cv_clone.lock().unwrap();
                   let harris_image = cv.harris(); 
                   harris_tx.send(harris_image).unwrap();
                })
            };
            let shi_handle = {
                let cv_clone = Arc::clone(&cv_arc);
                thread::spawn(move || {
                   let mut cv = cv_clone.lock().unwrap();
                   let shi_image = cv.shi(); 
                   shi_tx.send(shi_image).unwrap();
                })
            };


            canny_handle.join().unwrap();
            sobel_handle.join().unwrap();
            harris_handle.join().unwrap();
            shi_handle.join().unwrap();

            println!("Elapsed time: {:.2?}", now.elapsed());

            response_json(json!({
                "data": {
                    "canny": canny_rx.recv().unwrap(),
                    "sobel": sobel_rx.recv().unwrap(),
                    "harris": harris_rx.recv().unwrap(),
                    "shi": shi_rx.recv().unwrap()
                }
            }))
        }
        _ => {
            println!("Request line: {}", request_line);
            response_404()
        }
    };

    println!("sigma: {}, threshold: {}", cv.lock().unwrap().sigma, cv.lock().unwrap().threshold);

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

