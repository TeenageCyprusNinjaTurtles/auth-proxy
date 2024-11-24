use rouille::Request;
use serde::{Deserialize, Serialize};
mod controller;
mod lib;
mod service_map;

#[derive(Serialize, Deserialize)]
struct AuthInternalResponse {
    token: String,
    level: i32,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct AuthExternalResponse {
    token: String,
    expiration: u64,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    result: String,
}

struct Context {
    is_auth_request: bool,
}

struct UserObject {
    email: String,
    level: i32,
}

impl Context {
    pub fn new() -> Context {
        Context {
            is_auth_request: false,
        }
    }
    pub fn set_is_auth_request(&mut self) {
        self.is_auth_request = true;
    }
    pub fn is_auth_request(&self) -> bool {
        self.is_auth_request
    }
}

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();
    let port = std::env::var("AUTH_PROXY_PORT").unwrap_or("8081".to_string());
    let host = std::env::var("AUTH_PROXY_HOST").unwrap_or("localhost".to_string());

    let redis_host = std::env::var("REDIS_HOST").unwrap_or("127.0.0.1".to_string());
    let redis_port = std::env::var("REDIS_PORT").unwrap_or("6379".to_string());

    let mut service_map = service_map::ServiceMap::new();

    service_map.add_service(
        "auth-service",
        "user",
        "127.0.0.1",
        "AUTH_SERVICE_HOST",
        5432,
        "AUTH_SERVICE_PORT",
        false,
    );
    service_map.add_service(
        "event-service",
        "event",
        "127.0.0.1",
        "EVENT_SERVICE_HOST",
        5001,
        "EVENT_SERVICE_PORT",
        false
    );


    let connection_pool = r2d2::Pool::builder()
        .max_size(15)
        .build(
            redis::Client::open(format!("redis://{}:{}", redis_host, redis_port))
                .expect("Failed to connect to Redis"),
        )
        .expect("Failed to create pool");

    rouille::start_server(
        format!("{}:{}", host, port),
        move |request: &Request| -> rouille::Response {
            let mut context = Context::new();
            let body = lib::utils::request_to_bytes(request);
            let user_data = controller::get_user_object(request, connection_pool.clone());
            if user_data.level == -1 {
                return rouille::Response::json(&ErrorResponse {
                    result: lib::enums::ERROR_RESPONSE_ACCESS_ERROR.to_string(),
                });
            }
            if request.url() == "/user/auth" {
                context.set_is_auth_request();
            }
            let result = controller::pass_by(request, &user_data, &service_map, &body);
            if context.is_auth_request() {
                let auth_data = lib::utils::response_to_bytes(result);
                let auth_response: Result<AuthInternalResponse, serde_json::Error> =
                    serde_json::from_slice(&auth_data);
                if auth_response.is_err() {
                    return rouille::Response::from_data("application/json", auth_data);
                }
                log::info!("Valid Auth Data");
                let auth_response = auth_response.unwrap();
                return controller::on_auth(&auth_response, connection_pool.clone());
            }
            return result;
        },
    );
}
