extern crate serde_json;

use std::io::Error as IoError;
use std::path::Path;
use std::fs;

use crypto::sha1::Sha1;
use crypto::digest::Digest;
use tiny_http::{Method, Request, Response, Server, StatusCode};
use uuid::Uuid;

const COOKIE_PREFIX: &str = "token";


fn main() {
    let server = Server::http("0.0.0.0:8000").unwrap();

    println!("listening on 8000");

    for request in server.incoming_requests() {
        println!("received request!\n, method: {:?}\n, url: {:?}\n, headers: {:?}\n",
                 request.method(),
                 request.url(),
                 request.headers(),
        );
        match handle(request) {
            Err(_e) => {
                println!("cannot respond")
            }
            Ok(()) => {}
        }
    }
}

fn handle(request: Request) -> Result<(), IoError> {
    if request.method() == &Method::Post {
        handle_post(request)
    } else if request.method() == &Method::Get {
        handle_get(request)
    } else {
        let response = Response::new_empty(StatusCode(405));
        request.respond(response)
    }
}

fn handle_post(mut request: Request) -> Result<(), IoError> {
    let data = mod_token::Data::new(Uuid::new_v4());

    let mut content = String::new();
    request.as_reader().read_to_string(&mut content).unwrap();

    if &content[..11] != "passphrase=" {
        let response = Response::new_empty(StatusCode(500));
        return request.respond(response);
    }

    let mut hasher = Sha1::new();
    hasher.input_str(&content[11..]);

    let hex = hasher.result_str();


    let response = match mod_token::generate_token(data)
        {
            Ok(token) => {
                let mut response = Response::from_string(token.clone());
                let bearer = format!("{}={}", COOKIE_PREFIX, token);
                let header = tiny_http::Header::from_bytes(&b"Set-Cookie"[..], bearer.as_bytes()).unwrap();

                response.add_header(header);
                Some(response)
            }

            Err(_e) => {
                None
            }
        };
    if response.is_none() {
        let response = Response::new_empty(StatusCode(500));

        request.respond(response)
    } else {
        request.respond(response.unwrap())
    }
}

fn handle_get(request: Request) -> Result<(), IoError> {
    let mut authorized = false;

    println!("{:?}", request);

    let url = request.url().to_string();
    let path = Path::new(&url);
    let file = fs::File::open(&path);

    if file.is_ok() {
        let mut response = tiny_http::Response::from_file(file.unwrap());
        let header = tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/javascript"[..]).unwrap();

        response.add_header(header);

        return request.respond(response);
    }


    for h in request.headers() {
        if h.field.equiv("cookie") {
            let value = h.value.to_string();

            let cookie_values: Vec<&str> = value.split("; ").collect();

            for cookie_value in cookie_values {
                if !authorized {
                    let split: Vec<&str> = cookie_value.split("=").collect();
                    println!("{:?}", split);
                    if split.get(0) == Some(&COOKIE_PREFIX) {
                        match split.get(1) {
                            Some(&key) => {
                                authorized = mod_token::check_token(&key);
                            }
                            None => {}
                        }
                    }
                }
            }
        }
    }
    if !authorized {
        //TODO
    }

    if authorized {
        let response = Response::from_string("SUCCESS".to_string());
        request.respond(response)
    } else {
        let response = Response::new_empty(StatusCode(403));
        request.respond(response)
    }
}

