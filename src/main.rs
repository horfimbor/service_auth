use tiny_http::{Server, Response, Method, StatusCode};
use frank_jwt::{Algorithm, encode};
use uuid::Uuid;

#[macro_use]
extern crate serde_json;


fn main() {
    let server = Server::http("0.0.0.0:8000").unwrap();

    for request in server.incoming_requests() {
        if request.method() != &Method::Post {
            let response = Response::new_empty(StatusCode(405));

            match request.respond(response) {
                Err(_e) => {
                    println!("cannot respond")
                }
                Ok(()) => {}
            }
        } else {
            println!("received request! method: {:?}, url: {:?}, headers: {:?}",
                     request.method(),
                     request.url(),
                     request.headers()
            );

            let payload = json!({
            "acc": Uuid::new_v4(),
            "pla": vec![Uuid::new_v4()],
        });

            let header = json!({});
            let secret = "secret123";
            let response = match encode(header, &secret.to_string(), &payload, Algorithm::HS256)
                {
                    Ok(token) => {
                        Some(Response::from_string(token))
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
    }
}

