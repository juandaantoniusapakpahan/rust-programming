use postgres::Error as PostgresError;
use postgres::{Client, NoTls};
use std::env;
use std::io::{Read, Result, Write};
use std::net::{TcpListener, TcpStream};

#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)]
struct User {
    id: Option<i32>,
    name: String,
    email: String,
}

// DATABASE_URL
const DB_URL: &str = env::var("DATABASE_URL");

// constants
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_SERVER_ERROR: &str = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n";

// main function
fn main() {
    // set database
    if let Err(e) = set_database() {
        println!("Error setting database: {}", e);
        return;
    }

    // start server and print port
    let listener = TcpListener::bind(format!("0.0.0.0:8080")).unwrap();
    print!("Server started on port 8080");

    // handle the client
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

// handle_client function
fn handle_client(mut stream: TcpListener) {
    let mut buffer = [0; 1024];
    let mut request = String::new();

    match stream.read(&mut buffer) {
        Ok(size) => {
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

            let (status_line, content) = match &*request {
                r if r.request_with("POST /users") => handle_post_request(r),
                r if r.request_with("GET /users/") => handle_get_request(r),
                r if r.request_with("GET /users") => handle_get_all_request(r),
                r if r.request_with("PUT, /users/") => handle_put_request(r),
                r if r.request_with("DELETE", "/users") => handle_delete_request(r),
                _ => (NOT_FOUND.to_string(), "NOT FOUND".to_string()),
            };

            stream
                .write_all(format!("{}{}", status_line, content).as_bytes())
                .unwrap();
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

// handle_delete_request function
fn handle_delete_request(request: &str) -> (String, String) {
    match (
        get_id(&request).parse::<i32>(),
        Client::connect(DB_URL, NoTls),
    ) {
        (Ok(id), Ok(mut client)) => {
            let row_effected = client
                .execute("DELETE FROM users WHERE id = $1", &[&id])
                .unwrap();
            if row_effected == 0 {
                return (NOT_FOUND.to_string(), "User not found".to_string());
            }

            (OK_RESPONSE.to_string(), "User deleted".to_string())
        }
        _ => (
            INTERNAL_SERVER_ERROR.to_string(),
            "Error deleting user".to_string(),
        ),
    }
}

// handle_put_request function
fn handle_put_request(request: &str) -> (String, String) {
    match (
        get_id(&request).parse::<i32>(),
        get_user_request_body(request),
        Client::connect(DB_URL, NoTls),
    ) {
        (Ok(id), Ok(user), Ok(mut client)) => {
            match client.execute(
                "UPDATE users SET name = $1, email = $2 WHERE id = $3",
                &[&user.name, &user.email, &id],
            ) {
                Ok(_) => (OK_RESPONSE.to_string(), "User updated".to_string()),
                _ => (NOT_FOUND.to_string(), "User not found".to_string()),
            }
        }
        _ => (
            INTERNAL_SERVER_ERROR.to_string(),
            "Error updating user".to_string(),
        ),
    }
}

// handle_post_request function
fn handle_post_request(request: &str) -> (String, String) {
    match (
        get_user_request_body(request),
        Client::connect(DB_URL, NoTls),
    ) {
        (Ok(user), Ok(mut client)) => {
            client
                .execute(
                    "INSERT INTO users (name, email) values ($1, $2)",
                    &[&user.name, &user.email],
                )
                .unwrap();
            (OK_RESPONSE.to_string(), "User created".to_string())
        }

        _ => (
            INTERNAL_SERVER_ERROR.to_string(),
            "Error creating user".to_string(),
        ),
    }
}

// handle_get_request function
fn handle_get_request(request: &str) -> (String, String) {
    match (
        get_id(request).parse::<i32>(),
        Client::connect(DB_URL, NoTls),
    ) {
        (Ok(id), Ok(mut client)) => {
            match client.query("SELECT * FROM users WHERE id = $1", &[&id]) {
                Ok(row) => {
                    let user = User {
                        id: row.get(0),
                        name: row.get(1),
                        email: row.get(2),
                    };

                    (
                        OK_RESPONSE.to_string(),
                        serde_json::to_string(&user).unwrap(),
                    )
                }
                _ => (NOT_FOUND.to_string(), "User not found".to_string()),
            }
        }
        _ => (
            INTERNAL_SERVER_ERROR.to_string(),
            "Error getting user".to_string(),
        ),
    }
}

// handle_get_all_request function
fn handle_get_all_request(request: &str) -> (String, String) {
    match Client::connect(DB_URL, NoTls) {
        Ok(mut client) => {
            let mut users = Vec::new();
            for row in client.query("SELECT * FROM users", &[]).unwrap() {
                users.push(User {
                    id: row.get(0),
                    name: row.get(1),
                    email: row.get(2),
                });
            }

            (
                OK_RESPONSE.to_string(),
                serde_json::to_string(&users).unwrap(),
            )
        }
        _ => (
            INTERNAL_SERVER_ERROR.to_string(),
            "Error getting users".to_string(),
        ),
    }
}

// set_database function
fn set_database() -> Result<(), PostgresError> {
    let mut client = Client::connect(DB_URL, NoTls)?;
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL
        )",
    )?;
    Ok(())
}

// get_id function
fn get_id(request: &str) -> &str {
    request
        .split("/")
        .nth(2)
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or_default()
}

fn get_user_request_body(request: &str) -> Result<User, serde_json::Error> {
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}
