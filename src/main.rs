#[macro_use]
extern crate mysql;

#[macro_use]
extern crate serde_json;

use std::io::{Error as IoError, Read};
use std::path::Path;
use std::fs;

use crypto::sha1::Sha1;
use crypto::digest::Digest;
use eventstore::Connection;
use futures::{Future};
use mysql::Pool;
use serde::{Deserialize, Serialize };
use tiny_http::{Method, Request, Response, Server, StatusCode};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Login {
    passphrase: String
}

#[derive(Debug, Deserialize)]
struct Signup {
    passphrase: String,
    name: String, // use this to send event account created
}

#[derive(Debug, PartialEq, Eq)]
struct Account {
    passphrase: String,
    uuid: String,
    is_admin: bool,
}

struct Dbs {
    sql: Pool,
    event: Connection,
}

fn main() {
    let dbs = init_bdd();

    let server = Server::http("0.0.0.0:8000").unwrap();

    println!("listening on 8000");

    for request in server.incoming_requests() {
        println!("received request!\n, method: {:?}\n, url: {:?}\n, headers: {:?}\n",
                 request.method(),
                 request.url(),
                 request.headers(),
        );
        match handle(request, &dbs) {
            Err(_e) => {
                println!("cannot respond")
            }
            Ok(()) => {}
        }
    }
}

fn init_bdd() -> Dbs {
    // TODO check env == dev

    let sql = match mysql::Pool::new("mysql://root:rootpwd@infra_dev_db_1:3306/service_auth") {
        Ok(pool) => {
            pool
        }
        Err(_e) => {
            let tmp_pool = mysql::Pool::new("mysql://root:rootpwd@infra_dev_db_1:3306/mysql").unwrap();
            tmp_pool.prep_exec(r"CREATE database service_auth", ()).unwrap();

            mysql::Pool::new("mysql://root:rootpwd@infra_dev_db_1:3306/service_auth").unwrap()
        }
    };

    sql.prep_exec(r"CREATE TABLE if not exists account (
                         passphrase varchar(50) not null,
                         uuid varchar(50) not null,
                         name varchar(255) not null,
                         is_admin tinyint not null default 0
                     )", ()).unwrap();


    let event = Connection::builder()
        .single_node_connection("172.28.1.1:1113".parse().unwrap());

    Dbs {
        sql,
        event,
    }
}

fn handle(request: Request, dbs: &Dbs) -> Result<(), IoError> {
    if request.method() == &Method::Post {
        handle_post(request, &dbs)
    } else if request.method() == &Method::Get {
        handle_get(request)
    } else if request.method() == &Method::Options {
        handle_option(request)
    } else {
        let mut response = Response::new_empty(StatusCode(405));
        add_cors(&request, &mut response);
        request.respond(response)
    }
}

fn handle_post(mut request: Request, dbs: &Dbs) -> Result<(), IoError> {
    let mut content = String::new();


    let url = request.url().to_string();
    request.as_reader().read_to_string(&mut content).unwrap();

    if url == "/login" {
        let deserialize: serde_json::Result<Login> = serde_json::from_str(&content.as_str());
        match deserialize {
            Ok(login) => {
                return handle_login(request, login, &dbs);
            }
            Err(_e) => {
                let mut response = Response::new_empty(StatusCode(500));
                add_cors(&request, &mut response);
                return request.respond(response);
            }
        }
    } else if url == "/signup" {
        let deserialize: serde_json::Result<Signup> = serde_json::from_str(&content.as_str());
        match deserialize {
            Ok(signup) => {
                return handle_signup(request, signup, &dbs);
            }
            Err(_e) => {
                let mut response = Response::new_empty(StatusCode(500));
                add_cors(&request, &mut response);
                return request.respond(response);
            }
        }
    }

    let mut response = Response::new_empty(StatusCode(500));
    add_cors(&request, &mut response);
    return request.respond(response);
}

fn handle_login(request: Request, login: Login, dbs: &Dbs) -> Result<(), IoError> {
    let mut hasher = Sha1::new();

    // TODO add salt
    hasher.input_str(login.passphrase.as_str());

    let encoded_pass_phrase = hasher.result_str();

    let uuid_option = get_uuid(encoded_pass_phrase, dbs);

    let response = match uuid_option {
        Some(uuid) => {
            let data = mod_token::Data::new(uuid);

            match mod_token::generate_token(data)
                {
                    Ok(token) => {
                        let mut response = Response::from_string(token.clone());
                        add_cors(&request, &mut response);
                        Some(response)
                    }

                    Err(_e) => {
                        None
                    }
                }
        }
        None => {
            let mut response = Response::from_string("data_required");

            add_cors(&request, &mut response);
            Some(response)
        }
    };

    if response.is_none() {
        let mut response = Response::new_empty(StatusCode(500));
        add_cors(&request, &mut response);
        request.respond(response)
    } else {
        request.respond(response.unwrap())
    }
}

fn handle_signup(request: Request, signup: Signup, dbs: &Dbs) -> Result<(), IoError> {
    let mut hasher = Sha1::new();

    // TODO add salt
    hasher.input_str(signup.passphrase.as_str());

    let encoded_pass_phrase = hasher.result_str();

    let uuid = Uuid::new_v4();

    dbs.sql.prep_exec("INSERT INTO account (passphrase, name, uuid) VALUES (:pp , :name , :uuid)",
                      params! {"pp" => encoded_pass_phrase, "uuid" => uuid.to_string() , "name" => signup.name.clone() }).unwrap();

    let encoded_pass_phrase = hasher.result_str();

    let uuid_option = get_uuid(encoded_pass_phrase, dbs);


    let response = match uuid_option {
        Some(uuid) => {
            let payload = json!(
            event_auth::AccountCreated{
                uuid: uuid.to_string(),
                name: signup.name,
            });

            let event = eventstore::EventData::json("account_created", payload).unwrap();

            let _ = dbs.event.write_events("account")
                .push_event(event)
                .execute()
                .wait()
                .unwrap();


            let data = mod_token::Data::new(uuid);

            match mod_token::generate_token(data)
                {
                    Ok(token) => {
                        let mut response = Response::from_string(token.clone());
                        add_cors(&request, &mut response);

                        Some(response)
                    }
                    Err(_e) => {
                        None
                    }
                }
        }
        None => {
            None
        }
    };


    if response.is_none() {
        let mut response = Response::new_empty(StatusCode(500));
        add_cors(&request, &mut response);
        request.respond(response)
    } else {
        request.respond(response.unwrap())
    }
}

fn get_uuid(encoded_pass_phrase: String, dbs: &Dbs) -> Option<Uuid> {
    let pp = encoded_pass_phrase.clone();
    let accounts: Vec<Account> =
        dbs.sql.prep_exec("SELECT passphrase, uuid, is_admin FROM account WHERE passphrase = :pp", params! {pp})
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
        return None;
    } else {
        let acc = accounts.first().unwrap();
        return Some(Uuid::parse_str(acc.uuid.as_str()).unwrap());
    }
}

fn handle_get(request: Request) -> Result<(), IoError> {
    let authorized = false;

    let url = request.url().to_string();
    let path = Path::new(&url);
    let file = fs::File::open(&path);

    if file.is_ok() {
        let mut response = tiny_http::Response::from_file(file.unwrap());
        let header = tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/javascript"[..]).unwrap();

        response.add_header(header);
        add_cors(&request, &mut response);

        return request.respond(response);
    }


    //TODO check bearer for jwt (or not ?) for authorized = true

    if authorized {
        let mut response = Response::from_string("SUCCESS".to_string());
        add_cors(&request, &mut response);
        request.respond(response)
    } else {
        let mut response = Response::new_empty(StatusCode(403));
        add_cors(&request, &mut response);
        request.respond(response)
    }
}

fn handle_option(request: Request) -> Result<(), IoError> {

    // TODO check "Host"

    let mut response = Response::from_string("ok");

    add_cors(&request, &mut response);

    request.respond(response)
}

fn add_cors<T: Read>(request: &Request, response: &mut Response<T>) {

    //TODO check main domain

    for h in request.headers() {
        if h.field.equiv("Origin") {
            let header = tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], h.value.as_bytes()).unwrap();
            response.add_header(header);
        }
    }

    let header = tiny_http::Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"POST, GET"[..]).unwrap();
    response.add_header(header);
    let header = tiny_http::Header::from_bytes(&b"Access-Control-Max-Age"[..], &b"86400"[..]).unwrap();
    response.add_header(header);
    let header = tiny_http::Header::from_bytes(&b"Vary"[..], &b"Origin"[..]).unwrap();
    response.add_header(header);
    let header = tiny_http::Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"body, cache, Content-Type"[..]).unwrap();
    response.add_header(header);
}

