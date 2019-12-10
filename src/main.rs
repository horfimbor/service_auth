#[macro_use]
extern crate mysql;

extern crate serde_json;

use std::io::{Error as IoError, Cursor};
use std::path::Path;
use std::fs;

use crypto::sha1::Sha1;
use crypto::digest::Digest;
use mysql::Pool;
use serde::Deserialize;
use tiny_http::{Method, Request, Response, Server, StatusCode};
use uuid::Uuid;

const COOKIE_PREFIX: &str = "token";

#[derive(Deserialize)]
struct Login {
    passphrase: String
}

#[derive(Debug, PartialEq, Eq)]
struct Account {
    passphrase: String,
    uuid: String,
    is_admin: bool,
}

fn main() {

    let pool = init_bdd();

    let server = Server::http("0.0.0.0:8000").unwrap();

    println!("listening on 8000");

    for request in server.incoming_requests() {
        println!("received request!\n, method: {:?}\n, url: {:?}\n, headers: {:?}\n",
                 request.method(),
                 request.url(),
                 request.headers(),
        );
        match handle(request, &pool) {
            Err(_e) => {
                println!("cannot respond")
            }
            Ok(()) => {}
        }
    }
}

fn init_bdd() -> Pool{
    // TODO check env == dev

    let pool = match mysql::Pool::new("mysql://root:rootpwd@infra_dev_db_1:3306/service_auth"){
        Ok(pool) => {
            pool
        }
        Err(_e) => {
            let tmp_pool = mysql::Pool::new("mysql://root:rootpwd@infra_dev_db_1:3306/mysql").unwrap();
            tmp_pool.prep_exec(r"CREATE database service_auth", ()).unwrap();

            mysql::Pool::new("mysql://root:rootpwd@infra_dev_db_1:3306/service_auth").unwrap()
        }
    };

    pool.prep_exec(r"CREATE TABLE if not exists account (
                         passphrase varchar(50) not null,
                         uuid varchar(50) not null,
                         is_admin tinyint not null default 0
                     )", ()).unwrap();

    pool
}

fn handle(request: Request, pool: &Pool) -> Result<(), IoError> {
    if request.method() == &Method::Post {
        handle_post(request, &pool)
    } else if request.method() == &Method::Get {
        handle_get(request)
    } else if request.method() == &Method::Options {
        handle_option(request)
    } else {
        let response = Response::new_empty(StatusCode(405));
        request.respond(response)
    }
}

fn handle_post(mut request: Request, pool: &Pool) -> Result<(), IoError> {
    let mut content = String::new();
    request.as_reader().read_to_string(&mut content).unwrap();

    let deserialize: serde_json::Result<Login> = serde_json::from_str(&content.as_str());

    match deserialize {
        Ok(login) => {
            return handle_login(request, login, &pool);
        }
        Err(_e) => {
            let response = Response::new_empty(StatusCode(500));
            return request.respond(response);
        }
    }
}

fn handle_login(request: Request, login: Login, pool: &Pool) -> Result<(), IoError> {
    let mut hasher = Sha1::new();

    // TODO add salt
    hasher.input_str(login.passphrase.as_str());

    let encoded_pass_phrase = hasher.result_str();


    let uuid = get_uuid(encoded_pass_phrase, pool);

    let data = mod_token::Data::new(uuid);

    let response = match mod_token::generate_token(data)
        {
            Ok(token) => {
                let mut response = Response::from_string(token.clone());
                let bearer = format!("{}={}", COOKIE_PREFIX, token);
                let header = tiny_http::Header::from_bytes(&b"Set-Cookie"[..], bearer.as_bytes()).unwrap();

                response.add_header(header);

                add_cors(&mut response);
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

fn get_uuid(encoded_pass_phrase: String, pool: &Pool) -> Uuid {
    let pp = encoded_pass_phrase.clone();
    let accounts: Vec<Account> =
        pool.prep_exec("SELECT passphrase, uuid, is_admin FROM account WHERE passphrase = :pp", params! {pp})
            .map(|result| {
                result.map(|x| x.unwrap())
                    .map(|row| {
                        let (passphrase, uuid, is_admin) = mysql::from_row(row);
                        Account {
                            passphrase,
                            uuid,
                            is_admin,
                        }
                    }).collect()
            }).unwrap();
    if accounts.len() == 0 {
        let uuid = Uuid::new_v4();

        pool.prep_exec("INSERT INTO account (passphrase, uuid) VALUES (:pp , :uuid)",
                       params! {"pp" => encoded_pass_phrase, "uuid" => uuid.to_string() }).unwrap();

        return uuid;
    } else {
        let acc = accounts.first().unwrap();
        return Uuid::parse_str(acc.uuid.as_str()).unwrap();
    }
}

fn handle_get(request: Request) -> Result<(), IoError> {
    let mut authorized = false;

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

fn handle_option(request: Request) -> Result<(), IoError> {

    // TODO check "Host"

    let mut response = Response::from_string("ok");

    add_cors(&mut response);

    request.respond(response)
}

fn add_cors(response: &mut Response<Cursor<Vec<u8>>>) {
    let header = tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"http://localhost"[..]).unwrap();
    response.add_header(header);
    let header = tiny_http::Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"POST, GET"[..]).unwrap();
    response.add_header(header);
    let header = tiny_http::Header::from_bytes(&b"Access-Control-Max-Age"[..], &b"86400"[..]).unwrap();
    response.add_header(header);
    let header = tiny_http::Header::from_bytes(&b"Vary"[..], &b"Origin"[..]).unwrap();
    response.add_header(header);
    let header = tiny_http::Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"body, cache, Content-Type"[..]).unwrap();
    response.add_header(header);
}

