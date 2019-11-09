use tiny_http::{Server, Response, Method, StatusCode, Request};
use frank_jwt::{Algorithm, encode};
use uuid::Uuid;

#[macro_use]
extern crate serde_json;


fn main() {
    let server = Server::http("0.0.0.0:8000").unwrap();

    println!("listening on 8000");

    for request in server.incoming_requests() {
        println!("received request!\n, method: {:?}\n, url: {:?}\n, headers: {:?}\n",
                 request.method(),
                 request.url(),
                 request.headers()
        );
        if request.method() == &Method::Post {
            handle_post(request)
        } else if request.method() == &Method::Get {
            let response = Response::new_empty(StatusCode(403));

            match request.respond(response) {
                Err(_e) => {
                    println!("cannot respond")
                }
                Ok(()) => {}
            }
        } else {
            let response = Response::new_empty(StatusCode(405));

            match request.respond(response) {
                Err(_e) => {
                    println!("cannot respond")
                }
                Ok(()) => {}
            }
        }
    }
}

fn handle_post(request: Request) -> () {
    let payload = json!({
            "acc": Uuid::new_v4(),
            "pla": vec![Uuid::new_v4()],
        });
    let header = json!({});
    let secret = "secret123";
    let response = match encode(header, &secret.to_string(), &payload, Algorithm::HS256)
        {
            Ok(token) => {
                let mut response = Response::from_string(token.clone());
                let bearer = format!("jwt={}", token);
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

        match request.respond(response) {
            Err(_e) => {
                println!("cannot respond")
            }
            Ok(()) => {}
        }
    } else {
        match request.respond(response.unwrap()) {
            Err(_e) => {
                println!("cannot respond")
            }
            Ok(()) => {}
        }
    }
}

